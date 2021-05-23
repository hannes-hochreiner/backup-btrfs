use crate::{btrfs::Subvolume, custom_duration::CustomDuration, custom_error::CustomError};
use anyhow::{anyhow, Context, Error, Result};
use chrono::{DateTime, Duration, FixedOffset, SecondsFormat, Utc};
use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
    path::{Path, PathBuf},
    process::{Command, Output},
};
use uuid::Uuid;
#[cfg(test)]
mod tests;

#[derive(Debug, PartialEq, Clone)]
pub struct SnapshotLocal {
    pub path: String,
    pub timestamp: chrono::DateTime<FixedOffset>,
    pub uuid: Uuid,
    pub parent_uuid: Uuid,
    pub suffix: String,
}

impl TryFrom<&Subvolume> for SnapshotLocal {
    type Error = Error;

    fn try_from(value: &Subvolume) -> Result<Self, Self::Error> {
        let (timestamp, suffix) = get_timestamp_suffix_from_snapshot_path(&value.path)?;

        Ok(SnapshotLocal {
            parent_uuid: value
                .parent_uuid
                .ok_or(anyhow!("no uuid found for snapshot"))?,
            path: value.path.clone(),
            timestamp,
            uuid: value.uuid,
            suffix,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct SnapshotRemote {
    pub path: String,
    pub timestamp: chrono::DateTime<FixedOffset>,
    pub uuid: Uuid,
    pub received_uuid: Uuid,
    pub suffix: String,
}

impl TryFrom<&Subvolume> for SnapshotRemote {
    type Error = Error;

    fn try_from(value: &Subvolume) -> Result<Self, Self::Error> {
        let (timestamp, suffix) = get_timestamp_suffix_from_snapshot_path(&value.path)?;

        Ok(SnapshotRemote {
            received_uuid: value
                .received_uuid
                .ok_or(anyhow!("no uuid found for snapshot"))?,
            path: value.path.clone(),
            timestamp,
            uuid: value.uuid,
            suffix,
        })
    }
}

/// Send snapshot
///
/// * `snapshot` - snapshot to be sent
/// * `parent_snapshot` - an optional parent snapshot
/// * `backup_path` - absolute path for backups on the remote host
/// * `config_ssh` - ssh configuration
///
// pub fn send_snapshot(snapshot: &SnapshotLocal, parent_snapshot: &Option<SnapshotLocal>, backup_path: &str, config_ssh: &ConfigSsh) -> Result<()> {
//     let mut args = vec!["send"];

//     match parent_snapshot {
//         Some(ps) => {
//             args.push("-p");
//             args.push(&*ps.path);
//         },
//         _ => {}
//     }

//     args.push(&snapshot.path);

//     let cmd_btrfs = Command::new("btrfs")
//         .args(args)
//         .stdout(Stdio::piped())
//         .spawn().context("error running btrfs send command")?;

//     let cmd_ssh = Command::new("ssh")
//         .arg("-i")
//         .arg(&config_ssh.identity_file_path)
//         .arg(format!("{}@{}", config_ssh.remote_user, config_ssh.remote_host))
//         .arg(format!("sudo btrfs receive \"{}\"", backup_path))
//         .stdin(cmd_btrfs.stdout.ok_or(CustomError::CommandError("could not open stdout".into()))?)
//         .output().context("error running ssh command")?;

//     check_output(&cmd_ssh).context("error checking btrfs output")?;

//     Ok(())
// }

/// Create a new snapshot
///
/// A new snapshot of the subvolume `<subvolume_path>` is created at the snapshot path `<snapshot_path>/<rfc3339 UTC date>_<snapshot_suffix>` is created.
///
/// * `subvolume_path` - absolute path to the subvolume
/// * `snapshot_path` - absolute path to the snapshot directory
/// * `snapshot_suffix` - string to be appended to identify the snapshot
///
pub fn create_snapshot(
    subvolume_path: &String,
    snapshot_path: &String,
    snapshot_suffix: &String,
) -> Result<()> {
    let snapshot_path_extension = format!(
        "{}_{}",
        Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        snapshot_suffix
    );
    let mut snapshot_path = PathBuf::from(&*snapshot_path);
    let subvolume_path = PathBuf::from(&*subvolume_path);

    check_dir_absolute(subvolume_path.as_path())
        .context("subvolume_path must be an absolute path to a directory")?;
    check_dir_absolute(snapshot_path.as_path())
        .context("snapshot_path must be an absolute path to a directory")?;

    snapshot_path.push(snapshot_path_extension);

    let output = Command::new("btrfs")
        .arg("subvolume")
        .arg("snapshot")
        .arg("-r")
        .arg(subvolume_path)
        .arg(snapshot_path)
        .output()
        .context("running command to create subvolume failed")?;

    check_output(&output).context("output of command to create subvolume contained an error")?;

    Ok(())
}

fn check_output(output: &Output) -> Result<Vec<u8>> {
    match output.status.code() {
        Some(code) => {
            if code == 0 {
                Ok(output.stdout.clone())
            } else {
                match String::from_utf8(output.stderr.clone()) {
                    Ok(s) => Err(CustomError::CommandError(format!(
                        "command finished with status code {}: {}",
                        code, s
                    ))
                    .into()),
                    Err(_) => Err(CustomError::CommandError(format!(
                        "command finished with status code {}",
                        code
                    ))
                    .into()),
                }
            }
        }
        None => Err(CustomError::CommandError("command was terminated by signal".into()).into()),
    }
}

fn check_dir_absolute(path: &Path) -> Result<()> {
    if !path.is_dir() {
        return Err(CustomError::ConfigurationError("path is not a directory".into()).into());
    }

    if !path.is_absolute() {
        return Err(CustomError::ConfigurationError("path is not an absolute path".into()).into());
    }

    Ok(())
}

fn get_timestamp_suffix_from_snapshot_path(
    snapshot_path: &String,
) -> Result<(chrono::DateTime<FixedOffset>, String)> {
    let snapshot_name = String::from(
        Path::new(snapshot_path)
            .components()
            .last()
            .ok_or(CustomError::ExtractionError(
                "could not extract last path component".into(),
            ))?
            .as_os_str()
            .to_str()
            .ok_or(CustomError::ExtractionError(
                "could not convert last path component".into(),
            ))?,
    );
    let mut snapshot_tokens = snapshot_name.split("_");
    let snapshot_timestamp = DateTime::parse_from_rfc3339(snapshot_tokens.nth(0).ok_or(
        CustomError::ExtractionError("could not find date part of backup name".into()),
    )?)?;

    Ok((
        snapshot_timestamp,
        snapshot_tokens.collect::<Vec<&str>>().join("_"),
    ))
}

/// Return a list of backups to be deleted
///
/// The policy vector is taken to create bucket, where the entries in the vector are the limits.
/// Two additional buckets are created for the time before the last entry of the vector and after the first entry.
/// All backups are assigned to these buckets starting with the nearest past.
/// If there is no entry found for a bucket, the next newest entry will be used.
/// If there are multiple entries in one bucket, the oldest entry will be kept.
/// For the bucket with the oldest entries, only the newest entry will be kept.
///
/// * `current_timestamp` - current date and time
/// * `policy` - a vector of durations ordered from smallest to largest
/// * `backups` - a vector of backup path names; the last component of the path must follow the format `<rfc3339 date>_<other text>`; the entries must be sorted oldest to newest
///
pub fn find_backups_to_be_deleted(
    current_timestamp: &DateTime<FixedOffset>,
    policy: &Vec<CustomDuration>,
    backups: &Vec<String>,
    snapshot_suffix: &String,
) -> Result<Vec<String>> {
    let mut bucket_vec: Vec<String> = Vec::new();
    let mut policy_iter = policy.iter();
    let mut policy = policy_iter.next();
    let mut res: Vec<String> = Vec::new();

    for backup in backups.iter().rev() {
        match policy {
            Some(p) => {
                let pol: Duration = p.try_into().context(format!(
                    "could not convert custom interval ({:?}) into chrono::interval",
                    p
                ))?;
                let (backup_time, backup_suffix) = get_timestamp_suffix_from_snapshot_path(backup)
                    .context("error extracting snapshot timestamp and suffix")?;

                if backup_suffix != *snapshot_suffix {
                    continue;
                }

                if *current_timestamp - backup_time > pol {
                    if bucket_vec.len() > 0 {
                        bucket_vec.pop();
                        res.append(&mut bucket_vec);
                        bucket_vec.push(backup.clone());
                    }

                    policy = policy_iter.next();
                } else {
                    bucket_vec.push(backup.clone());
                }
            }
            None => {
                bucket_vec.push(backup.clone());
            }
        }
    }

    if bucket_vec.len() > 0 {
        bucket_vec.remove(0);
        res.append(&mut bucket_vec);
    }

    Ok(res)
}

/// Get a reference to a subvolume from an iterator over subvolumes based on the path.
///
/// * `path` - path of the subvolume
/// * `subvolume_list` - iterator over subvolumes
///
pub fn get_subvolume_by_path<'a>(
    path: &str,
    subvolumes: &mut impl Iterator<Item = &'a Subvolume>,
) -> Result<&'a Subvolume> {
    subvolumes
        .find(|e| &*e.path == path)
        .ok_or(anyhow!("subvolume to be backed up not found"))
}

/// Filter out the snapshots of a subvolume from a list of subvolumes.
///
/// The matching items are converted into local snapshot types.
///
/// * `subvolume` - subvolume for which to find snapshots
/// * `subvolumes` - list of subvolumes
///
pub fn get_local_snapshots<'a>(
    subvolume: &Subvolume,
    subvolumes: &mut impl Iterator<Item = &'a Subvolume>,
) -> Result<Vec<SnapshotLocal>> {
    subvolumes
        .filter(|e| match e.parent_uuid {
            Some(parent_uuid) => parent_uuid == subvolume.uuid,
            None => false,
        })
        .map(|e| e.try_into())
        .collect()
}

/// Extract the remote snapshots
///
/// * `subvolumes` - list of subvolumes
///
pub fn get_remote_snapshots<'a>(
    subvolumes: &mut impl Iterator<Item = &'a Subvolume>,
) -> Result<Vec<SnapshotRemote>> {
    subvolumes
        .filter(|e| match e.received_uuid {
            Some(..) => true,
            None => false,
        })
        .map(|e| e.try_into())
        .collect()
}

/// Get the newest snapshot that was used received on the remote host and is still available locally
///
/// * `snapshots_local` - list of local snapshots
/// * `snapshots_remote` - list of remote snapshots
///
pub fn get_common_parent(
    snapshots_local: &Vec<SnapshotLocal>,
    snapshots_remote: &Vec<SnapshotRemote>,
) -> Result<Option<SnapshotLocal>> {
    let mut res: Option<SnapshotLocal> = None;
    let uuids: HashMap<&Uuid, &SnapshotLocal> =
        snapshots_local.iter().map(|e| (&e.uuid, e)).collect();

    for r in snapshots_remote {
        match uuids.get(&r.received_uuid) {
            Some(&e) => {
                res = Some(e.clone());
            }
            _ => {}
        }
    }

    Ok(res)
}
