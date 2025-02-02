extern crate backup_btrfs;

use anyhow::{Context as _, Result as AnyhowResult};
use backup_btrfs::actions::{Actions, ActionsSystem};
use backup_btrfs::configuration::Configuration;
use chrono::Utc;
use exec_rs::Context;
use log::{debug, info};
use std::env;

fn main() -> AnyhowResult<()> {
    env_logger::init();

    // read config file
    let config_filename = env::var("BACKUP_BTRFS_CONFIG")
        .context("could not find environment variable BACKUP_BTRFS_CONFIG")?;
    let config = Configuration::read_from_file(&config_filename)?;

    debug!("configuration read from file \"{}\"", config_filename);

    let mut actions: Box<dyn Actions> = Box::new(ActionsSystem::default());

    // create local context
    let context_local = Context::Local {
        user: config.user_local,
    };
    // create remote context
    let context_remote = Context::Remote {
        host: config.config_ssh.host,
        config: config.config_ssh.config,
    };

    // create a new local snapshot
    let new_snapshot_info = actions.create_snapshot(
        &config.source_subvolume_path,
        &config.snapshot_path,
        &config.snapshot_suffix,
        &context_local,
    )?;

    info!("created new snapshot: \"{}\"", new_snapshot_info.fs_path);

    // get device information
    let snapshot_devices = actions.read_link(&config.snapshot_device, &context_local)?;
    let backup_devices = actions.read_link(&config.backup_device, &context_remote)?;

    let local_mount_information = actions.get_mount_information(&context_local)?;
    let remote_mount_information = actions.get_mount_information(&context_remote)?;

    actions.send_snapshot(
        &config.source_subvolume_path,
        &snapshot_devices,
        &config.snapshot_subvolume_path,
        &local_mount_information,
        &new_snapshot_info,
        &context_local,
        &config.backup_subvolume_path,
        &config.backup_path,
        &context_remote,
    )?;

    let timestamp = Utc::now();

    info!("policing local snapshots");

    // police local snapshots
    actions.police_snapshots(
        &config.snapshot_subvolume_path,
        &context_local,
        &new_snapshot_info,
        &config.policy_local,
        &timestamp.into(),
        &config.snapshot_suffix,
        &snapshot_devices,
        &local_mount_information,
    )?;

    info!("policing local snapshots");

    // police remote snapshots
    actions.police_snapshots(
        &config.backup_subvolume_path,
        &context_remote,
        &new_snapshot_info,
        &config.policy_remote,
        &timestamp.into(),
        &config.snapshot_suffix,
        &backup_devices,
        &remote_mount_information,
    )?;

    log::info!("backup completed");

    Ok(())
}
