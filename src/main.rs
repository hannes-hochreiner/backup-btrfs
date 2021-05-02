use std::{env, error::Error, fs::File, path::{PathBuf}};
use std::env::Vars;
use std::process::Command;
mod custom_error;
use chrono::{Duration, SecondsFormat, Utc};
use custom_error::CustomError;
mod utils;
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
    let snapshot_path_extension = format!("{}_{}", Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true), config.snapshot_suffix);
    let mut snapshot_path = PathBuf::from(&*config.snapshot_path);

    if !snapshot_path.is_dir() {
        return Err(CustomError::ConfigurationError("snapshot_path must be a directory".into()).into());
    }

    if !snapshot_path.is_absolute() {
        return Err(CustomError::ConfigurationError("snapshot_path must be an absolute path".into()).into());
    }

    snapshot_path.push(snapshot_path_extension);

    Command::new("btrfs")
            .arg("subvolume")
            .arg("snapshot")
            .arg("-r")
            .arg(config.subvolume_path)
            .arg(snapshot_path)
            .output()?;

    // get local snapshots

    // get remote snapshots

    // find common parent

    // send remote backup

    // review local snapshots
    
    // review remote snapshots
    Ok(())
}
