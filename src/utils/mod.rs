use crate::{btrfs::Subvolume, custom_duration::CustomDuration};
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Duration, FixedOffset};
use std::{collections::HashMap, convert::TryInto};
use uuid::Uuid;
pub mod snapshot;
#[cfg(test)]
mod tests;
use self::snapshot::Snapshot;
use snapshot::{SnapshotLocal, SnapshotRemote};

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
pub fn find_backups_to_be_deleted<'a>(
    current_timestamp: &DateTime<FixedOffset>,
    policy: &Vec<CustomDuration>,
    backups: &Vec<&'a dyn Snapshot>,
    snapshot_suffix: &str,
) -> Result<Vec<&'a dyn Snapshot>> {
    let mut bucket_vec: Vec<&'a dyn Snapshot> = Vec::new();
    let mut policy_iter = policy.iter();
    let mut policy = policy_iter.next();
    let mut res: Vec<&'a dyn Snapshot> = Vec::new();
    let mut backups = backups.clone();

    backups.sort_by_key(|&e| e.timestamp());

    for &backup in backups.iter().rev() {
        if backup.suffix() != snapshot_suffix {
            continue;
        }

        match policy {
            Some(p) => {
                let pol: Duration = p.try_into().context(format!(
                    "could not convert custom interval ({:?}) into chrono::interval",
                    p
                ))?;

                println!("time: {}", *current_timestamp - *backup.timestamp());
                if *current_timestamp - *backup.timestamp() > pol {
                    if bucket_vec.len() > 0 {
                        bucket_vec.pop();
                        res.append(&mut bucket_vec);
                        bucket_vec.push(backup);
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
pub fn get_common_parent<'a>(
    snapshots_local: &'a Vec<SnapshotLocal>,
    snapshots_remote: &Vec<SnapshotRemote>,
) -> Result<Option<&'a SnapshotLocal>> {
    let mut res: Option<&SnapshotLocal> = None;
    let uuids: HashMap<&Uuid, &SnapshotLocal> =
        snapshots_local.iter().map(|e| (&e.uuid, e)).collect();

    for r in snapshots_remote {
        match uuids.get(&r.received_uuid) {
            Some(&e) => {
                res = Some(e);
            }
            _ => {}
        }
    }

    Ok(res)
}
