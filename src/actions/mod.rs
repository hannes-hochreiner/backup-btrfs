use crate::backup_error::BackupError;
use crate::commands::{Commander, Commands};
use crate::custom_duration::CustomDuration;
use crate::objects::*;
use chrono::{DateTime, Duration, FixedOffset, Utc};
use exec_rs::{CommandExec, Context};
use policer::police;
use std::collections::HashMap;
use std::convert::TryInto;
use std::path::Path;
use uuid::Uuid;

pub trait Actions {
    /// Create snapshot
    ///
    /// * `source_subvolume_path` - path of the subvolume that serves as the parent of the new snapshot
    /// * `snapshot_path` - path at which to create the snapshot
    /// * `snapshot_suffix` - suffix for the new snapshot
    /// * `context` - the context to use for the execution of the required commands
    fn create_snapshot(
        &mut self,
        source_subvolume_path: &str,
        snapshot_path: &str,
        snapshot_suffix: &str,
        context: &Context,
    ) -> Result<SubvolumeInfo, BackupError>;
    /// Get mount information
    ///
    /// * `context` - the context to use for the execution of the required commands
    fn get_mount_information(
        &mut self,
        context: &Context,
    ) -> Result<Vec<MountInformation>, BackupError>;
    /// Send snapshot
    ///
    /// * `parent_subvolume_path` - path of the parent subvolume of the snapshot to be sent
    /// * `local_device` - path of the local device
    /// * `local_subvolume_path` - path the subvolume containing the snapshot to be sent
    /// * `local_mount_information` - local mount information
    /// * `snapshot` - snapshot to be sent
    /// * `context_local` - context for executing local commands
    /// * `remote_subvolume_path` - path of the subvolume to receive the snapshot
    /// * `remote_snapshot_path` - path of the remote snapshots
    /// * `context_remote` - context for executing remote commands
    fn send_snapshot(
        &mut self,
        parent_subvolume_path: &str,
        local_device: &str,
        local_subvolume_path: &str,
        local_mount_information: &Vec<MountInformation>,
        snapshot: &SubvolumeInfo,
        context_local: &Context,
        remote_subvolume_path: &str,
        remote_snapshot_path: &str,
        context_remote: &Context,
    ) -> Result<(), BackupError>;
    /// Police snapshots
    ///
    /// * `subvolume_path` - path of the subvolume containing the snapshots to be policed
    /// * `context` - context in which to execute the commands
    /// * `latest_local_snapshot` - latest local snapshot (will be excluded from the deletion list, if contained)
    /// * `policy` - policy to be applied
    /// * `timestamp` - timestamp to use as the current moment
    /// * `suffix` - suffix of the snapshots (used for filtering)
    /// * `device` - path of the device
    /// * `mount_information` - mount information (used to translate btrfs paths in to filesystem paths)
    fn police_snapshots(
        &mut self,
        subvolume_path: &str,
        context: &Context,
        latest_local_snapshot: &SubvolumeInfo,
        policy: &Vec<CustomDuration>,
        timestamp: &DateTime<FixedOffset>,
        suffix: &str,
        device: &str,
        mount_information: &Vec<MountInformation>,
    ) -> Result<(), BackupError>;
}

pub struct ActionsSystem<C: Commands> {
    commander: C,
}

impl Default for ActionsSystem<Commander<CommandExec>> {
    fn default() -> Self {
        ActionsSystem {
            commander: Commander::default(),
        }
    }
}

impl<C: Commands> ActionsSystem<C> {
    /// Get the newest subvolume that was used received on the remote host and is still available locally
    ///
    /// * `subvolumes_local` - list of local subvolumes
    /// * `snapshots_remote` - list of remote subvolumes
    ///
    pub fn get_common_parent<'a>(
        subvolumes_local: &'a Vec<Subvolume>,
        subvolumes_remote: &'a Vec<Subvolume>,
    ) -> Result<Option<&'a Subvolume>, BackupError> {
        let uuids: HashMap<&Uuid, &Subvolume> =
            subvolumes_local.iter().map(|e| (&e.uuid, e)).collect();

        let mut common_subvolumes: Vec<&Subvolume> = subvolumes_remote
            .iter()
            .filter(|sv| match sv.received_uuid {
                Some(received_uuid) => match uuids.get(&received_uuid) {
                    Some(_) => true,
                    _ => false,
                },
                None => false,
            })
            .collect();

        common_subvolumes.sort_by_key(|&sv| &sv.btrfs_path);

        Ok(common_subvolumes
            .last()
            .and_then(|&s| s.received_uuid)
            .and_then(|uuid| uuids.get(&uuid))
            .copied())
    }

    pub fn btrfs_to_fs_path(
        mount_information: &Vec<MountInformation>,
        device: &str,
        btrfs_path: &str,
    ) -> Result<String, BackupError> {
        let mut mi = mount_information
            .iter()
            .filter(|mi| mi.fs_type == "btrfs" && mi.device == device)
            .filter_map(|mi| {
                Path::new(btrfs_path)
                    .strip_prefix(&mi.root)
                    .and_then(|p| Ok(Path::new(&mi.mount_point).join(p)))
                    .ok()
                    .and_then(|s| {
                        s.to_str()
                            .and_then(|s| Some((mi.root.len(), String::from(s))))
                    })
            })
            .collect::<Vec<(usize, String)>>();

        mi.sort_by_key(|s| s.0);

        mi.last()
            .map(|e| e.1.clone())
            .ok_or(BackupError::PathConversionError.into())
    }

    pub fn eq_or_received(sv: &Subvolume, svi: &SubvolumeInfo) -> bool {
        sv.uuid == svi.uuid
            || sv
                .received_uuid
                .and_then(|uuid| match uuid == svi.uuid {
                    true => Some(()),
                    false => None,
                })
                .is_some()
    }
}

impl<C: Commands> Actions for ActionsSystem<C> {
    fn create_snapshot(
        &mut self,
        subvolume_path: &str,
        snapshot_path: &str,
        snapshot_suffix: &str,
        context: &Context,
    ) -> Result<SubvolumeInfo, BackupError> {
        // create snapshot
        let snapshot_path = self.commander.snapshot_subvolume(
            subvolume_path,
            snapshot_path,
            snapshot_suffix,
            &Utc::now(),
            context,
        )?;
        // get snapshot/subvolume information
        self.commander.get_subvolume_info(&snapshot_path, context)
    }

    fn send_snapshot(
        &mut self,
        parent_subvolume_path: &str,
        local_device: &str,
        local_subvolume_path: &str,
        local_mount_information: &Vec<MountInformation>,
        snapshot: &SubvolumeInfo,
        context_local: &Context,
        remote_subvolume_path: &str,
        remote_snapshot_path: &str,
        context_remote: &Context,
    ) -> Result<(), BackupError> {
        // get parent subvolume info
        let parent_subvolume = self
            .commander
            .get_subvolume_info(parent_subvolume_path, context_local)?;

        // get local snapshots, which are children of the supplied parent subvolume
        let subvolumes_local = self
            .commander
            .get_subvolumes(local_subvolume_path, &context_local)?
            .iter()
            .filter(|sv| match sv.parent_uuid {
                Some(sv_uuid) => sv_uuid == parent_subvolume.uuid,
                None => false,
            })
            .cloned()
            .collect();

        // get remote snapshots
        let subvolumes_remote = self
            .commander
            .get_subvolumes(remote_subvolume_path, &context_remote)?;

        // find common parent
        let common_parent =
            ActionsSystem::<C>::get_common_parent(&subvolumes_local, &subvolumes_remote)?
                .map(|sv| {
                    Ok::<SubvolumeInfo, BackupError>(SubvolumeInfo {
                        btrfs_path: sv.btrfs_path.clone(),
                        fs_path: ActionsSystem::<C>::btrfs_to_fs_path(
                            local_mount_information,
                            local_device,
                            &sv.btrfs_path,
                        )?,
                        uuid: sv.uuid,
                    })
                })
                .transpose()?;

        match &common_parent {
            Some(s) => log::info!("found common parent snapshot \"{}\"", &s.fs_path),
            None => log::info!("no common parent snapshot found"),
        }

        // send remote backup
        self.commander.send_snapshot(
            snapshot,
            common_parent.as_ref(),
            &context_local,
            remote_snapshot_path,
            &context_remote,
        )?;

        Ok(())
    }

    fn police_snapshots(
        &mut self,
        subvolume_path: &str,
        context: &Context,
        latest_local_snapshot: &SubvolumeInfo,
        policy: &Vec<CustomDuration>,
        timestamp: &DateTime<FixedOffset>,
        suffix: &str,
        device: &str,
        mount_information: &Vec<MountInformation>,
    ) -> Result<(), BackupError> {
        // get subvolumes
        let subvolumes = self.commander.get_subvolumes(subvolume_path, context)?;
        // filter out the relevant snapshots
        let snapshots: Vec<(DateTime<Utc>, Subvolume)> = subvolumes
            .iter()
            // .filter(|sv| sv.btrfs_path.ends_with(suffix))
            .filter_map(|sv| {
                sv.btrfs_path
                    .rfind('/')
                    .and_then(|idx| {
                        sv.btrfs_path[(idx + 1)..].strip_suffix(&("_".to_string() + suffix))
                    })
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .and_then(|date| Some((DateTime::<Utc>::from(date), sv.clone())))
            })
            .collect();

        log::debug!("snapshots: {}", snapshots.len());

        // convert the policy
        let policy: Vec<Duration> = policy.iter().map(|d| d.try_into()).flatten().collect();
        // apply the policy
        let to_be_deleted = police(&DateTime::<Utc>::from(*timestamp), &policy, &snapshots);

        log::debug!("subvolumes to be deleted: {}", to_be_deleted.len());

        // filter out the latest local snapshot
        for sv in to_be_deleted.iter().filter_map(|sv| {
            match ActionsSystem::<C>::eq_or_received(&sv.1, latest_local_snapshot) {
                true => None,
                false => Some(sv.1.clone()),
            }
        }) {
            let subvolume_path =
                ActionsSystem::<C>::btrfs_to_fs_path(mount_information, device, &sv.btrfs_path)?;
            log::info!("deleting subvolume: \"{}\"", subvolume_path);
            self.commander.delete_subvolume(&subvolume_path, context)?;
        }

        Ok(())
    }

    fn get_mount_information(
        &mut self,
        context: &Context,
    ) -> Result<Vec<MountInformation>, BackupError> {
        self.commander.get_mount_information(context)
    }
}

#[cfg(test)]
mod test {
    use chrono::TimeZone;
    use mockall::Sequence;

    use crate::commands::MockCommander;

    use super::*;

    #[test]
    fn eq_uuid_uuid() {
        let sv = Subvolume {
            btrfs_path: String::from("/some/path"),
            parent_uuid: None,
            received_uuid: None,
            uuid: Uuid::parse_str("84749615-6961-4693-88d3-0bb820dc7e3f").unwrap(),
        };
        let svi = SubvolumeInfo {
            btrfs_path: String::from("/some/path"),
            fs_path: String::from("/other/path"),
            uuid: Uuid::parse_str("84749615-6961-4693-88d3-0bb820dc7e3f").unwrap(),
        };

        assert!(ActionsSystem::<Commander<CommandExec>>::eq_or_received(
            &sv, &svi
        ));
    }

    #[test]
    fn eq_uuid_received() {
        let sv = Subvolume {
            btrfs_path: String::from("/some/path"),
            parent_uuid: None,
            received_uuid: Some(Uuid::parse_str("84749615-6961-4693-88d3-0bb820dc7e3f").unwrap()),
            uuid: Uuid::parse_str("8fe49b0e-6bb3-4f7e-9ead-6fcfc6f79658").unwrap(),
        };
        let svi = SubvolumeInfo {
            btrfs_path: String::from("/some/path"),
            fs_path: String::from("/other/path"),
            uuid: Uuid::parse_str("84749615-6961-4693-88d3-0bb820dc7e3f").unwrap(),
        };

        assert!(ActionsSystem::<Commander<CommandExec>>::eq_or_received(
            &sv, &svi
        ));
    }

    #[test]
    fn uneq_uuid_received() {
        let sv1 = Subvolume {
            btrfs_path: String::from("/some/path"),
            parent_uuid: None,
            received_uuid: Some(Uuid::parse_str("45feb757-df21-42ae-b923-bef21ee993c9").unwrap()),
            uuid: Uuid::parse_str("8fe49b0e-6bb3-4f7e-9ead-6fcfc6f79658").unwrap(),
        };
        let sv2 = Subvolume {
            btrfs_path: String::from("/some/path"),
            parent_uuid: None,
            received_uuid: None,
            uuid: Uuid::parse_str("8fe49b0e-6bb3-4f7e-9ead-6fcfc6f79658").unwrap(),
        };
        let svi = SubvolumeInfo {
            btrfs_path: String::from("/some/path"),
            fs_path: String::from("/other/path"),
            uuid: Uuid::parse_str("84749615-6961-4693-88d3-0bb820dc7e3f").unwrap(),
        };

        assert!(!ActionsSystem::<Commander<CommandExec>>::eq_or_received(
            &sv1, &svi
        ));
        assert!(!ActionsSystem::<Commander<CommandExec>>::eq_or_received(
            &sv2, &svi
        ));
    }

    #[test]
    fn btrfs_to_fs_path_1() {
        let mi = vec![
            MountInformation {
                device: String::from("device"),
                fs_type: String::from("btrfs"),
                mount_point: String::from("/mount/point"),
                root: String::from("/test"),
                properties: HashMap::new(),
            },
            MountInformation {
                device: String::from("device"),
                fs_type: String::from("btrfs"),
                mount_point: String::from("/mount/point"),
                root: String::from("/test2"),
                properties: HashMap::new(),
            },
        ];

        assert_eq!(
            ActionsSystem::<Commander<CommandExec>>::btrfs_to_fs_path(
                &mi,
                "device",
                "/test/some/other/path"
            )
            .unwrap(),
            String::from("/mount/point/some/other/path")
        );
    }

    #[test]
    fn btrfs_to_fs_path_2() {
        let mi = vec![
            MountInformation {
                device: String::from("device"),
                fs_type: String::from("btrfs"),
                mount_point: String::from("/mount/point"),
                root: String::from("/test"),
                properties: HashMap::new(),
            },
            MountInformation {
                device: String::from("device"),
                fs_type: String::from("btrfs"),
                mount_point: String::from("/mount/point/2"),
                root: String::from("/test/some/other"),
                properties: HashMap::new(),
            },
        ];

        assert_eq!(
            ActionsSystem::<Commander<CommandExec>>::btrfs_to_fs_path(
                &mi,
                "device",
                "/test/some/other/path"
            )
            .unwrap(),
            String::from("/mount/point/2/path")
        );
    }

    #[test]
    fn btrfs_to_fs_path_3() {
        let mi = vec![
            MountInformation {
                device: String::from("/dev/mapper/device_1"),
                fs_type: String::from("btrfs"),
                mount_point: String::from("/mount/point"),
                root: String::from("/"),
                properties: HashMap::new(),
            },
            MountInformation {
                device: String::from("/dev/mapper/device_2"),
                fs_type: String::from("btrfs"),
                mount_point: String::from("/mount/point/2"),
                root: String::from("/"),
                properties: HashMap::new(),
            },
        ];

        assert_eq!(
            ActionsSystem::<Commander<CommandExec>>::btrfs_to_fs_path(
                &mi,
                "/dev/mapper/device_1",
                "/test/some/other/path"
            )
            .unwrap(),
            String::from("/mount/point/test/some/other/path")
        );
    }

    #[test]
    fn create_snapshot() {
        let mut mock = MockCommander::new();
        let subvolume_path = "/subvolume/path";
        let snapshot_path = "/snapshot/path";
        let snapshot_suffix = "snapshot_suffix";
        let context = Context::Local {
            user: "test_user".into(),
        };
        let new_snapshot_path = "/snapshot/path/2022-12-11T21:24:04+01:00_snapshot_suffix";
        let subvolume_info = SubvolumeInfo {
            btrfs_path: "/btrfs/path".into(),
            fs_path: new_snapshot_path.into(),
            uuid: Uuid::nil(),
        };

        let mut sequence = Sequence::new();

        mock.expect_snapshot_subvolume()
            .times(1)
            .in_sequence(&mut sequence)
            .withf(
                move |f_subvolume_path, f_snapshot_path, f_snapshot_suffix, _, _| {
                    assert_eq!(f_subvolume_path, subvolume_path);
                    assert_eq!(f_snapshot_path, snapshot_path);
                    assert_eq!(f_snapshot_suffix, snapshot_suffix);
                    true
                },
            )
            .returning(move |_, _, _, _, _| Ok(new_snapshot_path.into()));
        mock.expect_get_subvolume_info()
            .times(1)
            .in_sequence(&mut sequence)
            .withf(move |f_subvolume_path, _| {
                assert_eq!(f_subvolume_path, new_snapshot_path);
                true
            })
            .returning(move |_, _| {
                Ok(SubvolumeInfo {
                    btrfs_path: "/btrfs/path".into(),
                    fs_path: new_snapshot_path.into(),
                    uuid: Uuid::nil(),
                })
            });

        let mut actions = ActionsSystem { commander: mock };

        let test_path = actions
            .create_snapshot(subvolume_path, snapshot_path, snapshot_suffix, &context)
            .unwrap();

        assert_eq!(test_path, subvolume_info);
    }

    #[test]
    fn send_snapshot_parent() {
        let mut mock = MockCommander::new();
        let local_subvolume_path = "/subvolume/path";
        let snapshot = &SubvolumeInfo {
            fs_path: String::from("/backup/path"),
            btrfs_path: String::from("/root/path"),
            uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1e").unwrap(),
        };
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
        let parent_subvolume = SubvolumeInfo {
            btrfs_path: "/btrfs/path".into(),
            fs_path: "/fs/path".into(),
            uuid: parent_uuid,
        };
        let parent_subvolume_check = parent_subvolume.clone();
        let parent_subvolume_fs_path_check = parent_subvolume.fs_path.clone();
        let local_mount_information = vec![MountInformation {
            device: String::from("/dev/some/device"),
            fs_type: String::from("btrfs"),
            mount_point: String::from("/data"),
            root: String::from("/subvolume"),
            properties: HashMap::new(),
        }];
        let backup_snapshot_path = "/data/snapshots";
        let mut seq = Sequence::new();

        mock.expect_get_subvolume_info()
            .once()
            .in_sequence(&mut seq)
            .returning(move |f_subvolume_path, _| {
                assert_eq!(f_subvolume_path, parent_subvolume_fs_path_check.clone());
                Ok(parent_subvolume_check.clone())
            });

        mock.expect_get_subvolumes()
            .once()
            .in_sequence(&mut seq)
            .returning(move |f_subvolume_path, f_context| {
                assert_eq!(f_subvolume_path, local_subvolume_path);
                assert!(match f_context {
                    Context::Local { user: _ } => true,
                    _ => false,
                });

                Ok(vec![
                    Subvolume {
                        parent_uuid: None,
                        btrfs_path: "/subvolume/path".into(),
                        received_uuid: None,
                        uuid: parent_uuid.clone(),
                    },
                    Subvolume {
                        parent_uuid: Some(parent_uuid),
                        btrfs_path: "/subvolume/2020-05-10T12:00:00Z_test".into(),
                        received_uuid: None,
                        uuid: snapshot_uuid.clone(),
                    },
                ])
            });

        mock.expect_get_subvolumes()
            .once()
            .in_sequence(&mut seq)
            .returning(move |f_subvolume_path, f_context| {
                assert_eq!(f_subvolume_path, backup_subvolume_path);
                assert!(match f_context {
                    Context::Remote {
                        host: _,
                        identity: _,
                        user: _,
                    } => true,
                    _ => false,
                });

                Ok(vec![Subvolume {
                    parent_uuid: None,
                    received_uuid: Some(snapshot_uuid),
                    btrfs_path: "/backup/path/2019-05-10T12:00:00Z_test".into(),
                    uuid: Uuid::nil(),
                }])
            });

        mock.expect_send_snapshot()
            .times(1)
            .in_sequence(&mut seq)
            .returning(
                move |f_subvolume_info,
                      f_common_parent,
                      f_context_local,
                      f_backup_path,
                      f_context_remote| {
                    assert!(f_common_parent.is_some());
                    assert_eq!(f_backup_path, backup_snapshot_path);
                    assert_eq!(f_subvolume_info.fs_path, "/backup/path");
                    assert!(match f_context_local {
                        Context::Local { user: _ } => true,
                        _ => false,
                    });
                    assert!(match f_context_remote {
                        Context::Remote {
                            host: _,
                            identity: _,
                            user: _,
                        } => true,
                        _ => false,
                    });

                    Ok(())
                },
            );

        let mut actions = ActionsSystem { commander: mock };

        actions
            .send_snapshot(
                &parent_subvolume.fs_path.clone(),
                "/dev/some/device",
                local_subvolume_path,
                &local_mount_information,
                snapshot,
                &context_local,
                backup_subvolume_path,
                backup_snapshot_path,
                &context_remote,
            )
            .unwrap();
    }

    #[test]
    fn send_snapshot_no_parent() {
        let mut mock = MockCommander::new();
        let snapshot = &SubvolumeInfo {
            fs_path: String::from("/backup/path"),
            btrfs_path: String::from("/root/path"),
            uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1e").unwrap(),
        };
        let local_subvolume_path = "/subvolume/path";
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
        let parent_subvolume = SubvolumeInfo {
            btrfs_path: "/btrfs/path".into(),
            fs_path: "/fs/path".into(),
            uuid: parent_uuid,
        };
        let parent_subvolume_check = parent_subvolume.clone();
        let parent_subvolume_fs_path_check = parent_subvolume.fs_path.clone();
        let local_mount_information = vec![MountInformation {
            device: String::from("/dev/some/device"),
            fs_type: String::from("btrfs"),
            mount_point: String::from("/data"),
            root: String::from("/subvolume"),
            properties: HashMap::new(),
        }];
        let backup_snapshot_path = "/data/snapshots";
        let mut seq = Sequence::new();

        mock.expect_get_subvolume_info()
            .once()
            .in_sequence(&mut seq)
            .returning(move |f_subvolume_path, _| {
                assert_eq!(f_subvolume_path, parent_subvolume_fs_path_check.clone());
                Ok(parent_subvolume_check.clone())
            });

        mock.expect_get_subvolumes()
            .once()
            .in_sequence(&mut seq)
            .returning(move |f_subvolume_path, f_context| {
                assert!(match f_context {
                    Context::Local { user: _ } => true,
                    _ => false,
                });
                assert_eq!(f_subvolume_path, local_subvolume_path);
                Ok(vec![
                    Subvolume {
                        parent_uuid: None,
                        btrfs_path: "/subvolume/path".into(),
                        received_uuid: None,
                        uuid: parent_uuid.clone(),
                    },
                    Subvolume {
                        parent_uuid: Some(parent_uuid),
                        btrfs_path: "/other/2020-05-10T12:00:00Z_test".into(),
                        received_uuid: None,
                        uuid: snapshot_uuid.clone(),
                    },
                ])
            });

        mock.expect_get_subvolumes()
            .once()
            .in_sequence(&mut seq)
            .returning(move |f_subvolume_path, f_context| {
                assert_eq!(f_subvolume_path, backup_subvolume_path);
                assert!(match f_context {
                    Context::Remote {
                        host: _,
                        identity: _,
                        user: _,
                    } => true,
                    _ => false,
                });

                Ok(vec![Subvolume {
                    parent_uuid: None,
                    received_uuid: Some(parent_uuid),
                    btrfs_path: "/backup/path/2019-05-10T12:00:00Z_test".into(),
                    uuid: Uuid::nil(),
                }])
            });

        mock.expect_send_snapshot()
            .once()
            .in_sequence(&mut seq)
            .returning(
                move |f_subvolume_info,
                      f_common_parent,
                      f_context_local,
                      f_backup_path,
                      f_context_remote| {
                    assert_eq!(f_subvolume_info.fs_path, "/backup/path");
                    assert!(f_common_parent.is_none());
                    assert_eq!(f_backup_path, backup_snapshot_path);
                    assert!(match f_context_local {
                        Context::Local { user: _ } => true,
                        _ => false,
                    });
                    assert!(match f_context_remote {
                        Context::Remote {
                            host: _,
                            identity: _,
                            user: _,
                        } => true,
                        _ => false,
                    });
                    Ok(())
                },
            );

        let mut actions = ActionsSystem { commander: mock };

        actions
            .send_snapshot(
                &parent_subvolume.fs_path.clone(),
                "/dev/some/device",
                local_subvolume_path,
                &local_mount_information,
                snapshot,
                &context_local,
                backup_subvolume_path,
                backup_snapshot_path,
                &context_remote,
            )
            .unwrap();
    }

    #[test]
    fn police_local_snapshots() {
        let mut mock = MockCommander::new();

        let context = Context::Local {
            user: "test_user".into(),
        };
        let suffix = "test2";
        let parent_uuid = Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap();
        let latest_local_snapshot = SubvolumeInfo {
            btrfs_path: "/snapshots/home/2020-05-01T13:00:00Z_test2".into(),
            uuid: Uuid::nil(),
            fs_path: String::from("/some/path"),
        };
        let policy = vec![CustomDuration::minutes(10)];
        let timestamp = Utc.with_ymd_and_hms(2020, 5, 10, 12, 0, 0).unwrap();
        let mut seq = Sequence::new();
        let subvolume_path = "/";
        let mount_information = vec![MountInformation {
            device: String::from("/dev/some/device"),
            fs_type: String::from("btrfs"),
            mount_point: String::from("/data"),
            root: String::from("/subvolume"),
            properties: HashMap::new(),
        }];

        mock.expect_get_subvolumes()
            .once()
            .in_sequence(&mut seq)
            .returning(move |f_subvolume_path, _| {
                assert_eq!(f_subvolume_path, subvolume_path);

                Ok(vec![
                    Subvolume {
                        parent_uuid: None,
                        btrfs_path: "/subvolume/path".into(),
                        received_uuid: None,
                        uuid: parent_uuid.clone(),
                    },
                    Subvolume {
                        parent_uuid: Some(parent_uuid),
                        btrfs_path: "/other/2020-05-10T12:00:00Z_test".into(),
                        received_uuid: None,
                        uuid: Uuid::parse_str("4f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
                    },
                ])
            });

        let mut actions = ActionsSystem { commander: mock };

        actions
            .police_snapshots(
                subvolume_path,
                &context,
                &latest_local_snapshot,
                &policy,
                &timestamp.into(),
                suffix,
                "/dev/some/device",
                &mount_information,
            )
            .unwrap();
    }
}
