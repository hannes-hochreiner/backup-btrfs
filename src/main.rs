use std::{
    env, 
    fs::File,
};
mod custom_error;
mod utils;
use utils::{
    create_snapshot,
    get_snapshot_list_local,
    get_snapshots,
};
use serde::Deserialize;
use anyhow::{Result, Context};

#[derive(Debug, Deserialize)]
struct Config {
    subvolume_path: String,
    snapshot_path: String,
    snapshot_suffix: String,
}

fn main() -> Result<()>{
    // read config file
    let config_filename = env::var("BACKUP_LOCAL_RS_CONFIG").context("could not find environment variable BACKUP_LOCAL_RS_CONFIG")?;
    let file = File::open(&config_filename).context(format!("could not open configuration file \"{}\"", config_filename))?;
    let config: Config = serde_json::from_reader(file)?;

    // create a new local snapshot
    create_snapshot(&config.subvolume_path, &config.snapshot_path, &config.snapshot_suffix)?;

    // get local snapshots
    let snapshots_local = get_snapshots(&config.subvolume_path, &*get_snapshot_list_local()?)?;

    // get remote snapshots

    // find common parent

    // send remote backup

    // review local snapshots
    
    // review remote snapshots
    Ok(())
}
