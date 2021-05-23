use std::env;
mod custom_error;
use custom_error::CustomError;
use chrono::{
    Utc,
};
mod utils;
use utils::{
    find_backups_to_be_deleted,
    get_common_parent,
};
use anyhow::{Result, Context};
mod custom_duration;
use custom_duration::CustomDuration;
use log::{debug, info};
mod btrfs;
use btrfs::Btrfs;
mod command;
mod configuration;
use configuration::Configuration;

fn main() -> Result<()>{
    env_logger::init();

    // read config file
    let config_filename = env::var("BACKUP_LOCAL_RS_CONFIG").context("could not find environment variable BACKUP_LOCAL_RS_CONFIG")?;
    let config = Configuration::read_from_file(&config_filename)?;
    debug!("configuration read from file \"{}\"", config_filename);

    let mut btrfs = Btrfs::default();
    // create a new local snapshot
    btrfs.create_local_snapshot(&config.subvolume_path, &config.snapshot_path, &config.snapshot_suffix, &config.user_local)?;

    info!("created new snapshot");

    // get local snapshots
    let subvolumes_local = btrfs.get_local_subvolumes(&config.user_local)?;
    let subvolume_backup = utils::get_subvolume_by_path(&config.subvolume_path, &mut subvolumes_local.iter())?;
    let snapshots_local = utils::get_local_snapshots(subvolume_backup, &mut subvolumes_local.iter())?;

    // get remote snapshots
    let subvolumes_remote = btrfs.get_remote_subvolumes(&config.config_ssh.remote_host, &config.config_ssh.remote_user, &config.config_ssh.identity_file_path)?;
    let snapshots_remote = utils::get_remote_snapshots(&mut subvolumes_remote.iter())?;

    // find common parent
    let common_parent = get_common_parent(&snapshots_local, &snapshots_remote)?;

    match &common_parent {
        Some(s) => info!("found common parent snapshot \"{}\"", s.path),
        None => info!("no common parent snapshot found")
    }

    let latest_local_snapshot = snapshots_local.last().ok_or(CustomError::SnapshotError("no snapshot found".into()))?.clone();

    // send remote backup
    // send_snapshot(&latest_local_snapshot, &common_parent, &*config.backup_path, &config.config_ssh)?;

    info!("sent snapshot \"{}\" to \"{}\" on host \"{}\"", &latest_local_snapshot.path, config.backup_path, config.config_ssh.remote_host);

    // review local snapshots
    let filter_time = Utc::now();
    let snapshots_delete_local = find_backups_to_be_deleted(&filter_time.into(), &config.policy_local, &snapshots_local.iter().map(|e| e.path.clone()).collect(), &config.snapshot_suffix)?;

    // delete local snapshots - filter out the most recent snapshot
    for snapshot_path in snapshots_delete_local.iter().filter(|&e| *e != latest_local_snapshot.path) {
        btrfs.delete_local_subvolume(&snapshot_path, &config.user_local).context(format!("error deleting snapshot \"{}\"", &snapshot_path))?;
        info!("deleted local snapshot \"{}\"", snapshot_path);
    }

    // get remote snapshots again
    let subvolumes_remote = btrfs.get_remote_subvolumes(&config.config_ssh.remote_host, &config.config_ssh.remote_user, &config.config_ssh.identity_file_path)?;
    let snapshots_remote = utils::get_remote_snapshots(&mut subvolumes_remote.iter())?;

    // review remote snapshots
    let snapshots_delete_remote = find_backups_to_be_deleted(&filter_time.into(), &config.policy_remote, &snapshots_remote.iter().map(|e| e.path.clone()).collect(), &config.snapshot_suffix)?;

    // delete remote snapshots - filter out the most recent snapshot
    let snapshot_remote_common = snapshots_remote.iter().find(|&e| e.received_uuid == latest_local_snapshot.uuid).ok_or(CustomError::SnapshotError("common snapshot not found".into()))?;
    
    for snapshot_path in snapshots_delete_remote.iter().filter(|&e| *e != snapshot_remote_common.path) {
        btrfs.delete_remote_subvolume(&snapshot_path, &config.config_ssh.remote_user, &config.config_ssh.remote_host, &config.config_ssh.identity_file_path).context(format!("error deleting snapshot \"{}\"", &snapshot_path))?;
        info!("deleted snapshot \"{}\" on host \"{}\"", snapshot_path, config.config_ssh.remote_host);
    }
    
    Ok(())
}
