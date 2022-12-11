extern crate backup_local_rs;

use anyhow::{Context, Result};
use chrono::Utc;
use log::{debug, info};
use std::env;

use backup_local_rs::actions::{Actions, ActionsSystem};
use backup_local_rs::btrfs::{Btrfs, BtrfsCommands};
use backup_local_rs::command;
use backup_local_rs::configuration::Configuration;
use backup_local_rs::custom_error::CustomError;

fn main() -> Result<()> {
    env_logger::init();

    // read config file
    let config_filename = env::var("BACKUP_LOCAL_RS_CONFIG")
        .context("could not find environment variable BACKUP_LOCAL_RS_CONFIG")?;
    let config = Configuration::read_from_file(&config_filename)?;
    debug!("configuration read from file \"{}\"", config_filename);

    let mut actions: Box<dyn Actions> = Box::new(ActionsSystem::default());

    let context_local = command::Context::Local {
        user: config.user_local,
    };
    let context_remote = command::Context::Remote {
        user: config.config_ssh.remote_user,
        host: config.config_ssh.remote_host.clone(),
        identity: config.config_ssh.identity_file_path,
    };
    // create a new local snapshot
    let new_snapshot_path = actions.create_snapshot(
        &config.source_subvolume_path,
        &config.snapshot_path,
        &config.snapshot_suffix,
        &context_local,
    )?;

    // info!("created new snapshot: \"{}\"", new_snapshot_path);

    // actions.send_snapshot(
    //     &config.snapshot_subvolume_path,
    //     &config.backup_path,
    //     &context_local,
    //     &config.backup_subvolume_path,
    //     &context_remote,
    // )?;

    // TODO: get local snapshots
    //  TODO: get source subvolume uuid
    //  TODO: get destination subvolume snapshots
    //  TODO: filter destination subvolume snapshots by source subvolume uuid

    // // get local snapshots
    // let subvolumes_local = btrfs.get_subvolumes(&config.dst_subvolume_path, &context_local)?;
    // let subvolume_backup =
    //     utils::get_subvolume_by_path(&config.src_subvolume_path, &mut subvolumes_local.iter())?;
    // let snapshots_local =
    //     utils::get_local_snapshots(subvolume_backup, &mut subvolumes_local.iter())?;

    // // get remote snapshots
    // let subvolumes_remote = btrfs.get_subvolumes(&config.backup_subvolume_path, &context_remote)?;
    // let snapshots_remote = utils::get_remote_snapshots(&mut subvolumes_remote.iter())?;

    // // find common parent
    // let common_parent = get_common_parent(&snapshots_local, &snapshots_remote)?;

    // match &common_parent {
    //     Some(s) => info!("found common parent snapshot \"{}\"", s.path),
    //     None => info!("no common parent snapshot found"),
    // }

    // let latest_local_snapshot = snapshots_local
    //     .last()
    //     .ok_or(CustomError::SnapshotError("no snapshot found".into()))?
    //     .clone();

    // // send remote backup
    // btrfs.send_snapshot(
    //     &latest_local_snapshot,
    //     common_parent,
    //     &context_local,
    //     &*config.backup_path,
    //     &context_remote,
    // )?;

    // info!(
    //     "sent snapshot \"{}\" to \"{}\" on host \"{}\"",
    //     &latest_local_snapshot.path, config.backup_path, config.config_ssh.remote_host
    // );

    let timestamp = Utc::now();

    // police local snapshots
    // actions.police_snapshots(
    //     &config.snapshot_subvolume_path,
    //     &context_local,
    //     latest_local_snapshot,
    //     &config.policy_local,
    //     &timestamp.into(),
    //     &config.snapshot_suffix,
    // )?;

    // police remote snapshots
    // actions.police_snapshots(
    //     &config.backup_subvolume_path,
    //     &context_remote,
    //     latest_local_snapshot,
    //     &config.policy_remote,
    //     &timestamp.into(),
    //     &config.snapshot_suffix,
    // )?;

    // // review local snapshots
    // let filter_time = Utc::now();
    // let snapshots_delete_local = find_backups_to_be_deleted(
    //     &filter_time.into(),
    //     &config.policy_local,
    //     &snapshots_local.iter().map(|e| e as &dyn Snapshot).collect(),
    //     &config.snapshot_suffix,
    // )?;

    // // delete local snapshots - filter out the most recent snapshot
    // for &snapshot in snapshots_delete_local
    //     .iter()
    //     .filter(|&e| e.path() != latest_local_snapshot.path)
    // {
    //     log::debug!("local snapshot to be deleted: {}", snapshot.path());

    //     btrfs
    //         .delete_subvolume(snapshot.path(), &context_local)
    //         .context(format!("error deleting snapshot \"{}\"", snapshot.path()))?;
    //     info!("deleted local snapshot \"{}\"", snapshot.path());
    // }

    // // get remote snapshots again
    // let subvolumes_remote = btrfs.get_subvolumes(&config.backup_subvolume_path, &context_remote)?;
    // let snapshots_remote = utils::get_remote_snapshots(&mut subvolumes_remote.iter())?;

    // // review remote snapshots
    // let snapshots_delete_remote = find_backups_to_be_deleted(
    //     &filter_time.into(),
    //     &config.policy_remote,
    //     &snapshots_remote
    //         .iter()
    //         .map(|e| e as &dyn Snapshot)
    //         .collect(),
    //     &config.snapshot_suffix,
    // )?;

    // // delete remote snapshots - filter out the most recent snapshot
    // let snapshot_remote_common = snapshots_remote
    //     .iter()
    //     .find(|&e| e.received_uuid == latest_local_snapshot.uuid)
    //     .ok_or(CustomError::SnapshotError(
    //         "common snapshot not found".into(),
    //     ))?;

    // for &snapshot in snapshots_delete_remote
    //     .iter()
    //     .filter(|&e| e.path() != snapshot_remote_common.path)
    // {
    //     btrfs
    //         .delete_subvolume(snapshot.path(), &context_remote)
    //         .context(format!("error deleting snapshot \"{}\"", snapshot.path()))?;
    //     info!(
    //         "deleted snapshot \"{}\" on host \"{}\"",
    //         snapshot.path(),
    //         config.config_ssh.remote_host
    //     );
    // }

    Ok(())
}
