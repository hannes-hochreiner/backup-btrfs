use mockall::{mock, predicate::eq, Sequence};
use uuid::Uuid;

use super::{Actions, ActionsSystem};
use crate::btrfs::{BtrfsCommands, Subvolume};
use crate::command::Context;
use crate::custom_duration::CustomDuration;
use crate::utils::snapshot::SnapshotLocal;
use anyhow::Result;
use chrono::{TimeZone, Utc};

mock! {
    Btrfs {}
    impl BtrfsCommands for Btrfs {
        fn get_subvolumes(&mut self, subvolume_path: &str, context: &Context) -> Result<Vec<Subvolume>>;
        fn create_snapshot(
            &mut self,
            subvolume_path: &str,
            snapshot_path: &str,
            snapshot_suffix: &str,
            context: &Context,
        ) -> Result<()>;
        fn delete_subvolume(&mut self, subvolume: &str, context: &Context) -> Result<()>;
        fn send_snapshot<'a>(
            &mut self,
            local_snapshot: &SnapshotLocal,
            common_parent: Option<&'a SnapshotLocal>,
            context_local: &Context,
            backup_path: &str,
            context_remote: &Context,
        ) -> Result<()>;
    }
}

#[test]
fn create_snapshot() {
    let mut mock = MockBtrfs::new();
    let subvolume_path = "/subvolume/path";
    let snapshot_path = "/snapshot/path";
    let snapshot_suffix = "snapshot_suffix";
    let context = Context::Local {
        user: "test_user".into(),
    };

    mock.expect_create_snapshot()
        .times(1)
        .with(
            eq(subvolume_path),
            eq(snapshot_path),
            eq(snapshot_suffix),
            eq(context.clone()),
        )
        .returning(|_, _, _, _| Ok(()));

    let mut actions = ActionsSystem {
        btrfs: Box::new(mock),
    };

    actions
        .create_snapshot(subvolume_path, snapshot_path, snapshot_suffix, &context)
        .unwrap();
}

#[test]
fn send_snapshot_parent() {
    let mut mock = MockBtrfs::new();
    let subvolume_path = "/subvolume/path";
    let backup_path = "/backup/path";
    let backup_subvolume_path = "/";
    let context_local = Context::Local {
        user: "test_user".into(),
    };
    let context_remote = Context::Remote {
        user: "remote_user".into(),
        host: "remote_host".into(),
        identity: "remote_identity".into(),
    };
    let parent_uuid = Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap();
    let snapshot_uuid = Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1e").unwrap();

    let mut seq = Sequence::new();

    mock.expect_get_subvolumes()
        .times(1)
        .with(eq(subvolume_path), eq(context_local.clone()))
        .in_sequence(&mut seq)
        .returning(move |_, _| {
            Ok(vec![
                Subvolume {
                    parent_uuid: None,
                    path: "/subvolume/path".into(),
                    received_uuid: None,
                    uuid: parent_uuid.clone(),
                },
                Subvolume {
                    parent_uuid: Some(parent_uuid),
                    path: "/other/2020-05-10T12:00:00Z_test".into(),
                    received_uuid: None,
                    uuid: snapshot_uuid.clone(),
                },
            ])
        });

    mock.expect_get_subvolumes()
        .times(1)
        .with(eq(backup_subvolume_path), eq(context_remote.clone()))
        .in_sequence(&mut seq)
        .returning(move |_, _| {
            Ok(vec![Subvolume {
                parent_uuid: None,
                received_uuid: Some(snapshot_uuid),
                path: "/backup/path/2019-05-10T12:00:00Z_test".into(),
                uuid: Uuid::nil(),
            }])
        });

    mock.expect_send_snapshot()
        .times(1)
        .withf(|local_snapshot, common_parent, _, backup_path, _| {
            local_snapshot.path == "/other/2020-05-10T12:00:00Z_test"
                && common_parent.is_some()
                && backup_path == "/backup/path"
        })
        .in_sequence(&mut seq)
        .returning(|_, _, _, _, _| Ok(()));

    let mut actions = ActionsSystem {
        btrfs: Box::new(mock),
    };

    actions
        .send_snapshot(
            subvolume_path,
            backup_path,
            &context_local,
            backup_subvolume_path,
            &context_remote,
        )
        .unwrap();
}

#[test]
fn send_snapshot_no_parent() {
    let mut mock = MockBtrfs::new();
    let subvolume_path = "/subvolume/path";
    let backup_subvolume_path = "/";
    let backup_path = "/backup/path";
    let context_local = Context::Local {
        user: "test_user".into(),
    };
    let context_remote = Context::Remote {
        user: "remote_user".into(),
        host: "remote_host".into(),
        identity: "remote_identity".into(),
    };
    let parent_uuid = Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap();
    let snapshot_uuid = Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1e").unwrap();

    let mut seq = Sequence::new();

    mock.expect_get_subvolumes()
        .times(1)
        .with(eq(subvolume_path), eq(context_local.clone()))
        .in_sequence(&mut seq)
        .returning(move |_, _| {
            Ok(vec![
                Subvolume {
                    parent_uuid: None,
                    path: "/subvolume/path".into(),
                    received_uuid: None,
                    uuid: parent_uuid.clone(),
                },
                Subvolume {
                    parent_uuid: Some(parent_uuid),
                    path: "/other/2020-05-10T12:00:00Z_test".into(),
                    received_uuid: None,
                    uuid: snapshot_uuid.clone(),
                },
            ])
        });

    mock.expect_get_subvolumes()
        .times(1)
        .with(eq(backup_subvolume_path), eq(context_remote.clone()))
        .in_sequence(&mut seq)
        .returning(move |_, _| {
            Ok(vec![Subvolume {
                parent_uuid: None,
                received_uuid: Some(parent_uuid),
                path: "/backup/path/2019-05-10T12:00:00Z_test".into(),
                uuid: Uuid::nil(),
            }])
        });

    mock.expect_send_snapshot()
        .times(1)
        .withf(|local_snapshot, common_parent, _, backup_path, _| {
            local_snapshot.path == "/other/2020-05-10T12:00:00Z_test"
                && common_parent.is_none()
                && backup_path == "/backup/path"
        })
        .in_sequence(&mut seq)
        .returning(|_, _, _, _, _| Ok(()));

    let mut actions = ActionsSystem {
        btrfs: Box::new(mock),
    };

    actions
        .send_snapshot(
            subvolume_path,
            backup_path,
            &context_local,
            backup_subvolume_path,
            &context_remote,
        )
        .unwrap();
}

#[test]
fn police_local_snapshots() {
    let mut mock = MockBtrfs::new();

    let context = Context::Local {
        user: "test_user".into(),
    };
    let suffix = "test2";
    let parent_uuid = Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap();
    let latest_local_snapshot = Subvolume {
        parent_uuid: Some(parent_uuid),
        path: "/snapshots/home/2020-05-01T13:00:00Z_test2".into(),
        received_uuid: None,
        uuid: Uuid::nil(),
    };
    let policy = vec![CustomDuration::minutes(10)];
    let timestamp = Utc.ymd(2020, 5, 10).and_hms(12, 0, 0);
    let mut seq = Sequence::new();
    let subvolume_path = "/";

    mock.expect_get_subvolumes()
        .times(1)
        .with(eq(subvolume_path), eq(context.clone()))
        .in_sequence(&mut seq)
        .returning(move |_, _| {
            Ok(vec![
                Subvolume {
                    parent_uuid: None,
                    path: "/subvolume/path".into(),
                    received_uuid: None,
                    uuid: parent_uuid.clone(),
                },
                Subvolume {
                    parent_uuid: Some(parent_uuid),
                    path: "/other/2020-05-10T12:00:00Z_test".into(),
                    received_uuid: None,
                    uuid: Uuid::parse_str("4f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
                },
            ])
        });

    let mut actions = ActionsSystem {
        btrfs: Box::new(mock),
    };

    actions
        .police_snapshots(
            subvolume_path,
            &context,
            &latest_local_snapshot,
            &policy,
            &timestamp.into(),
            suffix,
        )
        .unwrap();
}
