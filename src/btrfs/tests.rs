use crate::{
    btrfs::{Btrfs, Subvolume},
    command::{CommandMock, Context},
};
use std::str::FromStr;
use uuid::Uuid;

#[test]
fn delete_subvolume() {
    let mut btrfs = Btrfs {
        command: Box::new(CommandMock {
            commands: vec![(
                "sudo btrfs subvolume delete \"/tmp\"".into(),
                Context::Local {
                    user: "test".into(),
                },
            )],
            responses: vec![String::new()],
        }),
    };

    assert!(btrfs
        .delete_subvolume(
            &String::from("/tmp"),
            &Context::Local {
                user: "test".into()
            }
        )
        .is_ok());
}

#[test]
fn delete_subvolume_home() {
    let mut btrfs = Btrfs {
        command: Box::new(CommandMock {
            commands: vec![(
                "sudo btrfs subvolume delete \"/home\"".into(),
                Context::Local {
                    user: "test".into(),
                },
            )],
            responses: vec![String::new()],
        }),
    };

    assert!(btrfs
        .delete_subvolume(
            &String::from("/home"),
            &Context::Local {
                user: "test".into()
            }
        )
        .is_err());
}

#[test]
fn delete_subvolume_root() {
    let mut btrfs = Btrfs {
        command: Box::new(CommandMock {
            commands: vec![(
                "sudo btrfs subvolume delete \"/\"".into(),
                Context::Local {
                    user: "test".into(),
                },
            )],
            responses: vec![String::new()],
        }),
    };

    assert!(btrfs
        .delete_subvolume(
            &String::from("/"),
            &Context::Local {
                user: "test".into()
            }
        )
        .is_err());
}

#[test]
fn get_local_subvolumes() {
    let mut btrfs = Btrfs {
        command: Box::new(CommandMock {
            commands: vec![(
                String::from("sudo btrfs subvolume list -tupqR --sort=rootid /"),
                Context::Local {
                    user: "test".into(),
                },
            )],
            responses: vec![String::from(
                r#"ID      gen     parent  top level       parent_uuid     received_uuid   uuid    path
--      ---     ------  ---------       -----------     -------------   ----    ----
256     119496  5       5               -                                       -                                       11eed410-7829-744e-8288-35c21d278f8e    home
359     119496  5       5               -                                       -                                       32c672fa-d3ce-0b4e-8eaa-ab9205f377ca    root
360     119446  359     359             -                                       -                                       5f0b151b-52e4-4445-aa94-d07056733a1f    opt/btrfs_test
367     118687  359     359             5f0b151b-52e4-4445-aa94-d07056733a1f    -                                       7f305e3e-851b-974b-a476-e2f206e7a407    snapshots/2021-05-02T07:40:32Z_inf_btrfs_test
370     119446  359     359             5f0b151b-52e4-4445-aa94-d07056733a1f    -                                       1bd1da76-b61f-db41-a2d2-c3474a31f38f    snapshots/2021-05-02T13:38:49Z_inf_btrfs_test
"#,
            )],
        }),
    };

    assert_eq!(
        btrfs.get_local_subvolumes("test").unwrap(),
        vec![
            Subvolume {
                uuid: Uuid::from_str("11eed410-7829-744e-8288-35c21d278f8e").unwrap(),
                path: String::from("/home"),
                parent_uuid: None,
                received_uuid: None
            },
            Subvolume {
                uuid: Uuid::from_str("32c672fa-d3ce-0b4e-8eaa-ab9205f377ca").unwrap(),
                path: String::from("/root"),
                parent_uuid: None,
                received_uuid: None
            },
            Subvolume {
                uuid: Uuid::from_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
                path: String::from("/opt/btrfs_test"),
                parent_uuid: None,
                received_uuid: None
            },
            Subvolume {
                uuid: Uuid::from_str("7f305e3e-851b-974b-a476-e2f206e7a407").unwrap(),
                path: String::from("/snapshots/2021-05-02T07:40:32Z_inf_btrfs_test"),
                parent_uuid: Some(Uuid::from_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap()),
                received_uuid: None
            },
            Subvolume {
                uuid: Uuid::from_str("1bd1da76-b61f-db41-a2d2-c3474a31f38f").unwrap(),
                path: String::from("/snapshots/2021-05-02T13:38:49Z_inf_btrfs_test"),
                parent_uuid: Some(Uuid::from_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap()),
                received_uuid: None
            },
        ]
    );
}

#[test]
fn get_remote_subvolumes() {
    let mut btrfs = Btrfs {
        command: Box::new(CommandMock {
            commands: vec![(
                String::from("sudo btrfs subvolume list -tupqR --sort=rootid /"),
                Context::Remote {
                    host: "host".into(),
                    user: "user".into(),
                    identity: "/home/test/.ssh".into(),
                },
            )],
            responses: vec![String::from(
                r#"ID      gen     parent  top level       parent_uuid     received_uuid   uuid    path
--      ---     ------  ---------       -----------     -------------   ----    ----
256     10789   5       5               -                                       -                                       0b5cc138-af8e-2744-be4f-bdede1b509ef    root
259     9051    256     256             -                                       -                                       574fef8d-7951-3e45-aa29-7167b9d4590a    var/lib/portables
270     4965    256     256             -                                       dc4e1039-9241-cd47-9c10-a5d1ce15ba20    d1bd727c-8a02-bb44-bdd2-bae468651e98    backups/2021-05-04T19:48:42Z_inf_btrfs_test
328     7505    256     256             19391f90-9007-3e4b-b757-6e5d2421b9bd    53bb5cfa-f45e-d147-9407-006271609062    54b52286-8265-9444-8603-214e7e0533e0    backups/2021-05-10T06:14:04Z_inf_btrfs_test
"#,
            )],
        }),
    };

    assert_eq!(
        btrfs
            .get_remote_subvolumes("host", "user", "/home/test/.ssh")
            .unwrap(),
        vec![
            Subvolume {
                uuid: Uuid::from_str("0b5cc138-af8e-2744-be4f-bdede1b509ef").unwrap(),
                path: String::from("/root"),
                parent_uuid: None,
                received_uuid: None
            },
            Subvolume {
                uuid: Uuid::from_str("574fef8d-7951-3e45-aa29-7167b9d4590a").unwrap(),
                path: String::from("/var/lib/portables"),
                parent_uuid: None,
                received_uuid: None
            },
            Subvolume {
                uuid: Uuid::from_str("d1bd727c-8a02-bb44-bdd2-bae468651e98").unwrap(),
                path: String::from("/backups/2021-05-04T19:48:42Z_inf_btrfs_test"),
                parent_uuid: None,
                received_uuid: Some(
                    Uuid::from_str("dc4e1039-9241-cd47-9c10-a5d1ce15ba20").unwrap()
                )
            },
            Subvolume {
                uuid: Uuid::from_str("54b52286-8265-9444-8603-214e7e0533e0").unwrap(),
                path: String::from("/backups/2021-05-10T06:14:04Z_inf_btrfs_test"),
                parent_uuid: Some(Uuid::from_str("19391f90-9007-3e4b-b757-6e5d2421b9bd").unwrap()),
                received_uuid: Some(
                    Uuid::from_str("53bb5cfa-f45e-d147-9407-006271609062").unwrap()
                )
            },
        ]
    );
}
