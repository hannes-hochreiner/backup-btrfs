use crate::backup_error::BackupError;
use chrono::{DateTime, SecondsFormat, Utc};
use exec_rs::{Context, Exec};
use std::path::PathBuf;

pub trait CommandSnapshotSubvolume {
    /// Create a snapshot locally
    ///
    /// The new snapshot will be created at the path `<snapshot_path>/<timestamp in rfc3339 format (UTC)>_<suffix>`.
    /// This function executes the command `sudo btrfs subvolume snapshot -r <subvolume_path> <snapshot_path>/<timestamp in rfc3339 format (UTC)>_<suffix>`.
    ///
    /// * `subvolume_path` - path to the subvolume from which to create the snapshot
    /// * `snapshot_path` - base path at which the snapshot should be created
    /// * `snapshot_suffix` - suffix of the subvolume
    /// * `timestamp` - timestamp used for the snapshot; should be close to current time
    /// * `exec` - command executor
    /// * `context` - context in which to execute the command
    /// * return the path of the created snapshot
    ///
    fn snapshot_subvolume(
        &mut self,
        subvolume_path: &str,
        snapshot_path: &str,
        snapshot_suffix: &str,
        timestamp: &DateTime<Utc>,
        context: &Context,
    ) -> Result<String, BackupError>;
}

impl<T: Exec> CommandSnapshotSubvolume for super::Commander<T> {
    fn snapshot_subvolume(
        &mut self,
        subvolume_path: &str,
        snapshot_path: &str,
        snapshot_suffix: &str,
        timestamp: &DateTime<Utc>,
        context: &Context,
    ) -> Result<String, BackupError> {
        let snapshot_path_extension = format!(
            "{}_{}",
            timestamp.to_rfc3339_opts(SecondsFormat::Secs, true),
            snapshot_suffix
        );
        let mut snapshot_path = PathBuf::from(&*snapshot_path);

        snapshot_path.push(snapshot_path_extension);

        let snapshot_path =
            snapshot_path
                .as_path()
                .to_str()
                .ok_or(BackupError::SnapshotSubvolume(String::from(
                    "could not construct snapshot_path",
                )))?;

        self.exec.exec(
            "sudo",
            &[
                "btrfs",
                "subvolume",
                "snapshot",
                "-r",
                subvolume_path,
                snapshot_path,
            ],
            Some(context),
        )?;

        Ok(snapshot_path.to_string())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::commands::Commander;
    use exec_rs::MockExec;

    #[test]
    fn snapshot_subvolume() {
        let context = Context::Remote {
            host: "host".into(),
            user: "user".into(),
            identity: "/home/test/.ssh".into(),
        };

        let timestamp = DateTime::parse_from_rfc3339("2022-11-02T12:13:14Z").unwrap();
        let mut mock = MockExec::new();

        mock.expect_exec()
            .once()
            .withf(move |command, args, _| {
                assert_eq!(command, "sudo");
                assert_eq!(
                    args,
                    &[
                        "btrfs",
                        "subvolume",
                        "snapshot",
                        "-r",
                        "/home",
                        "/snapshots/2022-11-02T12:13:14Z_test_test",
                    ]
                );
                true
            })
            .returning(|_, _, _| Ok(String::new()));

        let mut commands = Commander::new_with_exec(mock);

        assert_eq!(
            commands
                .snapshot_subvolume(
                    "/home",
                    "/snapshots",
                    "test_test",
                    &timestamp.into(),
                    &context,
                )
                .unwrap(),
            "/snapshots/2022-11-02T12:13:14Z_test_test"
        );
    }
}
