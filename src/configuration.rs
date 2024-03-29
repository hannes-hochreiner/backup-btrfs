use crate::custom_duration::CustomDuration;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs::File;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub source_subvolume_path: String,
    pub snapshot_device: String,
    pub snapshot_subvolume_path: String,
    pub snapshot_path: String,
    pub snapshot_suffix: String,
    pub user_local: String,
    pub policy_local: Vec<CustomDuration>,
    pub config_ssh: ConfigurationSsh,
    pub backup_device: String,
    pub backup_subvolume_path: String,
    pub backup_path: String,
    pub policy_remote: Vec<CustomDuration>,
}

#[derive(Debug, Deserialize)]
pub struct ConfigurationSsh {
    pub host: String,
    pub config: Option<String>,
}

impl Configuration {
    pub fn read_from_file(filepath: &str) -> Result<Self> {
        let file = File::open(filepath).context(format!(
            "could not open configuration file \"{}\"",
            filepath
        ))?;

        Ok(serde_json::from_reader(file)?)
    }
}
