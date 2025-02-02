use crate::backup_error::BackupError;
use crate::objects::MountInformation;
use exec_rs::{Context, Exec};

pub trait CommandGetMountInformation {
    /// Get the mount information of all btrfs mounts.
    ///
    /// * `exec` - command executor
    /// * `context` - context in which to run the command
    ///
    /// References:
    /// * https://mpdesouza.com/blog/btrfs-differentiating-bind-mounts-on-subvolumes/
    /// * https://www.kernel.org/doc/Documentation/filesystems/proc.txt
    fn get_mount_information(
        &mut self,
        context: &Context,
    ) -> Result<Vec<MountInformation>, BackupError>;
}

impl<T: Exec> CommandGetMountInformation for super::Commander<T> {
    fn get_mount_information(
        &mut self,
        context: &Context,
    ) -> Result<Vec<MountInformation>, BackupError> {
        let command_output = self.exec.exec(
            "findmnt",
            &[
                "-lnvt",
                "btrfs",
                "-o",
                "FSROOT,TARGET,FSTYPE,SOURCE,OPTIONS",
            ],
            Some(context),
        )?;

        command_output
            .lines()
            .filter(|&l| !l.is_empty())
            .map(|l| {
                let mut iter = l.split_ascii_whitespace();

                Ok(MountInformation {
                    root: iter
                        .next()
                        .ok_or(BackupError::MountParsing("could not find root".to_string()))?
                        .to_string(),
                    mount_point: iter
                        .next()
                        .ok_or(BackupError::MountParsing(
                            "could not find mount point".to_string(),
                        ))?
                        .to_string(),
                    fs_type: iter
                        .next()
                        .ok_or(BackupError::MountParsing(
                            "could not find fs type".to_string(),
                        ))?
                        .to_string(),
                    device: iter
                        .next()
                        .ok_or(BackupError::MountParsing(
                            "could not find device".to_string(),
                        ))?
                        .to_string(),
                    properties: iter
                        .next()
                        .ok_or(BackupError::MountParsing(
                            "could not find properties".to_string(),
                        ))?
                        .split(",")
                        .map(|s| match s.find("=") {
                            Some(equal_idx) => (
                                s[..equal_idx].to_string(),
                                Some(s[equal_idx + 1..].to_string()),
                            ),
                            None => (s.to_string(), None),
                        })
                        .collect(),
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::Commander;
    use exec_rs::{Context, MockExec};
    use std::collections::HashMap;

    #[test]
    fn get_mount_information_btrfs_1() {
        let mut mock = MockExec::new();

        mock.expect_exec().once().withf(|_,_,_| true).returning(|_,_,_| Ok(String::from(r#"/root           /          btrfs  /dev/dm-0 rw,relatime,ssd,space_cache=v2,subvolid=256,subvol=/root
/root/nix/store /nix/store btrfs  /dev/dm-0 ro,relatime,ssd,space_cache=v2,subvolid=256,subvol=/root
/swap           /swap      btrfs  /dev/dm-0 rw,relatime,ssd,space_cache=v2,subvolid=259,subvol=/swap
"#)));

        let mut commands = Commander::new_with_exec(mock);

        assert_eq!(
            commands
                .get_mount_information(&Context::Local {
                    user: String::from("test"),
                },)
                .unwrap(),
            vec![
                MountInformation {
                    device: String::from("/dev/dm-0"),
                    fs_type: String::from("btrfs"),
                    mount_point: String::from("/"),
                    root: String::from("/root"),
                    properties: vec![
                        (String::from("rw"), None),
                        (String::from("relatime"), None),
                        (String::from("ssd"), None),
                        (String::from("space_cache"), Some(String::from("v2"))),
                        (String::from("subvolid"), Some(String::from("256"))),
                        (String::from("subvol"), Some(String::from("/root")))
                    ]
                    .iter()
                    .cloned()
                    .collect::<HashMap<String, Option<String>>>()
                },
                MountInformation {
                    device: String::from("/dev/dm-0"),
                    fs_type: String::from("btrfs"),
                    mount_point: String::from("/nix/store"),
                    root: String::from("/root/nix/store"),
                    properties: vec![
                        (String::from("ro"), None),
                        (String::from("relatime"), None),
                        (String::from("ssd"), None),
                        (String::from("space_cache"), Some(String::from("v2"))),
                        (String::from("subvolid"), Some(String::from("256"))),
                        (String::from("subvol"), Some(String::from("/root")))
                    ]
                    .iter()
                    .cloned()
                    .collect::<HashMap<String, Option<String>>>()
                },
                MountInformation {
                    device: String::from("/dev/dm-0"),
                    fs_type: String::from("btrfs"),
                    mount_point: String::from("/swap"),
                    root: String::from("/swap"),
                    properties: vec![
                        (String::from("rw"), None),
                        (String::from("relatime"), None),
                        (String::from("ssd"), None),
                        (String::from("space_cache"), Some(String::from("v2"))),
                        (String::from("subvolid"), Some(String::from("259"))),
                        (String::from("subvol"), Some(String::from("/swap")))
                    ]
                    .iter()
                    .cloned()
                    .collect::<HashMap<String, Option<String>>>()
                },
            ]
        );
    }

    #[test]
    fn get_mount_information_btrfs_2() {
        let context = Context::Local {
            user: String::from("test"),
        };
        let mut mock = MockExec::new();

        mock.expect_exec().once().withf(|_,_,_| true).returning(|_,_,_| Ok(String::from(r#"/ /data btrfs  /dev/mapper/data  rw,relatime,space_cache=v2,subvolid=5,subvol=/"#)));

        let mut commands = Commander::new_with_exec(mock);

        assert_eq!(
            commands.get_mount_information(&context).unwrap(),
            vec![MountInformation {
                device: String::from("/dev/mapper/data"),
                fs_type: String::from("btrfs"),
                mount_point: String::from("/data"),
                root: String::from("/"),
                properties: vec![
                    (String::from("rw"), None),
                    (String::from("relatime"), None),
                    (String::from("space_cache"), Some(String::from("v2"))),
                    (String::from("subvolid"), Some(String::from("5"))),
                    (String::from("subvol"), Some(String::from("/")))
                ]
                .iter()
                .cloned()
                .collect::<HashMap<String, Option<String>>>()
            },]
        );
    }

    #[test]
    fn get_mount_information_any_1() {
        let context = Context::Local {
            user: String::from("test"),
        };
        let mut mock = MockExec::new();

        mock.expect_exec().once().withf(|_,_,_| true).returning(|_,_,_| Ok(String::from(r#"/ /data btrfs  /dev/mapper/data  rw,relatime,space_cache=v2,subvolid=5,subvol=/
/ /raid btrfs  /dev/mapper/raid0 rw,relatime,space_cache=v2,subvolid=5,subvol=/"#)));

        let mut commands = Commander::new_with_exec(mock);

        commands.get_mount_information(&context).unwrap();
    }

    #[test]
    fn get_mount_information_any_2() {
        let context = Context::Local {
            user: String::from("test"),
        };
        let mut mock = MockExec::new();

        mock.expect_exec().once().withf(|_,_,_| true).returning(|_,_,_| Ok(String::from(r#"/ /data btrfs  /dev/mapper/data  rw,relatime,space_cache=v2,subvolid=5,subvol=/
/ /raid btrfs  /dev/mapper/raid0 rw,relatime,space_cache=v2,subvolid=5,subvol=/"#)));

        let mut commands = Commander::new_with_exec(mock);

        commands.get_mount_information(&context).unwrap();
    }
}
