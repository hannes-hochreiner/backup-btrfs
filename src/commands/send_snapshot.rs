use crate::{backup_error::BackupError, objects::*};
use exec_rs::{Context, Exec};

pub trait CommandSendSnapshot {
    /// Send a local snapshot to a remote host
    ///
    /// * `local_snapshot` - snapshot to be sent
    /// * `common_parent` - parent snapshot (must be available on the remote host as well)
    /// * `context_local` - context to execute the local commands
    /// * `backup_path` - base path to store the snapshot on the remote host
    /// * `context_remote` - context to execute the remote commands
    ///
    fn send_snapshot<'a>(
        &mut self,
        local_snapshot: &SubvolumeInfo,
        common_parent: Option<&'a SubvolumeInfo>,
        context_local: &Context,
        backup_path: &str,
        context_remote: &Context,
    ) -> Result<(), BackupError>;
}

impl<T: Exec> CommandSendSnapshot for super::Commander<T> {
    fn send_snapshot(
        &mut self,
        local_snapshot: &SubvolumeInfo,
        common_parent: Option<&SubvolumeInfo>,
        context_local: &Context,
        backup_path: &str,
        context_remote: &Context,
    ) -> Result<(), BackupError> {
        log::debug!(
            "sending snapshot: \"{}\" to \"{}\"",
            local_snapshot.fs_path,
            backup_path
        );
        let mut args = vec!["btrfs", "send"];

        if let Some(parent_snapshot) = common_parent {
            args.push("-p");
            args.push(&parent_snapshot.fs_path);
        }

        args.push(&local_snapshot.fs_path);

        self.exec.exec_piped(&[
            ("sudo", &args, Some(context_local)),
            (
                "sudo",
                &["btrfs", "receive", backup_path],
                Some(context_remote),
            ),
        ])?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::commands::Commander;
    use exec_rs::MockExec;
    use uuid::Uuid;

    #[test]
    fn send_snapshot_no_parent() {
        let context_local = Context::Local {
            user: "test".into(),
        };
        let context_remote = Context::Remote {
            config: Some("/tmp/ident".into()),
        };
        let snapshot_local = SubvolumeInfo {
            fs_path: "/snapshots/to_be_sent".into(),
            btrfs_path: "/btrfs/path".into(),
            uuid: Uuid::nil(),
        };
        let mut mock = MockExec::new();
        let mock_context_local = context_local.clone();
        let mock_context_remote = context_remote.clone();

        mock.expect_exec_piped()
            .once()
            .withf(move |commands| {
                assert_eq!(commands.len(), 2);
                assert_eq!(commands[0].0, "sudo");
                assert_eq!(commands[0].1, &["btrfs", "send", "/snapshots/to_be_sent"]);
                assert_eq!(commands[0].2, Some(&mock_context_local));
                assert_eq!(commands[1].0, "sudo");
                assert_eq!(
                    commands[1].1,
                    &["btrfs", "receive", "/backups/to_be_received"]
                );
                assert_eq!(commands[1].2, Some(&mock_context_remote));
                true
            })
            .returning(|_| Ok(String::new()));

        let mut commander = Commander::new_with_exec(mock);

        assert!(commander
            .send_snapshot(
                &snapshot_local,
                None,
                &context_local,
                "/backups/to_be_received",
                &context_remote
            )
            .is_ok());
    }

    #[test]
    fn send_snapshot_parent() {
        let context_local = Context::Local {
            user: "test".into(),
        };
        let context_remote = Context::Remote {
            config: Some("/tmp/ident".into()),
        };
        let snapshot_local = SubvolumeInfo {
            fs_path: "/snapshots/to_be_sent".into(),
            btrfs_path: "/btrfs/path".into(),
            uuid: Uuid::nil(),
        };
        let snapshot_parent = SubvolumeInfo {
            fs_path: "/snapshots/parent".into(),
            btrfs_path: "/root/snapshots/parent".into(),
            uuid: Uuid::nil(),
        };

        let mut mock = MockExec::new();
        let mock_context_local = context_local.clone();
        let mock_context_remote = context_remote.clone();

        mock.expect_exec_piped()
            .once()
            .withf(move |commands| {
                assert_eq!(commands.len(), 2);
                assert_eq!(commands[0].0, "sudo");
                assert_eq!(
                    commands[0].1,
                    &[
                        "btrfs",
                        "send",
                        "-p",
                        "/snapshots/parent",
                        "/snapshots/to_be_sent"
                    ]
                );
                assert_eq!(commands[0].2, Some(&mock_context_local));
                assert_eq!(commands[1].0, "sudo");
                assert_eq!(
                    commands[1].1,
                    &["btrfs", "receive", "/backups/to_be_received"]
                );
                assert_eq!(commands[1].2, Some(&mock_context_remote));
                true
            })
            .returning(|_| Ok(String::new()));

        let mut commander = Commander::new_with_exec(mock);

        assert!(commander
            .send_snapshot(
                &snapshot_local,
                Some(&snapshot_parent),
                &context_local,
                "/backups/to_be_received",
                &context_remote
            )
            .is_ok());
    }
}
