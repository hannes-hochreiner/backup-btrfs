use anyhow::{anyhow, Context as _, Result};
use chrono::{DateTime, FixedOffset};

use crate::btrfs::{Btrfs, BtrfsCommands, Subvolume};
use crate::command::Context;
use crate::custom_duration::CustomDuration;
use crate::utils::snapshot::Snapshot;

#[cfg(test)]
mod tests;

pub enum Mode {
    Server,
    Client,
}

pub trait Actions {
    fn create_snapshot(
        &mut self,
        subvolume_path: &str,
        snapshot_path: &str,
        snapshot_suffix: &str,
        context: &Context,
    ) -> Result<()>;
    fn send_snapshot(
        &mut self,
        subvolume_path: &str,
        backup_path: &str,
        context_local: &Context,
        context_remote: &Context,
    ) -> Result<()>;
    fn police_snapshots(&mut self, context: &Context, latest_local_snapshot: &Subvolume, policy: &Vec<CustomDuration>, timestamp: &DateTime<FixedOffset>, suffix: &str) -> Result<()>;
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

impl Actions for ActionsSystem {
    fn create_snapshot(
        &mut self,
        subvolume_path: &str,
        snapshot_path: &str,
        snapshot_suffix: &str,
        context: &Context,
    ) -> Result<()> {
        self.btrfs
            .create_snapshot(subvolume_path, snapshot_path, snapshot_suffix, context)
    }

    fn send_snapshot(
        &mut self,
        subvolume_path: &str,
        backup_path: &str,
        context_local: &Context,
        context_remote: &Context,
    ) -> Result<()> {
        // get local snapshots
        let subvolumes_local = self.btrfs.get_subvolumes(&context_local)?;
        let subvolume_backup =
            crate::utils::get_subvolume_by_path(subvolume_path, &mut subvolumes_local.iter())
                .context("failed to get subvolume by path")?;
        let snapshots_local =
            crate::utils::get_local_snapshots(subvolume_backup, &mut subvolumes_local.iter())
                .context("failed to get local snapshots from subvolumes")?;

        // get remote snapshots
        let subvolumes_remote = self.btrfs.get_subvolumes(&context_remote)?;
        let snapshots_remote = crate::utils::get_remote_snapshots(&mut subvolumes_remote.iter())?;

        // find common parent
        let common_parent = crate::utils::get_common_parent(&snapshots_local, &snapshots_remote)?;

        // match &common_parent {
        //     Some(s) => info!("found common parent snapshot \"{}\"", s.path),
        //     None => info!("no common parent snapshot found"),
        // }

        let latest_local_snapshot = snapshots_local
            .last()
            .ok_or(anyhow!("no snapshot found"))?
            .clone();

        // send remote backup
        self.btrfs.send_snapshot(
            &latest_local_snapshot,
            common_parent,
            &context_local,
            backup_path,
            &context_remote,
        )?;

        Ok(())
    }

    fn police_snapshots(&mut self, context: &Context, latest_local_snapshot: &Subvolume, policy: &Vec<CustomDuration>, timestamp: &DateTime<FixedOffset>, suffix: &str) -> Result<()> {
        // get snapshots
        let subvolumes = self.btrfs.get_subvolumes(&context)?;
        let snapshots: Vec<&dyn Snapshot>;
        let latest_snapshot_uuid;
        let snapshots_local;
        let snapshots_remote;

        match context {
            &Context::Local {..} => {
                snapshots_local = crate::utils::get_local_snapshots(&latest_local_snapshot,&mut subvolumes.iter())?;
                snapshots = snapshots_local.iter().map(|e| e as &dyn Snapshot).collect();
                latest_snapshot_uuid = latest_local_snapshot.uuid;
            },
            &Context::Remote {..} => {
                snapshots_remote = crate::utils::get_remote_snapshots(&mut subvolumes.iter())?;
                latest_snapshot_uuid = snapshots_remote.iter().find(|e| e.received_uuid == latest_local_snapshot.uuid)
                .ok_or(anyhow!(
                    "common snapshot not found"
                ))?.uuid;
                snapshots = snapshots_remote.iter().map(|e| e as &dyn Snapshot).collect();
            },
        }

        // review remote snapshots
        let snapshots_delete = crate::utils::find_backups_to_be_deleted(
            timestamp,
            policy,
            &snapshots.iter().map(|e| *e).collect(),
            suffix,
        )?;

        // delete remote snapshots - filter out the most recent snapshot
        for &snapshot in snapshots_delete
            .iter()
            .filter(|&e| *e.uuid() != latest_snapshot_uuid)
        {
            self.btrfs
                .delete_subvolume(snapshot.path(), &context)
                .context(format!("error deleting snapshot \"{}\"", snapshot.path()))?;
            // info!(
            //     "deleted snapshot \"{}\" on host \"{}\"",
            //     snapshot.path(),
            //     config.config_ssh.remote_host
            // );
        }

        Ok(())
    }
}
