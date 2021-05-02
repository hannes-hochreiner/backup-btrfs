use std::{env, fs::File};
mod custom_error;
mod utils;
use chrono::{
    Utc,
};
use utils::{
    create_snapshot,
    get_snapshot_list_local,
    get_local_snapshots,
    find_backups_to_be_deleted,
    delete_snapshot,
    get_snapshot_list_remote,
    get_remote_snapshots,
    get_common_parent,
};
use serde::Deserialize;
use anyhow::{Result, Context};
mod custom_duration;
use custom_duration::CustomDuration;

#[derive(Debug, Deserialize)]
struct Config {
    subvolume_path: String,
    snapshot_path: String,
    snapshot_suffix: String,
    policy_local: Vec<CustomDuration>,
    remote_host: String,
    remote_user: String,
    identity_file_path: String,
}

fn main() -> Result<()>{
    // read config file
    let config_filename = env::var("BACKUP_LOCAL_RS_CONFIG").context("could not find environment variable BACKUP_LOCAL_RS_CONFIG")?;
    let file = File::open(&config_filename).context(format!("could not open configuration file \"{}\"", config_filename))?;
    let config: Config = serde_json::from_reader(file)?;

    // create a new local snapshot
    create_snapshot(&config.subvolume_path, &config.snapshot_path, &config.snapshot_suffix)?;

    // get local snapshots
    let snapshots_local = get_local_snapshots(&config.subvolume_path, &*get_snapshot_list_local()?)?;

    // get remote snapshots
    let snapshots_remote = get_remote_snapshots(&*get_snapshot_list_remote(&*config.remote_host, &*config.remote_user, &*config.identity_file_path)?)?;

    // find common parent
    let common_parent = get_common_parent(&snapshots_local, &snapshots_remote)?;

    // send remote backup

    // review local snapshots - filter out the most recent snapshot from the deletion list
    for snapshot_path in find_backups_to_be_deleted(&Utc::now().into(), &config.policy_local, &snapshots_local.iter().map(|e| e.path.clone()).collect())? {
        delete_snapshot(&snapshot_path).context(format!("error deleting snapshot \"{}\"", &snapshot_path))?;
    }

    // review remote snapshots - filter out the most recent snapshot from the deletion list
    Ok(())
}
