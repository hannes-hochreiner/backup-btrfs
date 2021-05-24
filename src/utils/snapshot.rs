use anyhow::{anyhow, Error, Result};
use chrono::{DateTime, FixedOffset};
use std::{convert::TryFrom, fmt::Debug, path::Path};
use uuid::Uuid;

use crate::btrfs::Subvolume;

pub trait Snapshot {
    fn path(&self) -> &str;
    fn timestamp(&self) -> &DateTime<FixedOffset>;
    fn suffix(&self) -> &str;
}

#[derive(Debug, PartialEq, Clone)]
pub struct SnapshotLocal {
    pub path: String,
    pub timestamp: chrono::DateTime<FixedOffset>,
    pub uuid: Uuid,
    pub parent_uuid: Uuid,
    pub suffix: String,
}

impl Snapshot for SnapshotLocal {
    fn path(&self) -> &str {
        &*self.path
    }

    fn timestamp(&self) -> &DateTime<FixedOffset> {
        &self.timestamp
    }

    fn suffix(&self) -> &str {
        &*self.suffix
    }
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

impl Snapshot for SnapshotRemote {
    fn path(&self) -> &str {
        &*self.path
    }

    fn timestamp(&self) -> &DateTime<FixedOffset> {
        &self.timestamp
    }

    fn suffix(&self) -> &str {
        &*self.suffix
    }
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

fn get_timestamp_suffix_from_snapshot_path(
    snapshot_path: &String,
) -> Result<(chrono::DateTime<FixedOffset>, String)> {
    let snapshot_name = String::from(
        Path::new(snapshot_path)
            .components()
            .last()
            .ok_or(anyhow!("could not extract last path component"))?
            .as_os_str()
            .to_str()
            .ok_or(anyhow!("could not convert last path component"))?,
    );
    let mut snapshot_tokens = snapshot_name.split("_");
    let snapshot_timestamp = DateTime::parse_from_rfc3339(
        snapshot_tokens
            .nth(0)
            .ok_or(anyhow!("could not find date part of backup name"))?,
    )?;

    Ok((
        snapshot_timestamp,
        snapshot_tokens.collect::<Vec<&str>>().join("_"),
    ))
}
