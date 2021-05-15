use std::{env, fs::File};
mod custom_error;
use custom_error::CustomError;
use chrono::{
    Utc,
};
mod utils;
use utils::{
    create_snapshot,
    get_snapshot_list_local,
    get_local_snapshots,
    find_backups_to_be_deleted,
    delete_snapshot,
    get_snapshot_list_remote,
    get_remote_snapshots,
    get_common_parent,
    send_snapshot,
    delete_remote_snapshot,
};
use serde::Deserialize;
use anyhow::{Result, Context};
mod custom_duration;
use custom_duration::CustomDuration;
use log::{debug, info};

#[derive(Debug, Deserialize)]
pub struct ConfigSsh {
    remote_host: String,
    remote_user: String,
    identity_file_path: String,
}

#[derive(Debug, Deserialize)]
struct Config {
    subvolume_path: String,
    snapshot_path: String,
    snapshot_suffix: String,
    policy_local: Vec<CustomDuration>,
    config_ssh: ConfigSsh,
    backup_path: String,
    policy_remote: Vec<CustomDuration>,
}

fn main() -> Result<()>{
    env_logger::init();

    // read config file
    let config_filename = env::var("BACKUP_LOCAL_RS_CONFIG").context("could not find environment variable BACKUP_LOCAL_RS_CONFIG")?;
    let file = File::open(&config_filename).context(format!("could not open configuration file \"{}\"", config_filename))?;
    let config: Config = serde_json::from_reader(file)?;

    debug!("configuration read from file \"{}\"", config_filename);

    // create a new local snapshot
    create_snapshot(&config.subvolume_path, &config.snapshot_path, &config.snapshot_suffix)?;

    info!("created new snapshot");

    // get local snapshots
    let snapshots_local = get_local_snapshots(&config.subvolume_path, &*get_snapshot_list_local()?)?;

    // get remote snapshots
    let snapshots_remote = get_remote_snapshots(&*get_snapshot_list_remote(&config.config_ssh)?)?;

    // find common parent
    let common_parent = get_common_parent(&snapshots_local, &snapshots_remote)?;

    match &common_parent {
        Some(s) => info!("found common parent snapshot \"{}\"", s.path),
        None => info!("no common parent snapshot found")
    }

    let latest_local_snapshot = snapshots_local.last().ok_or(CustomError::SnapshotError("no snapshot found".into()))?.clone();

    // send remote backup
    send_snapshot(&latest_local_snapshot, &common_parent, &*config.backup_path, &config.config_ssh)?;

    info!("sent snapshot \"{}\" to \"{}\" on host \"{}\"", &latest_local_snapshot.path, config.backup_path, config.config_ssh.remote_host);

    // review local snapshots
    let filter_time = Utc::now();
    let snapshots_delete_local = find_backups_to_be_deleted(&filter_time.into(), &config.policy_local, &snapshots_local.iter().map(|e| e.path.clone()).collect(), &config.snapshot_suffix)?;

    // delete local snapshots - filter out the most recent snapshot
    for snapshot_path in snapshots_delete_local.iter().filter(|&e| *e != latest_local_snapshot.path) {
        delete_snapshot(&snapshot_path).context(format!("error deleting snapshot \"{}\"", &snapshot_path))?;
        info!("deleted local snapshot \"{}\"", snapshot_path);
    }

    // get remote snapshots again
    let snapshots_remote = get_remote_snapshots(&*get_snapshot_list_remote(&config.config_ssh)?)?;

    // review remote snapshots
    let snapshots_delete_remote = find_backups_to_be_deleted(&filter_time.into(), &config.policy_remote, &snapshots_remote.iter().map(|e| e.path.clone()).collect(), &config.snapshot_suffix)?;

    // delete remote snapshots - filter out the most recent snapshot
    let snapshot_remote_common = snapshots_remote.iter().find(|&e| e.received_uuid == latest_local_snapshot.uuid).ok_or(CustomError::SnapshotError("common snapshot not found".into()))?;
    
    for snapshot_path in snapshots_delete_remote.iter().filter(|&e| *e != snapshot_remote_common.path) {
        delete_remote_snapshot(&snapshot_path, &config.config_ssh).context(format!("error deleting snapshot \"{}\"", &snapshot_path))?;
        info!("deleted snapshot \"{}\" on host \"{}\"", snapshot_path, config.config_ssh.remote_host);
    }
    
    Ok(())
}
