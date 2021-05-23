use std::{collections::HashMap, convert::{TryFrom, TryInto}, path::{Path, PathBuf}, process::{Command, Output}};
use chrono::{DateTime, Duration, FixedOffset, SecondsFormat, Utc};
use uuid::Uuid;
use crate::{btrfs::Subvolume, custom_duration::CustomDuration, custom_error::CustomError};
use anyhow::{Context, Result, anyhow, Error};

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
            parent_uuid: value.parent_uuid.ok_or(anyhow!("no uuid found for snapshot"))?,
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
            received_uuid: value.received_uuid.ok_or(anyhow!("no uuid found for snapshot"))?,
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

fn get_timestamp_suffix_from_snapshot_path(snapshot_path: &String) -> Result<(chrono::DateTime<FixedOffset>, String)> {
    let snapshot_name = String::from(Path::new(snapshot_path).components().last().ok_or(CustomError::ExtractionError("could not extract last path component".into()))?.as_os_str().to_str().ok_or(CustomError::ExtractionError("could not convert last path component".into()))?);
    let mut snapshot_tokens = snapshot_name.split("_");
    let snapshot_timestamp = DateTime::parse_from_rfc3339(snapshot_tokens.nth(0).ok_or(CustomError::ExtractionError("could not find date part of backup name".into()))?)?;

    Ok((snapshot_timestamp, snapshot_tokens.collect::<Vec<&str>>().join("_")))
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
pub fn find_backups_to_be_deleted(current_timestamp: &DateTime<FixedOffset>, policy: &Vec<CustomDuration>, backups: &Vec<String>, snapshot_suffix: &String) -> Result<Vec<String>> {
    let mut bucket_vec: Vec<String> = Vec::new();
    let mut policy_iter = policy.iter();
    let mut policy = policy_iter.next();
    let mut res: Vec<String> = Vec::new();

    for backup in backups.iter().rev() {
        match policy {
            Some(p) => {
                let pol: Duration = p.try_into().context(format!("could not convert custom interval ({:?}) into chrono::interval", p))?;
                let (backup_time, backup_suffix) = get_timestamp_suffix_from_snapshot_path(backup).context("error extracting snapshot timestamp and suffix")?;

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

/// Get a reference to a subvolume from an iterator over subvolumes based on the path.
///
/// * `path` - path of the subvolume
/// * `subvolume_list` - iterator over subvolumes
///
pub fn get_subvolume_by_path<'a>(path: &str, subvolumes: &mut impl Iterator<Item = &'a Subvolume>) -> Result<&'a Subvolume> {
    subvolumes.find(|e| &*e.path == path).ok_or(anyhow!("subvolume to be backed up not found"))
}

/// Filter out the snapshots of a subvolume from a list of subvolumes.
///
/// The matching items are converted into local snapshot types.
///
/// * `subvolume` - subvolume for which to find snapshots
/// * `subvolumes` - list of subvolumes
///
pub fn get_local_snapshots<'a>(subvolume: &Subvolume, subvolumes: &mut impl Iterator<Item = &'a Subvolume>) -> Result<Vec<SnapshotLocal>> {
    subvolumes.filter(|e| {
        match e.parent_uuid {
            Some(parent_uuid) => parent_uuid == subvolume.uuid,
            None => false,
        }
    }).map(|e| e.try_into()).collect()
}

/// Extract the remote snapshots
///
/// * `subvolumes` - list of subvolumes
///
pub fn get_remote_snapshots<'a>(subvolumes: &mut impl Iterator<Item = &'a Subvolume>) -> Result<Vec<SnapshotRemote>> {
    subvolumes.filter(|e| {
        match e.received_uuid {
            Some(..) => true,
            None => false,
        }
    }).map(|e| e.try_into()).collect()
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
mod tests {
    use chrono::{TimeZone, Utc};
    use uuid::Uuid;
    use crate::{
        utils::{SnapshotLocal, SnapshotRemote}
    };

    #[test]
    fn get_subvolume_by_path() {
        todo!()
    }

    #[test]
    fn get_common_parent_1() {
        let sl = vec![
            SnapshotLocal {
                path: "/snapshots/2021-05-02T07:40:32Z_inf_btrfs_test".into(),
                timestamp: Utc.ymd(2021, 5, 2).and_hms(7, 40, 32).into(),
                uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a407").unwrap(),
                parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
                suffix: "inf_btrfs_test".into(),
            },
        ];
        let sr = vec![
            SnapshotRemote {
                path: "/test/path".into(),
                timestamp: Utc.ymd(2021, 5, 2).and_hms(7, 40, 32).into(),
                uuid: Uuid::parse_str("11eed410-7829-744e-8288-35c21d278f8e").unwrap(),
                received_uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a407").unwrap(),
                suffix: "inf_btrfs_test".into(),
            },
        ];

        assert_eq!(Some(SnapshotLocal {
            path: "/snapshots/2021-05-02T07:40:32Z_inf_btrfs_test".into(),
            timestamp: Utc.ymd(2021, 5, 2).and_hms(7, 40, 32).into(),
            uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a407").unwrap(),
            parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
            suffix: "inf_btrfs_test".into(),
        }), crate::utils::get_common_parent(&sl, &sr).unwrap());
    }

    #[test]
    fn get_common_parent_2() {
        let sl = vec![
            SnapshotLocal {
                path: "/snapshots/2021-05-02T07:40:32Z_inf_btrfs_test".into(),
                timestamp: Utc.ymd(2021, 5, 2).and_hms(7, 40, 32).into(),
                uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a408").unwrap(),
                parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
                suffix: "inf_btrfs_test".into(),
            },
        ];
        let sr = vec![
            SnapshotRemote {
                path: "/test/path".into(),
                timestamp: Utc.ymd(2021, 5, 2).and_hms(7, 40, 32).into(),
                uuid: Uuid::parse_str("11eed410-7829-744e-8288-35c21d278f8e").unwrap(),
                received_uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a407").unwrap(),
                suffix: "inf_btrfs_test".into(),
            },
        ];

        assert_eq!(None, crate::utils::get_common_parent(&sl, &sr).unwrap());
    }

    #[test]
    fn get_common_parent_3() {
        let sl = vec![
            SnapshotLocal {
                path: "/snapshots/2021-05-02T07:40:32Z_inf_btrfs_test".into(),
                timestamp: Utc.ymd(2021, 5, 2).and_hms(7, 40, 32).into(),
                uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a408").unwrap(),
                parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
                suffix: "inf_btrfs_test".into(),
            },
            SnapshotLocal {
                path: "/snapshots/2021-05-02T07:40:32Z_inf_btrfs_test".into(),
                timestamp: Utc.ymd(2021, 5, 2).and_hms(7, 40, 32).into(),
                uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a407").unwrap(),
                parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
                suffix: "inf_btrfs_test".into(),
            },
        ];
        let sr = vec![
            SnapshotRemote {
                path: "/test/path".into(),
                timestamp: Utc.ymd(2021, 5, 2).and_hms(7, 40, 32).into(),
                uuid: Uuid::parse_str("11eed410-7829-744e-8288-35c21d278f8e").unwrap(),
                received_uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a407").unwrap(),
                suffix: "inf_btrfs_test".into(),
            },
            SnapshotRemote {
                path: "/test/path".into(),
                timestamp: Utc.ymd(2021, 5, 2).and_hms(7, 40, 32).into(),
                uuid: Uuid::parse_str("11eed410-7829-744e-8288-35c21d278f8e").unwrap(),
                received_uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a408").unwrap(),
                suffix: "inf_btrfs_test".into(),
            },
        ];

        assert_eq!(Some(SnapshotLocal {
            path: "/snapshots/2021-05-02T07:40:32Z_inf_btrfs_test".into(),
            timestamp: Utc.ymd(2021, 5, 2).and_hms(7, 40, 32).into(),
            uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a408").unwrap(),
            parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
            suffix: "inf_btrfs_test".into(),
        }), crate::utils::get_common_parent(&sl, &sr).unwrap());
    }

//     #[test]
//     fn find_backups_to_be_deleted_1() {
//         let current = Utc.ymd(2020, 1, 4).and_hms(10, 0, 0);
//         let policy = vec![CustomDuration::minutes(15)];
//         let backups = vec![
//             String::from("/snapshots/2020-01-02T09:00:00Z_host_subvolume"),
//             String::from("/snapshots/2020-01-02T09:30:00Z_host_subvolume"),
//             String::from("/snapshots/2020-01-03T09:00:00Z_host_subvolume"),
//         ];
//         let exp = vec![
//             String::from("/snapshots/2020-01-02T09:00:00Z_host_subvolume"),
//         ];
//         assert_eq!(exp, utils::find_backups_to_be_deleted(&current.into(), &policy, &backups, &String::from("host_subvolume")).unwrap());
//     }

//     #[test]
//     fn find_backups_to_be_deleted_2() {
//         let current = Utc.ymd(2020, 1, 4).and_hms(10, 0, 0);
//         let policy = vec![CustomDuration::days(1), CustomDuration::days(2)];
//         let backups = vec![
//             String::from("/snapshots/2020-01-02T09:00:00Z_host_subvolume"),
//             String::from("/snapshots/2020-01-02T09:30:00Z_host_subvolume"),
//             String::from("/snapshots/2020-01-03T09:00:00Z_host_subvolume"),
//         ];
//         let exp: Vec<String> = Vec::new();
//         assert_eq!(exp, utils::find_backups_to_be_deleted(&current.into(), &policy, &backups, &String::from("host_subvolume")).unwrap());
//     }

//     #[test]
//     fn find_backups_to_be_deleted_3() {
//         let current = Utc.ymd(2020, 1, 2).and_hms(09, 35, 0);
//         let policy = vec![CustomDuration::minutes(15), CustomDuration::days(1)];
//         let backups = vec![
//             String::from("/snapshots/2019-12-31T09:00:00Z_host_subvolume"),
//             String::from("/snapshots/2020-01-01T09:00:00Z_host_subvolume"),
//             String::from("/snapshots/2020-01-02T09:00:00Z_host_subvolume"),
//             String::from("/snapshots/2020-01-02T09:12:00Z_host2_subvolume"),
//             String::from("/snapshots/2020-01-02T09:15:00Z_host_subvolume"),
//             String::from("/snapshots/2020-01-02T09:07:00Z_host_subvolume"),
//             String::from("/snapshots/2020-01-02T09:30:00Z_host_subvolume"),
//         ];
//         let exp = vec![
//             String::from("/snapshots/2020-01-02T09:07:00Z_host_subvolume"),
//             String::from("/snapshots/2020-01-02T09:15:00Z_host_subvolume"),
//             String::from("/snapshots/2019-12-31T09:00:00Z_host_subvolume"),
//         ];
//         assert_eq!(exp, utils::find_backups_to_be_deleted(&current.into(), &policy, &backups, &String::from("host_subvolume")).unwrap());
//     }

//     #[test]
//     fn find_backups_to_be_deleted_4() {
//         let current = Utc.ymd(2020, 1, 2).and_hms(09, 35, 0);
//         let policy: Vec<CustomDuration> = Vec::new();
//         let backups = vec![
//             String::from("/snapshots/2019-12-31T09:00:00Z_host_subvolume"),
//             String::from("/snapshots/2020-01-01T09:00:00Z_host_subvolume"),
//             String::from("/snapshots/2020-01-02T09:00:00Z_host_subvolume"),
//             String::from("/snapshots/2020-01-02T09:12:00Z_host2_subvolume"),
//             String::from("/snapshots/2020-01-02T09:15:00Z_host_subvolume"),
//             String::from("/snapshots/2020-01-02T09:07:00Z_host_subvolume"),
//             String::from("/snapshots/2020-01-02T09:30:00Z_host_subvolume"),
//         ];
//         let exp = vec![
//             String::from("/snapshots/2020-01-02T09:07:00Z_host_subvolume"),
//             String::from("/snapshots/2020-01-02T09:15:00Z_host_subvolume"),
//             String::from("/snapshots/2020-01-02T09:12:00Z_host2_subvolume"),
//             String::from("/snapshots/2020-01-02T09:00:00Z_host_subvolume"),
//             String::from("/snapshots/2020-01-01T09:00:00Z_host_subvolume"),
//             String::from("/snapshots/2019-12-31T09:00:00Z_host_subvolume"),
//         ];
//         assert_eq!(exp, utils::find_backups_to_be_deleted(&current.into(), &policy, &backups, &String::from("host_subvolume")).unwrap());
//     }

//     #[test]
//     fn get_local_snapshots() {
//         let input = r#"ID      gen     parent  top level       parent_uuid     received_uuid   uuid    path
// --      ---     ------  ---------       -----------     -------------   ----    ----
// 256     119496  5       5               -                                       -                                       11eed410-7829-744e-8288-35c21d278f8e    home
// 359     119496  5       5               -                                       -                                       32c672fa-d3ce-0b4e-8eaa-ab9205f377ca    root
// 360     119446  359     359             -                                       -                                       5f0b151b-52e4-4445-aa94-d07056733a1f    opt/btrfs_test
// 367     118687  359     359             5f0b151b-52e4-4445-aa94-d07056733a1f    -                                       7f305e3e-851b-974b-a476-e2f206e7a407    snapshots/2021-05-02T07:40:32Z_inf_btrfs_test
// 370     119446  359     359             5f0b151b-52e4-4445-aa94-d07056733a1f    -                                       1bd1da76-b61f-db41-a2d2-c3474a31f38f    snapshots/2021-05-02T13:38:49Z_inf_btrfs_test
// "#;

//         match utils::get_local_snapshots("/opt/btrfs_test", input) {
//             Err(e) => println!("{}", e),
//             Ok(_) => {}
//         }

//         assert_eq!(vec![
//             SnapshotLocal { path: "/snapshots/2021-05-02T07:40:32Z_inf_btrfs_test".into(), timestamp: Utc.ymd(2021, 5, 2).and_hms(7, 40, 32).into(), uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a407").unwrap(), parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap() },
//             SnapshotLocal { path: "/snapshots/2021-05-02T13:38:49Z_inf_btrfs_test".into(), timestamp: Utc.ymd(2021, 5, 2).and_hms(13, 38, 49).into(), uuid: Uuid::parse_str("1bd1da76-b61f-db41-a2d2-c3474a31f38f").unwrap(), parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap() },
//         ], utils::get_local_snapshots("/opt/btrfs_test", input).unwrap());
//     }

//     #[test]
//     fn get_timestamp_suffix_from_snapshot_path() {
//         let (timestamp, suffix) = utils::get_timestamp_suffix_from_snapshot_path(&String::from("/opt/snapshots/2021-05-12T04:23:12Z_exo_btrfs_test")).unwrap();

//         assert_eq!(Utc.ymd(2021, 05, 12).and_hms(4, 23, 12), timestamp);
//         assert_eq!(String::from("exo_btrfs_test"), suffix);
//     }
}
