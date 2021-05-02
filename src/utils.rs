use std::{
    path::{
        Path,
        PathBuf,
    },
    process::Command,
};
use chrono::{Utc, SecondsFormat, DateTime, Duration, FixedOffset};
use crate::custom_error::CustomError;
use anyhow::{Context, Result};

/// Create a new snapshot
///
/// A new snapshot of the subvolume `<subvolume_path>` is created at the snapshot path `<snapshot_path>/<rfc3339 UTC date>_<snapshot_suffix>` is created.
///
/// * `subvolume_path` - absolute path to the subvolume
/// * `snapshot_path` - absolute path to the snapshot directory
/// * `snapshot_suffix` - string to be appended to identify the snapshot
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
            .output()?;
    
    match output.status.code() {
        Some(code) => {
            if code == 0 {
                Ok(())
            } else {
                Err(CustomError::CommandError(format!("command finished with status code {}", code)).into())
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
pub fn find_backups_to_be_deleted(current_timestamp: &DateTime<FixedOffset>, policy: &Vec<Duration>, backups: &Vec<String>) -> Result<Vec<String>> {
    let mut bucket_vec: Vec<String> = Vec::new();
    let mut policy_iter = policy.iter();
    let mut policy = policy_iter.next();
    let mut res: Vec<String> = Vec::new();

    for backup in backups.iter().rev() {
        match policy {
            Some(&p) => {
                let backup_time = DateTime::parse_from_rfc3339(String::from(Path::new(backup).components().last().ok_or(CustomError::ExtractionError("could not extract last path component".into()))?.as_os_str().to_str().ok_or(CustomError::ExtractionError("could not convert last path component".into()))?)
                    .split("_").nth(0).ok_or(CustomError::ExtractionError("could not find date part of backup name".into()))?)?;

                if *current_timestamp - backup_time > p {
                    if bucket_vec.len() > 0 {
                        bucket_vec.pop();
                        println!("append to result: {:?}", bucket_vec);
                        res.append(&mut bucket_vec);
                        println!("append to bucket_vec: {:?}", backup);
                        bucket_vec.push(backup.clone());
                    }
                    
                    println!("switch policy");
                    policy = policy_iter.next();
                } else {
                    println!("append to bucket_vec: {:?}", backup);
                    bucket_vec.push(backup.clone());
                }

            },
            None => {
                println!("append to bucket_vec: {:?}", backup);
                bucket_vec.push(backup.clone());
            },
        }
    }

    if bucket_vec.len() > 0 {
        bucket_vec.remove(0);
        println!("append to result: {:?}", bucket_vec);
        res.append(&mut bucket_vec);
    }

    Ok(res)
}

/// Extract the snapshots for a given subvolume.
///
/// * `path` - path of the subvolume
/// * `subvolume_list` - output of the commant `sudo btrfs subvolume list -tupq --sort=rootid /`
pub fn get_snapshots(path: &str, subvolume_list: &str) -> Result<Vec<String>> {
    let mut snapshots: Vec<String> = Vec::new();

    let mut lines = subvolume_list.split("\n");

    if lines.next().ok_or(CustomError::ExtractionError("could not find header line".into()))?
        .split_ascii_whitespace().collect::<Vec<&str>>() != vec!["ID", "gen", "parent", "top", "level", "parent_uuid", "uuid", "path"] {
        return Err(CustomError::ExtractionError("unexpected header line".into()).into());
    }

    let root = String::from("/");
    let mut sv_uuid: Option<String> = None;

    for line in lines.skip(1).into_iter() {
        let tokens: Vec<&str> = line.split_ascii_whitespace().collect();

        match &sv_uuid {
            Some(s) => {
                if tokens[4] == s {
                    snapshots.push(root.clone() + tokens[6]);
                }
            },
            None => {
                if root.clone() + tokens[6] == path {
                    sv_uuid = Some(tokens[5].into());
                }
            }
        }
    }

    Ok(snapshots)
}

#[cfg(test)]
mod utils_tests {
    use chrono::{TimeZone, Utc, Duration};
    use crate::utils;

    #[test]
    fn find_backups_to_be_deleted_1() {
        let current = Utc.ymd(2020, 1, 4).and_hms(10, 0, 0);
        let policy = vec![Duration::minutes(15)];
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
        let policy = vec![Duration::days(1), Duration::days(2)];
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
        let policy = vec![Duration::minutes(15), Duration::days(1)];
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
        let input = r#"ID      gen     parent  top level       parent_uuid     uuid    path
--      ---     ------  ---------       -----------     ----    ----
256     112747  5       5               -                                       11eed410-7829-744e-8288-35c21d278f8e    home
359     112747  5       5               -                                       32c672fa-d3ce-0b4e-8eaa-ab9205f377ca    root
360     112737  359     359             -                                       5f0b151b-52e4-4445-aa94-d07056733a1f    opt/btrfs_test
361     107324  359     359             5f0b151b-52e4-4445-aa94-d07056733a1f    8d5c1a34-2c33-c646-8bb6-0723e2c5c356    snapshots/2021-04-29T15:54:00Z_inf_btrfs_test
362     112737  360     360             -                                       099b9497-11ad-b14b-838a-79e5e7b6084e    opt/btrfs_test/test2
363     112744  256     256             -                                       d7a747f8-aed0-9846-82d1-7dd2ed38705f    home/test"#;

        assert_eq!(vec!["/snapshots/2021-04-29T15:54:00Z_inf_btrfs_test"], utils::get_snapshots("/opt/btrfs_test", input).unwrap());
    }
}
