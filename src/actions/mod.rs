use std::collections::HashMap;

use anyhow::{anyhow, Context as _, Result};
use chrono::{DateTime, FixedOffset};
use uuid::Uuid;

use crate::btrfs::{Btrfs, BtrfsCommands};
use crate::command::Context;
use crate::custom_duration::CustomDuration;
use crate::objects::subvolume::Subvolume;
use crate::objects::subvolume_info::SubvolumeInfo;

#[cfg(test)]
mod tests;

pub trait Actions {
    fn create_snapshot(
        &mut self,
        source_subvolume_path: &str,
        snapshot_path: &str,
        snapshot_suffix: &str,
        context: &Context,
    ) -> Result<SubvolumeInfo>;
    fn send_snapshot(
        &mut self,
        parent_subvolume: &SubvolumeInfo,
        local_subvolume_path: &str,
        snapshot_path: &str,
        context_local: &Context,
        remote_subvolume_path: &str,
        context_remote: &Context,
    ) -> Result<()>;
    fn police_snapshots(
        &mut self,
        subvolume_path: &str,
        context: &Context,
        latest_local_snapshot: &Subvolume,
        policy: &Vec<CustomDuration>,
        timestamp: &DateTime<FixedOffset>,
        suffix: &str,
    ) -> Result<()>;
}

pub struct ActionsSystem {
    btrfs: Box<dyn BtrfsCommands>,
}

impl Default for ActionsSystem {
    fn default() -> Self {
        ActionsSystem {
            btrfs: Box::new(Btrfs::default()),
        }
    }
}

impl ActionsSystem {
    /// Get the newest subvolume that was used received on the remote host and is still available locally
    ///
    /// * `subvolumes_local` - list of local subvolumes
    /// * `snapshots_remote` - list of remote subvolumes
    ///
    pub fn get_common_parent<'a>(
        subvolumes_local: &'a Vec<Subvolume>,
        subvolumes_remote: &'a Vec<Subvolume>,
    ) -> Result<Option<&'a Subvolume>> {
        let uuids: HashMap<&Uuid, &Subvolume> =
            subvolumes_local.iter().map(|e| (&e.uuid, e)).collect();

        let mut common_subvolumes: Vec<&Subvolume> = subvolumes_remote
            .iter()
            .filter(|sv| match sv.received_uuid {
                Some(received_uuid) => match uuids.get(&received_uuid) {
                    Some(_) => true,
                    _ => false,
                },
                None => false,
            })
            .collect();

        common_subvolumes.sort_by_key(|&sv| &sv.btrfs_path);

        Ok(common_subvolumes.last().copied())
    }
}

impl Actions for ActionsSystem {
    fn create_snapshot(
        &mut self,
        subvolume_path: &str,
        snapshot_path: &str,
        snapshot_suffix: &str,
        context: &Context,
    ) -> Result<SubvolumeInfo> {
        let snapshot_path =
            self.btrfs
                .create_snapshot(subvolume_path, snapshot_path, snapshot_suffix, context)?;
        // get snapshot/subvolume information
        self.btrfs.get_subvolume_info(&snapshot_path, context)
    }

    fn send_snapshot(
        &mut self,
        parent_subvolume: &SubvolumeInfo,
        local_subvolume_path: &str,
        backup_path: &str,
        context_local: &Context,
        remote_subvolume_path: &str,
        context_remote: &Context,
    ) -> Result<()> {
        // get local snapshots, which are children of the supplied parent subvolume
        let subvolumes_local = self
            .btrfs
            .get_subvolumes(local_subvolume_path, &context_local)?
            .iter()
            .filter(|sv| match sv.parent_uuid {
                Some(sv_uuid) => sv_uuid == parent_subvolume.uuid,
                None => false,
            })
            .cloned()
            .collect();

        // get remote snapshots
        let subvolumes_remote = self
            .btrfs
            .get_subvolumes(remote_subvolume_path, &context_remote)?;

        // find common parent
        let common_parent =
            ActionsSystem::get_common_parent(&subvolumes_local, &subvolumes_remote)?;

        match &common_parent {
            Some(s) => log::info!("found common parent snapshot \"{}\"", s.btrfs_path),
            None => log::info!("no common parent snapshot found"),
        }

        // send remote backup
        self.btrfs.send_snapshot(
            parent_subvolume,
            common_parent,
            &context_local,
            backup_path,
            &context_remote,
        )?;

        Ok(())
    }

    fn police_snapshots(
        &mut self,
        subvolume_path: &str,
        context: &Context,
        latest_local_snapshot: &Subvolume,
        policy: &Vec<CustomDuration>,
        timestamp: &DateTime<FixedOffset>,
        suffix: &str,
    ) -> Result<()> {
        todo!()
    }

    // fn police_snapshots(
    //     &mut self,
    //     subvolume_path: &str,
    //     context: &Context,
    //     latest_local_snapshot: &Subvolume,
    //     policy: &Vec<CustomDuration>,
    //     timestamp: &DateTime<FixedOffset>,
    //     suffix: &str,
    // ) -> Result<()> {
    //     // get snapshots
    //     let subvolumes = self.btrfs.get_subvolumes(subvolume_path, &context)?;
    //     let snapshots: Vec<&dyn Snapshot>;
    //     let latest_snapshot_uuid;
    //     let snapshots_local;
    //     let snapshots_remote;

    //     match context {
    //         &Context::Local { .. } => {
    //             snapshots_local = crate::utils::get_local_snapshots(
    //                 &latest_local_snapshot,
    //                 &mut subvolumes.iter(),
    //             )?;
    //             snapshots = snapshots_local.iter().map(|e| e as &dyn Snapshot).collect();
    //             latest_snapshot_uuid = latest_local_snapshot.uuid;
    //         }
    //         &Context::Remote { .. } => {
    //             snapshots_remote = crate::utils::get_remote_snapshots(&mut subvolumes.iter())?;
    //             latest_snapshot_uuid = snapshots_remote
    //                 .iter()
    //                 .find(|e| e.received_uuid == latest_local_snapshot.uuid)
    //                 .ok_or(anyhow!("common snapshot not found"))?
    //                 .uuid;
    //             snapshots = snapshots_remote
    //                 .iter()
    //                 .map(|e| e as &dyn Snapshot)
    //                 .collect();
    //         }
    //     }

    //     // review remote snapshots
    //     let snapshots_delete = crate::utils::find_backups_to_be_deleted(
    //         timestamp,
    //         policy,
    //         &snapshots.iter().map(|e| *e).collect(),
    //         suffix,
    //     )?;

    //     // delete remote snapshots - filter out the most recent snapshot
    //     for &snapshot in snapshots_delete
    //         .iter()
    //         .filter(|&e| *e.uuid() != latest_snapshot_uuid)
    //     {
    //         self.btrfs
    //             .delete_subvolume(snapshot.path(), &context)
    //             .context(format!("error deleting snapshot \"{}\"", snapshot.path()))?;
    //         // info!(
    //         //     "deleted snapshot \"{}\" on host \"{}\"",
    //         //     snapshot.path(),
    //         //     config.config_ssh.remote_host
    //         // );
    //     }

    //     Ok(())
    // }
}
