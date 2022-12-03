use std::str::FromStr;

use crate::{
    btrfs::Subvolume,
    custom_duration::CustomDuration,
    utils::{MountInformation, Snapshot, SnapshotLocal, SnapshotRemote},
};
use chrono::{TimeZone, Utc};
use uuid::Uuid;

#[test]
fn get_subvolume_by_path() {
    let subvolumes = vec![
        Subvolume {
            uuid: Uuid::from_str("0b5cc138-af8e-2744-be4f-bdede1b509ef").unwrap(),
            path: String::from("/root"),
            parent_uuid: None,
            received_uuid: None,
        },
        Subvolume {
            uuid: Uuid::from_str("574fef8d-7951-3e45-aa29-7167b9d4590a").unwrap(),
            path: String::from("/var/lib/portables"),
            parent_uuid: None,
            received_uuid: None,
        },
        Subvolume {
            uuid: Uuid::from_str("d1bd727c-8a02-bb44-bdd2-bae468651e98").unwrap(),
            path: String::from("/backups/2021-05-04T19:48:42Z_inf_btrfs_test"),
            parent_uuid: None,
            received_uuid: Some(Uuid::from_str("dc4e1039-9241-cd47-9c10-a5d1ce15ba20").unwrap()),
        },
    ];

    assert_eq!(
        crate::utils::get_subvolume_by_path(
            "/backups/2021-05-04T19:48:42Z_inf_btrfs_test",
            &mut subvolumes.iter(),
        )
        .unwrap()
        .uuid,
        subvolumes[2].uuid,
    );
}

#[test]
fn get_common_parent_1() {
    let sl = vec![SnapshotLocal {
        path: "/snapshots/2021-05-02T07:40:32Z_inf_btrfs_test".into(),
        timestamp: Utc.with_ymd_and_hms(2021, 5, 2, 7, 40, 32).unwrap().into(),
        uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a407").unwrap(),
        parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
        suffix: "inf_btrfs_test".into(),
    }];
    let sr = vec![SnapshotRemote {
        path: "/test/path".into(),
        timestamp: Utc.with_ymd_and_hms(2021, 5, 2, 7, 40, 32).unwrap().into(),
        uuid: Uuid::parse_str("11eed410-7829-744e-8288-35c21d278f8e").unwrap(),
        received_uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a407").unwrap(),
        suffix: "inf_btrfs_test".into(),
    }];

    assert_eq!(
        Some(&SnapshotLocal {
            path: "/snapshots/2021-05-02T07:40:32Z_inf_btrfs_test".into(),
            timestamp: Utc.with_ymd_and_hms(2021, 5, 2, 7, 40, 32).unwrap().into(),
            uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a407").unwrap(),
            parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
            suffix: "inf_btrfs_test".into(),
        }),
        crate::utils::get_common_parent(&sl, &sr).unwrap()
    );
}

#[test]
fn get_common_parent_2() {
    let sl = vec![SnapshotLocal {
        path: "/snapshots/2021-05-02T07:40:32Z_inf_btrfs_test".into(),
        timestamp: Utc.with_ymd_and_hms(2021, 5, 2, 7, 40, 32).unwrap().into(),
        uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a408").unwrap(),
        parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
        suffix: "inf_btrfs_test".into(),
    }];
    let sr = vec![SnapshotRemote {
        path: "/test/path".into(),
        timestamp: Utc.with_ymd_and_hms(2021, 5, 2, 7, 40, 32).unwrap().into(),
        uuid: Uuid::parse_str("11eed410-7829-744e-8288-35c21d278f8e").unwrap(),
        received_uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a407").unwrap(),
        suffix: "inf_btrfs_test".into(),
    }];

    assert_eq!(None, crate::utils::get_common_parent(&sl, &sr).unwrap());
}

#[test]
fn get_common_parent_3() {
    let sl = vec![
        SnapshotLocal {
            path: "/snapshots/2021-05-02T07:40:32Z_inf_btrfs_test".into(),
            timestamp: Utc.with_ymd_and_hms(2021, 5, 2, 7, 40, 32).unwrap().into(),
            uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a408").unwrap(),
            parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
            suffix: "inf_btrfs_test".into(),
        },
        SnapshotLocal {
            path: "/snapshots/2021-05-02T07:40:32Z_inf_btrfs_test".into(),
            timestamp: Utc.with_ymd_and_hms(2021, 5, 2, 7, 40, 32).unwrap().into(),
            uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a407").unwrap(),
            parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
            suffix: "inf_btrfs_test".into(),
        },
    ];
    let sr = vec![
        SnapshotRemote {
            path: "/test/path".into(),
            timestamp: Utc.with_ymd_and_hms(2021, 5, 2, 7, 40, 32).unwrap().into(),
            uuid: Uuid::parse_str("11eed410-7829-744e-8288-35c21d278f8e").unwrap(),
            received_uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a407").unwrap(),
            suffix: "inf_btrfs_test".into(),
        },
        SnapshotRemote {
            path: "/test/path".into(),
            timestamp: Utc.with_ymd_and_hms(2021, 5, 2, 7, 40, 32).unwrap().into(),
            uuid: Uuid::parse_str("11eed410-7829-744e-8288-35c21d278f8e").unwrap(),
            received_uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a408").unwrap(),
            suffix: "inf_btrfs_test".into(),
        },
    ];

    assert_eq!(
        Some(&SnapshotLocal {
            path: "/snapshots/2021-05-02T07:40:32Z_inf_btrfs_test".into(),
            timestamp: Utc.with_ymd_and_hms(2021, 5, 2, 7, 40, 32).unwrap().into(),
            uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a408").unwrap(),
            parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
            suffix: "inf_btrfs_test".into(),
        }),
        crate::utils::get_common_parent(&sl, &sr).unwrap()
    );
}

#[test]
fn find_backups_to_be_deleted_1() {
    let current = Utc.with_ymd_and_hms(2020, 1, 4, 10, 0, 0).unwrap();
    let policy = vec![CustomDuration::minutes(15)];
    let backups = vec![
        SnapshotLocal {
            path: "/snapshots/2020-01-02T09:00:00Z_host_subvolume".into(),
            timestamp: Utc.with_ymd_and_hms(2020, 1, 2, 9, 0, 0).unwrap().into(),
            uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a408").unwrap(),
            parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
            suffix: "host_subvolume".into(),
        },
        SnapshotLocal {
            path: "/snapshots/2020-01-02T09:30:00Z_host_subvolume".into(),
            timestamp: Utc.with_ymd_and_hms(2020, 1, 2, 9, 30, 00).unwrap().into(),
            uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a408").unwrap(),
            parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
            suffix: "host_subvolume".into(),
        },
        SnapshotLocal {
            path: "/snapshots/2020-01-03T09:00:00Z_host_subvolume".into(),
            timestamp: Utc.with_ymd_and_hms(2020, 1, 3, 9, 0, 0).unwrap().into(),
            uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a408").unwrap(),
            parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
            suffix: "host_subvolume".into(),
        },
    ];

    let res = crate::utils::find_backups_to_be_deleted(
        &current.into(),
        &policy,
        &backups.iter().map(|e| e as &dyn Snapshot).collect(),
        &String::from("host_subvolume"),
    )
    .unwrap();

    assert_eq!(res.len(), 1);
    assert_eq!(
        res[0].path(),
        "/snapshots/2020-01-02T09:00:00Z_host_subvolume"
    );
}

#[test]
fn find_backups_to_be_deleted_2() {
    let current = Utc.with_ymd_and_hms(2020, 1, 4, 10, 0, 0).unwrap();
    let policy = vec![CustomDuration::days(1), CustomDuration::days(2)];
    let backups = vec![
        SnapshotLocal {
            path: "/snapshots/2020-01-02T09:00:00Z_host_subvolume".into(),
            timestamp: Utc.with_ymd_and_hms(2020, 1, 2, 9, 0, 0).unwrap().into(),
            uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a408").unwrap(),
            parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
            suffix: "host_subvolume".into(),
        },
        SnapshotLocal {
            path: "/snapshots/2020-01-02T09:30:00Z_host_subvolume".into(),
            timestamp: Utc.with_ymd_and_hms(2020, 1, 2, 9, 30, 00).unwrap().into(),
            uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a408").unwrap(),
            parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
            suffix: "host_subvolume".into(),
        },
        SnapshotLocal {
            path: "/snapshots/2020-01-03T09:00:00Z_host_subvolume".into(),
            timestamp: Utc.with_ymd_and_hms(2020, 1, 3, 9, 0, 0).unwrap().into(),
            uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a408").unwrap(),
            parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
            suffix: "host_subvolume".into(),
        },
    ];
    let res = crate::utils::find_backups_to_be_deleted(
        &current.into(),
        &policy,
        &backups.iter().map(|e| e as &dyn Snapshot).collect(),
        &String::from("host_subvolume"),
    )
    .unwrap();

    assert_eq!(res.len(), 0);
}

#[test]
fn find_backups_to_be_deleted_3() {
    let current = Utc.with_ymd_and_hms(2020, 1, 2, 09, 35, 0).unwrap();
    let policy = vec![CustomDuration::minutes(15), CustomDuration::days(1)];
    let backups = vec![
        SnapshotLocal {
            path: "/snapshots/2019-12-31T09:00:00Z_host_subvolume".into(),
            timestamp: Utc.with_ymd_and_hms(2019, 12, 31, 9, 0, 0).unwrap().into(),
            uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a408").unwrap(),
            parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
            suffix: "host_subvolume".into(),
        },
        SnapshotLocal {
            path: "/snapshots/2020-01-01T09:00:00Z_host_subvolume".into(),
            timestamp: Utc.with_ymd_and_hms(2020, 1, 1, 9, 0, 0).unwrap().into(),
            uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a408").unwrap(),
            parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
            suffix: "host_subvolume".into(),
        },
        SnapshotLocal {
            path: "/snapshots/2020-01-02T09:00:00Z_host_subvolume".into(),
            timestamp: Utc.with_ymd_and_hms(2020, 1, 2, 9, 0, 0).unwrap().into(),
            uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a408").unwrap(),
            parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
            suffix: "host_subvolume".into(),
        },
        SnapshotLocal {
            path: "/snapshots/2020-01-02T09:12:00Z_host2_subvolume".into(),
            timestamp: Utc.with_ymd_and_hms(2020, 1, 2, 9, 12, 0).unwrap().into(),
            uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a408").unwrap(),
            parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
            suffix: "host2_subvolume".into(),
        },
        SnapshotLocal {
            path: "/snapshots/2020-01-02T09:15:00Z_host_subvolume".into(),
            timestamp: Utc.with_ymd_and_hms(2020, 1, 2, 9, 15, 0).unwrap().into(),
            uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a408").unwrap(),
            parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
            suffix: "host_subvolume".into(),
        },
        SnapshotLocal {
            path: "/snapshots/2020-01-02T09:07:00Z_host_subvolume".into(),
            timestamp: Utc.with_ymd_and_hms(2020, 1, 2, 9, 7, 0).unwrap().into(),
            uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a408").unwrap(),
            parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
            suffix: "host_subvolume".into(),
        },
        SnapshotLocal {
            path: "/snapshots/2020-01-02T09:30:00Z_host_subvolume".into(),
            timestamp: Utc.with_ymd_and_hms(2020, 1, 2, 9, 30, 0).unwrap().into(),
            uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a408").unwrap(),
            parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
            suffix: "host_subvolume".into(),
        },
    ];
    let res = crate::utils::find_backups_to_be_deleted(
        &current.into(),
        &policy,
        &backups.iter().map(|e| e as &dyn Snapshot).collect(),
        &String::from("host_subvolume"),
    )
    .unwrap();

    assert_eq!(res.len(), 3);
    assert_eq!(
        res[0].path(),
        "/snapshots/2020-01-02T09:15:00Z_host_subvolume"
    );
    assert_eq!(
        res[1].path(),
        "/snapshots/2020-01-02T09:07:00Z_host_subvolume"
    );
    assert_eq!(
        res[2].path(),
        "/snapshots/2019-12-31T09:00:00Z_host_subvolume"
    );
}

#[test]
fn find_backups_to_be_deleted_4() {
    let current = Utc.with_ymd_and_hms(2020, 1, 2, 09, 35, 0).unwrap();
    let policy: Vec<CustomDuration> = Vec::new();
    let backups = vec![
        SnapshotLocal {
            path: "/snapshots/2019-12-31T09:00:00Z_host_subvolume".into(),
            timestamp: Utc.with_ymd_and_hms(2019, 12, 31, 9, 0, 0).unwrap().into(),
            uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a408").unwrap(),
            parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
            suffix: "host_subvolume".into(),
        },
        SnapshotLocal {
            path: "/snapshots/2020-01-01T09:00:00Z_host_subvolume".into(),
            timestamp: Utc.with_ymd_and_hms(2020, 1, 1, 9, 0, 0).unwrap().into(),
            uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a408").unwrap(),
            parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
            suffix: "host_subvolume".into(),
        },
        SnapshotLocal {
            path: "/snapshots/2020-01-02T09:00:00Z_host_subvolume".into(),
            timestamp: Utc.with_ymd_and_hms(2020, 1, 2, 9, 0, 0).unwrap().into(),
            uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a408").unwrap(),
            parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
            suffix: "host_subvolume".into(),
        },
        SnapshotLocal {
            path: "/snapshots/2020-01-02T09:12:00Z_host2_subvolume".into(),
            timestamp: Utc.with_ymd_and_hms(2020, 1, 2, 9, 12, 0).unwrap().into(),
            uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a408").unwrap(),
            parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
            suffix: "host2_subvolume".into(),
        },
        SnapshotLocal {
            path: "/snapshots/2020-01-02T09:15:00Z_host_subvolume".into(),
            timestamp: Utc.with_ymd_and_hms(2020, 1, 2, 9, 15, 0).unwrap().into(),
            uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a408").unwrap(),
            parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
            suffix: "host_subvolume".into(),
        },
        SnapshotLocal {
            path: "/snapshots/2020-01-02T09:07:00Z_host_subvolume".into(),
            timestamp: Utc.with_ymd_and_hms(2020, 1, 2, 9, 7, 0).unwrap().into(),
            uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a408").unwrap(),
            parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
            suffix: "host_subvolume".into(),
        },
        SnapshotLocal {
            path: "/snapshots/2020-01-02T09:30:00Z_host_subvolume".into(),
            timestamp: Utc.with_ymd_and_hms(2020, 1, 2, 9, 30, 0).unwrap().into(),
            uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a408").unwrap(),
            parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
            suffix: "host_subvolume".into(),
        },
    ];
    let res = crate::utils::find_backups_to_be_deleted(
        &current.into(),
        &policy,
        &backups.iter().map(|e| e as &dyn Snapshot).collect(),
        &String::from("host_subvolume"),
    )
    .unwrap();
    assert_eq!(res.len(), 5);
    assert_eq!(
        res[0].path(),
        "/snapshots/2020-01-02T09:15:00Z_host_subvolume"
    );
    assert_eq!(
        res[1].path(),
        "/snapshots/2020-01-02T09:07:00Z_host_subvolume"
    );
    assert_eq!(
        res[2].path(),
        "/snapshots/2020-01-02T09:00:00Z_host_subvolume"
    );
    assert_eq!(
        res[3].path(),
        "/snapshots/2020-01-01T09:00:00Z_host_subvolume"
    );
    assert_eq!(
        res[4].path(),
        "/snapshots/2019-12-31T09:00:00Z_host_subvolume"
    );
}

#[test]
fn get_btrfs_mount_information() {
    let command_output = r"/dev/mapper/data on /data type btrfs (rw,relatime,space_cache=v2,subvolid=5,subvol=/)
/dev/mapper/luks-0f3f6c5e-621a-40d8-8be8-c372eaf2d616 on / type btrfs (rw,relatime,seclabel,compress=zstd:1,ssd,space_cache,subvolid=11858,subvol=/root)
/dev/mapper/luks-0f3f6c5e-621a-40d8-8be8-c372eaf2d616 on /home type btrfs (rw,relatime,seclabel,compress=zstd:1,ssd,space_cache,subvolid=256,subvol=/home)
/dev/mapper/luks-0f3f6c5e-621a-40d8-8be8-c372eaf2d616 on /var/lib/docker/btrfs type btrfs (rw,relatime,seclabel,compress=zstd:1,ssd,space_cache,subvolid=11858,subvol=/root)
";
    let mi = super::get_btrfs_mount_information(command_output).unwrap();

    let check = vec![
        MountInformation {
            device: "/dev/mapper/data".to_string(),
            fs_type: "btrfs".to_string(),
            mount_point: "/data".to_string(),
            properties: vec![
                ("rw".to_string(), None),
                ("relatime".to_string(), None),
                ("space_cache".to_string(), Some("v2".to_string())),
                ("subvolid".to_string(), Some("5".to_string())),
                ("subvol".to_string(), Some("/".to_string())),
            ]
            .iter()
            .cloned()
            .collect(),
        },
        MountInformation {
            device: "/dev/mapper/luks-0f3f6c5e-621a-40d8-8be8-c372eaf2d616".to_string(),
            fs_type: "btrfs".to_string(),
            mount_point: "/".to_string(),
            properties: vec![
                ("rw".to_string(), None),
                ("seclabel".to_string(), None),
                ("compress".to_string(), Some("zstd:1".to_string())),
                ("ssd".to_string(), None),
                ("space_cache".to_string(), None),
                ("subvolid".to_string(), Some("11858".to_string())),
                ("subvol".to_string(), Some("/root".to_string())),
                ("relatime".to_string(), None),
            ]
            .iter()
            .cloned()
            .collect(),
        },
        MountInformation {
            device: "/dev/mapper/luks-0f3f6c5e-621a-40d8-8be8-c372eaf2d616".to_string(),
            fs_type: "btrfs".to_string(),
            mount_point: "/home".to_string(),
            properties: vec![
                ("rw".to_string(), None),
                ("seclabel".to_string(), None),
                ("compress".to_string(), Some("zstd:1".to_string())),
                ("ssd".to_string(), None),
                ("space_cache".to_string(), None),
                ("subvolid".to_string(), Some("256".to_string())),
                ("subvol".to_string(), Some("/home".to_string())),
                ("relatime".to_string(), None),
            ]
            .iter()
            .cloned()
            .collect(),
        },
        MountInformation {
            device: "/dev/mapper/luks-0f3f6c5e-621a-40d8-8be8-c372eaf2d616".to_string(),
            fs_type: "btrfs".to_string(),
            mount_point: "/var/lib/docker/btrfs".to_string(),
            properties: vec![
                ("rw".to_string(), None),
                ("seclabel".to_string(), None),
                ("compress".to_string(), Some("zstd:1".to_string())),
                ("ssd".to_string(), None),
                ("space_cache".to_string(), None),
                ("subvolid".to_string(), Some("11858".to_string())),
                ("subvol".to_string(), Some("/root".to_string())),
                ("relatime".to_string(), None),
            ]
            .iter()
            .cloned()
            .collect(),
        },
    ];

    assert_eq!(mi, check);
}
