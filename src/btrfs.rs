use std::{path::{Path, PathBuf}, str::FromStr};
use crate::command::{Command, CommandSystem, Context};
use anyhow::{Result, Error, anyhow};
use chrono::{SecondsFormat, Utc};
use uuid::Uuid;
#[cfg(test)]
mod tests;

pub struct Btrfs {
    command: Box<dyn Command>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Subvolume {
    pub path: String,
    pub uuid: Uuid,
    pub parent_uuid: Option<Uuid>,
    pub received_uuid: Option<Uuid>,
}

impl Default for Btrfs {
    fn default() -> Self {
        Btrfs {
            command: Box::new(CommandSystem {}),
        }
    }
}

impl Btrfs {
    pub fn get_local_subvolumes(&mut self, user: &str) -> Result<Vec<Subvolume>> {
        self.get_subvolumes_with_context(&Context::Local {user: user.into()})
    }
    
    pub fn get_remote_subvolumes(&mut self, host: &str, user: &str, identity: &str) -> Result<Vec<Subvolume>> {
        self.get_subvolumes_with_context(&Context::Remote {
            host: host.into(),
            user: user.into(),
            identity: identity.into(),
        })
    }

    /// Create a snapshot locally
    ///
    /// The new snapshot will be created at the path `<snapshot_path>/<timestamp in rfc3339 format (UTC)>_<suffix>`.
    /// This function executes the command `sudo -nu <user> bash -c "sudo btrfs subvolume snapshot -r \"<subvolume_path>\" \"<snapshot_path>/<timestamp in rfc3339 format (UTC)>_<suffix>\""`.
    ///
    /// * `subvolume_path` - path to the subvolume from which to create the snapshot
    /// * `snapshot_path` - base path at which the snapshot should be created
    /// * `snapshot_suffix` - suffix of the subvolume
    /// * `user` - local user executing the snapshot creation
    ///
    pub fn create_local_snapshot(&mut self, subvolume_path: &String, snapshot_path: &String, snapshot_suffix: &String, user: &str) -> Result<()> {
        let snapshot_path_extension = format!("{}_{}", Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true), snapshot_suffix);
        let mut snapshot_path = PathBuf::from(&*snapshot_path);
    
        snapshot_path.push(snapshot_path_extension);

        let snapshot_path = snapshot_path.as_path().to_str().ok_or(anyhow!("could not construct snapshot_path"))?;
    
        self.command.run(&*format!("btrfs subvolume snapshot -r \"{}\" \"{}\"", subvolume_path, snapshot_path), &Context::Local {user: user.into()})?;
    
        Ok(())
    }

    /// Delete a snapshot
    ///
    /// Executes `btrfs subvolume delete <subvolume_path>`.
    ///
    /// * `subvolume_path` - absolute path of the snapshot to be deleted
    ///
    pub fn delete_local_subvolume(&mut self, subvolume_path: &String, user: &str) -> Result<()> {
        self.delete_subvolume(subvolume_path, &Context::Local{user:user.into()})
    }

    pub fn delete_remote_subvolume(&mut self, subvolume_path: &String, user: &str, host: &str, identity: &str) -> Result<()> {
        self.delete_subvolume(subvolume_path, &Context::Remote {
            user: user.into(),
            host: host.into(),
            identity: identity.into(),
        })
    }

    /// Delete a snapshot
    ///
    /// Executes `btrfs subvolume delete <subvolume_path>`.
    /// As a precaution, the subvolumes "home", "/home", "root", and "/" cannot be deleted.
    ///
    /// * `subvolume_path` - absolute path of the snapshot to be deleted
    ///
    fn delete_subvolume(&mut self, subvolume: &String, context: &Context) -> Result<()> {
        let subvolume_path = Path::new(subvolume).canonicalize()?;
        let subvolume = subvolume_path.as_path().to_str().ok_or(anyhow!("cannot canonicalize subvolume path"))?;

        if vec!["home", "/home", "root", "/"].contains(&subvolume) {
            return Err(anyhow!("subvolume cannot be deleted as its name is on the restricted names list (home, /home, /, root)"));
        }

        self.command.run(&format!("sudo btrfs subvolume delete \"{}\"", subvolume), context).map(|_| ())
    }

    fn get_subvolumes_with_context(&mut self, context: &Context) -> Result<Vec<Subvolume>> {
        let output = self.command.run("sudo btrfs subvolume list -tupqR --sort=rootid /", context)?;

        self.get_subvolumes(&output)
    }

    fn get_subvolumes(&self, input: &String) -> Result<Vec<Subvolume>> {
        let mut subvolumes: Vec<Subvolume> = Vec::new();
        let mut lines = input.split("\n");
    
        if lines.next().ok_or(Error::msg("could not find header line"))?
            .split_ascii_whitespace().collect::<Vec<&str>>() != vec!["ID", "gen", "parent", "top", "level", "parent_uuid", "received_uuid", "uuid", "path"] {
            return Err(Error::msg("unexpected header line").into());
        }
    
        for line in lines.skip(1).into_iter() {
            let tokens: Vec<&str> = line.split_ascii_whitespace().collect();
    
            if tokens.len() != 8 {
                continue;
            }
    
            subvolumes.push(Subvolume {
                path: format!("/{}", tokens[7]),
                uuid: Uuid::from_str(tokens[6])?,
                parent_uuid: match Uuid::from_str(tokens[4]) {
                    Ok(pu) => Some(pu),
                    Err(_) => None,
                },
                received_uuid: match Uuid::from_str(tokens[5]) {
                    Ok(ru) => Some(ru),
                    Err(_) => None,
                }
            });
        }

        Ok(subvolumes)
    }
}
