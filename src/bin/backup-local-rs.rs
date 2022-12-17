extern crate backup_local_rs;

use anyhow::{Context as _, Result as AnyhowResult};
use backup_local_rs::actions::{Actions, ActionsSystem};
use backup_local_rs::configuration::Configuration;
use chrono::Utc;
use exec_rs::Context;
use log::{debug, info};
use std::env;

fn main() -> AnyhowResult<()> {
    env_logger::init();

    let mut com = std::process::Command::new("ssh");

    com.arg("-i")
        .arg("/home/hannes/.ssh/local_backup_id")
        .arg("local_backup@charon")
        .arg("sudo")
        .args(&[
            "btrfs",
            "subvolume",
            "delete",
            "/data/backups/snapshots/2022-12-15T18:28:13Z_inf_home",
        ]);
    com.spawn().expect("failed");

    // read config file
    let config_filename = env::var("BACKUP_LOCAL_RS_CONFIG")
        .context("could not find environment variable BACKUP_LOCAL_RS_CONFIG")?;
    let config = Configuration::read_from_file(&config_filename)?;

    debug!("configuration read from file \"{}\"", config_filename);

    let mut actions: Box<dyn Actions> = Box::new(ActionsSystem::default());

    // create local context
    let context_local = Context::Local {
        user: config.user_local,
    };
    // create remote context
    let context_remote = Context::Remote {
        user: config.config_ssh.remote_user,
        host: config.config_ssh.remote_host.clone(),
        identity: config.config_ssh.identity_file_path,
    };

    // create a new local snapshot
    let new_snapshot_info = actions.create_snapshot(
        &config.source_subvolume_path,
        &config.snapshot_path,
        &config.snapshot_suffix,
        &context_local,
    )?;

    info!("created new snapshot: \"{}\"", new_snapshot_info.fs_path);

    let local_mount_information = actions.get_mount_information(&context_local)?;
    let remote_mount_information = actions.get_mount_information(&context_remote)?;

    actions.send_snapshot(
        &config.source_subvolume_path,
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
        &remote_mount_information,
    )?;

    log::info!("backup completed");

    Ok(())
}
