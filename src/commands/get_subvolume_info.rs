use crate::{backup_error::BackupError, objects::SubvolumeInfo};
use exec_rs::{Context, Exec};
use std::str::FromStr;
use uuid::Uuid;

pub trait CommandGetSubvolumeInfo {
    /// Get subvolume info
    ///
    /// * `subvolume_path` - path of the btrfs subvolume
    /// * `exec` - command executor
    /// * `context` - context in which to execute the command
    ///
    fn get_subvolume_info(
        &mut self,
        subvolume_path: &str,
        context: &Context,
    ) -> Result<SubvolumeInfo, BackupError>;
}

impl<T: Exec> CommandGetSubvolumeInfo for super::Commander<T> {
    fn get_subvolume_info(
        &mut self,
        subvolume_path: &str,
        context: &Context,
    ) -> Result<SubvolumeInfo, BackupError> {
        let command_output = self.exec.exec(
            "sudo",
            &["btrfs", "subvolume", "show", subvolume_path],
            Some(context),
        )?;
        let mut lines = command_output.lines();
        let btrfs_path_raw = lines
            .next()
            .ok_or(BackupError::SubvolumeInfoParsing(String::from(
                "could not find first line",
            )))?
            .trim();
        let btrfs_path = match btrfs_path_raw.starts_with("/") {
            true => btrfs_path_raw.to_string(),
            false => format!("/{}", btrfs_path_raw),
        };
        let uuid = lines
            .find_map(|l| {
                match l
                    .split_once(":")
                    .map(|(key, value)| (key.trim(), value.trim()))
                {
                    Some(("UUID", value)) => Some(Uuid::from_str(value)),
                    _ => None,
                }
            })
            .ok_or(BackupError::SubvolumeInfoParsing(String::from(
                "could not find UUID of subvolume".to_string(),
            )))??;

        Ok(SubvolumeInfo {
            btrfs_path,
            fs_path: subvolume_path.to_string(),
            uuid,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::commands::Commander;

    use super::*;
    use exec_rs::MockExec;

    #[test]
    fn get_subvolume_info_1() {
        let ctx = Context::Remote {
            host: "host".into(),
            user: "user".into(),
            identity: "/home/test/.ssh".into(),
        };

        let mut mock = MockExec::new();

        mock.expect_exec()
            .once()
            .withf(|command, args, _| {
                assert_eq!(command, "sudo");
                assert_eq!(args, &["btrfs", "subvolume", "show", "/data"]);
                true
            })
            .returning(|_, _, _| {
                Ok(String::from(
                    r#"/
            Name:                   <FS_TREE>
            UUID:                   672e900c-a5dd-47e2-b5c8-59587ee1fae3
            Parent UUID:            -
            Received UUID:          -
            Creation time:          2022-11-12 11:27:32 +0100
            Subvolume ID:           5
            Generation:             539
            Gen at creation:        0
            Parent ID:              0
            Top level ID:           0
            Flags:                  -
            Send transid:           0
            Send time:              2022-11-12 11:27:32 +0100
            Receive transid:        0
            Receive time:           -
            Snapshot(s):
    "#,
                ))
            });

        let mut commander = Commander::new_with_exec(mock);

        assert_eq!(
            commander.get_subvolume_info("/data", &ctx).unwrap(),
            SubvolumeInfo {
                btrfs_path: "/".to_string(),
                fs_path: "/data".to_string(),
                uuid: Uuid::from_str("672e900c-a5dd-47e2-b5c8-59587ee1fae3").unwrap()
            }
        );
    }

    #[test]
    fn get_subvolume_info_2() {
        let ctx = Context::Remote {
            host: "host".into(),
            user: "user".into(),
            identity: "/home/test/.ssh".into(),
        };

        let mut mock = MockExec::new();

        mock.expect_exec()
            .once()
            .withf(|command, args, _| {
                assert_eq!(command, "sudo");
                assert_eq!(args, &["btrfs", "subvolume", "show", "/home"]);
                true
            })
            .returning(|_, _, _| {
                Ok(String::from(
                    r#"home
                    Name:                   home
                    UUID:                   11eed410-7829-744e-8288-35c21d278f8e
                    Parent UUID:            -
                    Received UUID:          -
                    Creation time:          2021-04-02 05:53:59 +0200
                    Subvolume ID:           256
                    Generation:             966689
                    Gen at creation:        6
                    Parent ID:              5
                    Top level ID:           5
                    Flags:                  -
                    Send transid:           0
                    Send time:              2021-04-02 05:53:59 +0200
                    Receive transid:        0
                    Receive time:           -
                    Snapshot(s):
                                            root/snapshots/2022-12-03T19:07:26Z_inf_home
                                            root/snapshots/2022-12-03T20:37:50Z_inf_home
                                            root/snapshots/2022-12-03T21:38:49Z_inf_home
                                            root/snapshots/2022-12-03T21:54:05Z_inf_home"#,
                ))
            });

        let mut commander = Commander::new_with_exec(mock);

        assert_eq!(
            commander.get_subvolume_info("/home", &ctx).unwrap(),
            SubvolumeInfo {
                btrfs_path: "/home".to_string(),
                fs_path: "/home".to_string(),
                uuid: Uuid::from_str("11eed410-7829-744e-8288-35c21d278f8e").unwrap()
            }
        );
    }
}
