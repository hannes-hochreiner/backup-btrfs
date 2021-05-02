use std::{collections::HashMap, convert::TryInto, path::{Path, PathBuf}, process::{Command, Output}, str::FromStr};
use chrono::{Utc, SecondsFormat, DateTime, Duration, FixedOffset};
use uuid::Uuid;
use crate::{custom_error::CustomError, custom_duration::CustomDuration};
use anyhow::{Context, Result};

#[derive(Debug, PartialEq, Clone)]
pub struct SnapshotLocal {
    pub path: String,
    pub uuid: Uuid,
    pub parent_uuid: Uuid,
}

#[derive(Debug, PartialEq)]
pub struct SnapshotRemote {
    pub path: String,
    pub uuid: Uuid,
    pub received_uuid: Uuid
}

/// Delete a snapshot
///
/// Executes `btrfs subvolume delete <snapshot_path>`.
///
/// * `snapshot_path` - absolute path of the snapshot to be deleted
///
pub fn delete_snapshot(snapshot_path: &String) -> Result<()> {
    let snapshot_path = PathBuf::from(&*snapshot_path);

    check_dir_absolute(snapshot_path.as_path()).context("snapshot_path must be an absolute path to a directory")?;

    let output = Command::new("btrfs")
        .arg("subvolume")
        .arg("delete")
        .arg(snapshot_path)
        .output().context("running command to obtain subvolume list failed")?;

    check_output(&output).context("output of command to obtain subvolume list contained an error")?;

    Ok(())
}

/// Obtain the output of the command to create a list of subvolumes
///
/// Returns the output of the command `btrfs subvolume list -tupqR --sort=rootid /`.
///
pub fn get_snapshot_list_local() -> Result<String> {
    let output = Command::new("btrfs")
        .arg("subvolume")
        .arg("list")
        .arg("-tupqR")
        .arg("--sort=rootid")
        .arg("/")
        .output().context("running command to obtain subvolume list failed")?;

    let output = check_output(&output).context("output of command to obtain subvolume list contained an error")?;

    Ok(String::from_utf8(output).context("error converting output of the command to obtain the list of subvolumens into a string")?)
}

/// Obtain the output of the command to create a list of subvolumes from a remote host
///
/// Returns the output of the command `sudo <remote_host> "btrfs subvolume list -tupqR --sort=rootid /"`.
///
/// * `remote_host` - name of the remote host
/// * `identity_file_path` - absolute path of the identity file
///
pub fn get_snapshot_list_remote(remote_host: &str, remote_user: &str, identity_file_path: &str) -> Result<String> {
    let output = Command::new("ssh")
        .arg("-i")
        .arg(identity_file_path)
        .arg(format!("{}@{}", remote_user, remote_host))
        .arg("sudo btrfs subvolume list -tupqR --sort=rootid /")
        .output().context("running command to obtain subvolume list failed")?;

    let output = check_output(&output).context("output of command to obtain subvolume list from a remote host contained an error")?;

    Ok(String::from_utf8(output).context("error converting output of the command to obtain the list of subvolumens from a remote host into a string")?)
}

/// Create a new snapshot
///
/// A new snapshot of the subvolume `<subvolume_path>` is created at the snapshot path `<snapshot_path>/<rfc3339 UTC date>_<snapshot_suffix>` is created.
///
/// * `subvolume_path` - absolute path to the subvolume
/// * `snapshot_path` - absolute path to the snapshot directory
/// * `snapshot_suffix` - string to be appended to identify the snapshot
///
pub fn create_snapshot(subvolume_path: &String, snapshot_path: &String, snapshot_suffix: &String) -> Result<()> {
    let snapshot_path_extension = format!("{}_{}", Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true), snapshot_suffix);
    let mut snapshot_path = PathBuf::from(&*snapshot_path);
    let subvolume_path = PathBuf::from(&*subvolume_path);

    check_dir_absolute(subvolume_path.as_path()).context("subvolume_path must be an absolute path to a directory")?;
    check_dir_absolute(snapshot_path.as_path()).context("snapshot_path must be an absolute path to a directory")?;

    snapshot_path.push(snapshot_path_extension);

    let output = Command::new("btrfs")
        .arg("subvolume")
        .arg("snapshot")
        .arg("-r")
        .arg(subvolume_path)
        .arg(snapshot_path)
        .output().context("running command to create subvolume failed")?;
    
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
                    Ok(s) => Err(CustomError::CommandError(format!("command finished with status code {}: {}", code, s)).into()),
                    Err(_) => Err(CustomError::CommandError(format!("command finished with status code {}", code)).into())
                }
            }
        },
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
pub fn find_backups_to_be_deleted(current_timestamp: &DateTime<FixedOffset>, policy: &Vec<CustomDuration>, backups: &Vec<String>) -> Result<Vec<String>> {
    let mut bucket_vec: Vec<String> = Vec::new();
    let mut policy_iter = policy.iter();
    let mut policy = policy_iter.next();
    let mut res: Vec<String> = Vec::new();

    for backup in backups.iter().rev() {
        match policy {
            Some(p) => {
                let pol: Duration = p.try_into().context(format!("could not convert custom interval ({:?}) into chrono::interval", p))?;
                let backup_time = DateTime::parse_from_rfc3339(String::from(Path::new(backup).components().last().ok_or(CustomError::ExtractionError("could not extract last path component".into()))?.as_os_str().to_str().ok_or(CustomError::ExtractionError("could not convert last path component".into()))?)
                    .split("_").nth(0).ok_or(CustomError::ExtractionError("could not find date part of backup name".into()))?)?;

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

            },
            None => {
                bucket_vec.push(backup.clone());
            },
        }
    }

    if bucket_vec.len() > 0 {
        bucket_vec.remove(0);
        res.append(&mut bucket_vec);
    }

    Ok(res)
}

/// Extract the local snapshots for a given subvolume.
///
/// * `path` - path of the subvolume
/// * `subvolume_list` - output of the commant `btrfs subvolume list -tupq --sort=rootid /`
///
pub fn get_local_snapshots(path: &str, subvolume_list: &str) -> Result<Vec<SnapshotLocal>> {
    let mut snapshots: Vec<SnapshotLocal> = Vec::new();

    let mut lines = subvolume_list.split("\n");

    if lines.next().ok_or(CustomError::ExtractionError("could not find header line".into()))?
        .split_ascii_whitespace().collect::<Vec<&str>>() != vec!["ID", "gen", "parent", "top", "level", "parent_uuid", "received_uuid", "uuid", "path"] {
        return Err(CustomError::ExtractionError("unexpected header line".into()).into());
    }

    let mut sv_uuid: Option<Uuid> = None;

    for line in lines.skip(1).into_iter() {
        let tokens: Vec<&str> = line.split_ascii_whitespace().collect();

        if tokens.len() != 8 {
            continue;
        }

        let subvolume_path = format!("/{}", tokens[7]);
        let subvolume_uuid = Uuid::from_str(tokens[6])?;

        match &sv_uuid {
            Some(s) => {
                match Uuid::from_str(tokens[4]) {
                    Ok(parent_uuid) => {
                        if parent_uuid == *s {
                            snapshots.push(SnapshotLocal {
                                path: subvolume_path,
                                uuid: subvolume_uuid,
                                parent_uuid: parent_uuid,
                            });
                        }
                    },
                    Err(_) => {},
                }
            },
            None => {
                if subvolume_path == path {
                    sv_uuid = Some(subvolume_uuid);
                }
            }
        }
    }

    Ok(snapshots)
}

/// Extract the remote snapshots
///
/// * `subvolume_list` - output of the commant `btrfs subvolume list -tupq --sort=rootid /`
///
pub fn get_remote_snapshots(subvolume_list: &str) -> Result<Vec<SnapshotRemote>> {
    let mut snapshots: Vec<SnapshotRemote> = Vec::new();

    let mut lines = subvolume_list.split("\n");

    if lines.next().ok_or(CustomError::ExtractionError("could not find header line".into()))?
        .split_ascii_whitespace().collect::<Vec<&str>>() != vec!["ID", "gen", "parent", "top", "level", "parent_uuid", "received_uuid", "uuid", "path"] {
        return Err(CustomError::ExtractionError("unexpected header line".into()).into());
    }

    for line in lines.skip(1).into_iter() {
        let tokens: Vec<&str> = line.split_ascii_whitespace().collect();

        if tokens.len() != 8 {
            continue;
        }

        let subvolume_path = format!("/{}", tokens[7]);
        let subvolume_uuid = Uuid::from_str(tokens[6])?;

        match Uuid::from_str(tokens[5]) {
            Ok(received_uuid) => {
                snapshots.push(SnapshotRemote {
                    path: subvolume_path,
                    uuid: subvolume_uuid,
                    received_uuid,
                });
            },
            Err(_) => {},
        }
    }

    Ok(snapshots)
}

/// Get the newest snapshot that was used received on the remote host and is still available locally
///
/// * `snapshots_local` - list of local snapshots
/// * `snapshots_remote` - list of remote snapshots
///
pub fn get_common_parent(snapshots_local: &Vec<SnapshotLocal>, snapshots_remote: &Vec<SnapshotRemote>) -> Result<Option<SnapshotLocal>> {
    let mut res: Option<SnapshotLocal> = None;
    let uuids: HashMap<&Uuid, &SnapshotLocal> = snapshots_local.iter().map(|e| (&e.uuid, e)).collect();

    for r in snapshots_remote {
        match uuids.get(&r.received_uuid) {
            Some(&e) => {
                res = Some(e.clone());
            },
            _ => {},
        }
    }

    Ok(res)
}

#[cfg(test)]
mod utils_tests {
    use chrono::{TimeZone, Utc};
    use uuid::Uuid;
    use crate::{CustomDuration, utils::{SnapshotLocal, SnapshotRemote}};
    use crate::utils;

    #[test]
    fn get_common_parent_1() {
        let sl = vec![
            SnapshotLocal { path: "/snapshots/2021-05-02T07:40:32Z_inf_btrfs_test".into(), uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a407").unwrap(), parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap() },
        ];
        let sr = vec![
            SnapshotRemote { path: "/test/path".into(), uuid: Uuid::parse_str("11eed410-7829-744e-8288-35c21d278f8e").unwrap(), received_uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a407").unwrap() },
        ];

        assert_eq!(Some(SnapshotLocal { path: "/snapshots/2021-05-02T07:40:32Z_inf_btrfs_test".into(), uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a407").unwrap(), parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap() }), utils::get_common_parent(&sl, &sr).unwrap());
    }

    #[test]
    fn get_common_parent_2() {
        let sl = vec![
            SnapshotLocal { path: "/snapshots/2021-05-02T07:40:32Z_inf_btrfs_test".into(), uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a408").unwrap(), parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap() },
        ];
        let sr = vec![
            SnapshotRemote { path: "/test/path".into(), uuid: Uuid::parse_str("11eed410-7829-744e-8288-35c21d278f8e").unwrap(), received_uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a407").unwrap() },
        ];

        assert_eq!(None, utils::get_common_parent(&sl, &sr).unwrap());
    }

    #[test]
    fn get_common_parent_3() {
        let sl = vec![
            SnapshotLocal { path: "/snapshots/2021-05-02T07:40:32Z_inf_btrfs_test".into(), uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a408").unwrap(), parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap() },
            SnapshotLocal { path: "/snapshots/2021-05-02T07:40:32Z_inf_btrfs_test".into(), uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a407").unwrap(), parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap() },
        ];
        let sr = vec![
            SnapshotRemote { path: "/test/path".into(), uuid: Uuid::parse_str("11eed410-7829-744e-8288-35c21d278f8e").unwrap(), received_uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a407").unwrap() },
            SnapshotRemote { path: "/test/path".into(), uuid: Uuid::parse_str("11eed410-7829-744e-8288-35c21d278f8e").unwrap(), received_uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a408").unwrap() },
        ];

        assert_eq!(Some(SnapshotLocal { path: "/snapshots/2021-05-02T07:40:32Z_inf_btrfs_test".into(), uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a408").unwrap(), parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap() }), utils::get_common_parent(&sl, &sr).unwrap());
    }

    #[test]
    fn find_backups_to_be_deleted_1() {
        let current = Utc.ymd(2020, 1, 4).and_hms(10, 0, 0);
        let policy = vec![CustomDuration::minutes(15)];
        let backups = vec![
            String::from("/snapshots/2020-01-02T09:00:00Z_host_subvolume"),
            String::from("/snapshots/2020-01-02T09:30:00Z_host_subvolume"),
            String::from("/snapshots/2020-01-03T09:00:00Z_host_subvolume"),
        ];
        let exp = vec![
            String::from("/snapshots/2020-01-02T09:00:00Z_host_subvolume"),
        ];
        assert_eq!(exp, utils::find_backups_to_be_deleted(&current.into(), &policy, &backups).unwrap());
    }

    #[test]
    fn find_backups_to_be_deleted_2() {
        let current = Utc.ymd(2020, 1, 4).and_hms(10, 0, 0);
        let policy = vec![CustomDuration::days(1), CustomDuration::days(2)];
        let backups = vec![
            String::from("/snapshots/2020-01-02T09:00:00Z_host_subvolume"),
            String::from("/snapshots/2020-01-02T09:30:00Z_host_subvolume"),
            String::from("/snapshots/2020-01-03T09:00:00Z_host_subvolume"),
        ];
        let exp: Vec<String> = Vec::new();
        assert_eq!(exp, utils::find_backups_to_be_deleted(&current.into(), &policy, &backups).unwrap());
    }

    #[test]
    fn find_backups_to_be_deleted_3() {
        let current = Utc.ymd(2020, 1, 2).and_hms(09, 35, 0);
        let policy = vec![CustomDuration::minutes(15), CustomDuration::days(1)];
        let backups = vec![
            String::from("/snapshots/2019-12-31T09:00:00Z_host_subvolume"),
            String::from("/snapshots/2020-01-01T09:00:00Z_host_subvolume"),
            String::from("/snapshots/2020-01-02T09:00:00Z_host_subvolume"),
            String::from("/snapshots/2020-01-02T09:15:00Z_host_subvolume"),
            String::from("/snapshots/2020-01-02T09:07:00Z_host_subvolume"),
            String::from("/snapshots/2020-01-02T09:30:00Z_host_subvolume"),
        ];
        let exp = vec![
            String::from("/snapshots/2020-01-02T09:07:00Z_host_subvolume"),
            String::from("/snapshots/2020-01-02T09:15:00Z_host_subvolume"),
            String::from("/snapshots/2019-12-31T09:00:00Z_host_subvolume"),
        ];
        assert_eq!(exp, utils::find_backups_to_be_deleted(&current.into(), &policy, &backups).unwrap());
    }

    #[test]
    fn get_snapshots() {
        let input = r#"ID      gen     parent  top level       parent_uuid     received_uuid   uuid    path
--      ---     ------  ---------       -----------     -------------   ----    ----
256     119496  5       5               -                                       -                                       11eed410-7829-744e-8288-35c21d278f8e    home
359     119496  5       5               -                                       -                                       32c672fa-d3ce-0b4e-8eaa-ab9205f377ca    root
360     119446  359     359             -                                       -                                       5f0b151b-52e4-4445-aa94-d07056733a1f    opt/btrfs_test
367     118687  359     359             5f0b151b-52e4-4445-aa94-d07056733a1f    -                                       7f305e3e-851b-974b-a476-e2f206e7a407    snapshots/2021-05-02T07:40:32Z_inf_btrfs_test
370     119446  359     359             5f0b151b-52e4-4445-aa94-d07056733a1f    -                                       1bd1da76-b61f-db41-a2d2-c3474a31f38f    snapshots/2021-05-02T13:38:49Z_inf_btrfs_test
"#;

        assert_eq!(vec![
            SnapshotLocal { path: "/snapshots/2021-05-02T07:40:32Z_inf_btrfs_test".into(), uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a407").unwrap(), parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap() },
            SnapshotLocal { path: "/snapshots/2021-05-02T13:38:49Z_inf_btrfs_test".into(), uuid: Uuid::parse_str("1bd1da76-b61f-db41-a2d2-c3474a31f38f").unwrap(), parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap() },
        ], utils::get_local_snapshots("/opt/btrfs_test", input).unwrap());
    }
}
