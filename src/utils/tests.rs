use chrono::{TimeZone, Utc};
use uuid::Uuid;
use crate::{
    utils::{SnapshotLocal, SnapshotRemote}
};

#[test]
fn get_subvolume_by_path() {
    todo!()
}

#[test]
fn get_common_parent_1() {
    let sl = vec![
        SnapshotLocal {
            path: "/snapshots/2021-05-02T07:40:32Z_inf_btrfs_test".into(),
            timestamp: Utc.ymd(2021, 5, 2).and_hms(7, 40, 32).into(),
            uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a407").unwrap(),
            parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
            suffix: "inf_btrfs_test".into(),
        },
    ];
    let sr = vec![
        SnapshotRemote {
            path: "/test/path".into(),
            timestamp: Utc.ymd(2021, 5, 2).and_hms(7, 40, 32).into(),
            uuid: Uuid::parse_str("11eed410-7829-744e-8288-35c21d278f8e").unwrap(),
            received_uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a407").unwrap(),
            suffix: "inf_btrfs_test".into(),
        },
    ];

    assert_eq!(Some(SnapshotLocal {
        path: "/snapshots/2021-05-02T07:40:32Z_inf_btrfs_test".into(),
        timestamp: Utc.ymd(2021, 5, 2).and_hms(7, 40, 32).into(),
        uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a407").unwrap(),
        parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
        suffix: "inf_btrfs_test".into(),
    }), crate::utils::get_common_parent(&sl, &sr).unwrap());
}

#[test]
fn get_common_parent_2() {
    let sl = vec![
        SnapshotLocal {
            path: "/snapshots/2021-05-02T07:40:32Z_inf_btrfs_test".into(),
            timestamp: Utc.ymd(2021, 5, 2).and_hms(7, 40, 32).into(),
            uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a408").unwrap(),
            parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
            suffix: "inf_btrfs_test".into(),
        },
    ];
    let sr = vec![
        SnapshotRemote {
            path: "/test/path".into(),
            timestamp: Utc.ymd(2021, 5, 2).and_hms(7, 40, 32).into(),
            uuid: Uuid::parse_str("11eed410-7829-744e-8288-35c21d278f8e").unwrap(),
            received_uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a407").unwrap(),
            suffix: "inf_btrfs_test".into(),
        },
    ];

    assert_eq!(None, crate::utils::get_common_parent(&sl, &sr).unwrap());
}

#[test]
fn get_common_parent_3() {
    let sl = vec![
        SnapshotLocal {
            path: "/snapshots/2021-05-02T07:40:32Z_inf_btrfs_test".into(),
            timestamp: Utc.ymd(2021, 5, 2).and_hms(7, 40, 32).into(),
            uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a408").unwrap(),
            parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
            suffix: "inf_btrfs_test".into(),
        },
        SnapshotLocal {
            path: "/snapshots/2021-05-02T07:40:32Z_inf_btrfs_test".into(),
            timestamp: Utc.ymd(2021, 5, 2).and_hms(7, 40, 32).into(),
            uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a407").unwrap(),
            parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
            suffix: "inf_btrfs_test".into(),
        },
    ];
    let sr = vec![
        SnapshotRemote {
            path: "/test/path".into(),
            timestamp: Utc.ymd(2021, 5, 2).and_hms(7, 40, 32).into(),
            uuid: Uuid::parse_str("11eed410-7829-744e-8288-35c21d278f8e").unwrap(),
            received_uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a407").unwrap(),
            suffix: "inf_btrfs_test".into(),
        },
        SnapshotRemote {
            path: "/test/path".into(),
            timestamp: Utc.ymd(2021, 5, 2).and_hms(7, 40, 32).into(),
            uuid: Uuid::parse_str("11eed410-7829-744e-8288-35c21d278f8e").unwrap(),
            received_uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a408").unwrap(),
            suffix: "inf_btrfs_test".into(),
        },
    ];

    assert_eq!(Some(SnapshotLocal {
        path: "/snapshots/2021-05-02T07:40:32Z_inf_btrfs_test".into(),
        timestamp: Utc.ymd(2021, 5, 2).and_hms(7, 40, 32).into(),
        uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a408").unwrap(),
        parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
        suffix: "inf_btrfs_test".into(),
    }), crate::utils::get_common_parent(&sl, &sr).unwrap());
}

//     #[test]
//     fn find_backups_to_be_deleted_1() {
//         let current = Utc.ymd(2020, 1, 4).and_hms(10, 0, 0);
//         let policy = vec![CustomDuration::minutes(15)];
//         let backups = vec![
//             String::from("/snapshots/2020-01-02T09:00:00Z_host_subvolume"),
//             String::from("/snapshots/2020-01-02T09:30:00Z_host_subvolume"),
//             String::from("/snapshots/2020-01-03T09:00:00Z_host_subvolume"),
//         ];
//         let exp = vec![
//             String::from("/snapshots/2020-01-02T09:00:00Z_host_subvolume"),
//         ];
//         assert_eq!(exp, utils::find_backups_to_be_deleted(&current.into(), &policy, &backups, &String::from("host_subvolume")).unwrap());
//     }

//     #[test]
//     fn find_backups_to_be_deleted_2() {
//         let current = Utc.ymd(2020, 1, 4).and_hms(10, 0, 0);
//         let policy = vec![CustomDuration::days(1), CustomDuration::days(2)];
//         let backups = vec![
//             String::from("/snapshots/2020-01-02T09:00:00Z_host_subvolume"),
//             String::from("/snapshots/2020-01-02T09:30:00Z_host_subvolume"),
//             String::from("/snapshots/2020-01-03T09:00:00Z_host_subvolume"),
//         ];
//         let exp: Vec<String> = Vec::new();
//         assert_eq!(exp, utils::find_backups_to_be_deleted(&current.into(), &policy, &backups, &String::from("host_subvolume")).unwrap());
//     }

//     #[test]
//     fn find_backups_to_be_deleted_3() {
//         let current = Utc.ymd(2020, 1, 2).and_hms(09, 35, 0);
//         let policy = vec![CustomDuration::minutes(15), CustomDuration::days(1)];
//         let backups = vec![
//             String::from("/snapshots/2019-12-31T09:00:00Z_host_subvolume"),
//             String::from("/snapshots/2020-01-01T09:00:00Z_host_subvolume"),
//             String::from("/snapshots/2020-01-02T09:00:00Z_host_subvolume"),
//             String::from("/snapshots/2020-01-02T09:12:00Z_host2_subvolume"),
//             String::from("/snapshots/2020-01-02T09:15:00Z_host_subvolume"),
//             String::from("/snapshots/2020-01-02T09:07:00Z_host_subvolume"),
//             String::from("/snapshots/2020-01-02T09:30:00Z_host_subvolume"),
//         ];
//         let exp = vec![
//             String::from("/snapshots/2020-01-02T09:07:00Z_host_subvolume"),
//             String::from("/snapshots/2020-01-02T09:15:00Z_host_subvolume"),
//             String::from("/snapshots/2019-12-31T09:00:00Z_host_subvolume"),
//         ];
//         assert_eq!(exp, utils::find_backups_to_be_deleted(&current.into(), &policy, &backups, &String::from("host_subvolume")).unwrap());
//     }

//     #[test]
//     fn find_backups_to_be_deleted_4() {
//         let current = Utc.ymd(2020, 1, 2).and_hms(09, 35, 0);
//         let policy: Vec<CustomDuration> = Vec::new();
//         let backups = vec![
//             String::from("/snapshots/2019-12-31T09:00:00Z_host_subvolume"),
//             String::from("/snapshots/2020-01-01T09:00:00Z_host_subvolume"),
//             String::from("/snapshots/2020-01-02T09:00:00Z_host_subvolume"),
//             String::from("/snapshots/2020-01-02T09:12:00Z_host2_subvolume"),
//             String::from("/snapshots/2020-01-02T09:15:00Z_host_subvolume"),
//             String::from("/snapshots/2020-01-02T09:07:00Z_host_subvolume"),
//             String::from("/snapshots/2020-01-02T09:30:00Z_host_subvolume"),
//         ];
//         let exp = vec![
//             String::from("/snapshots/2020-01-02T09:07:00Z_host_subvolume"),
//             String::from("/snapshots/2020-01-02T09:15:00Z_host_subvolume"),
//             String::from("/snapshots/2020-01-02T09:12:00Z_host2_subvolume"),
//             String::from("/snapshots/2020-01-02T09:00:00Z_host_subvolume"),
//             String::from("/snapshots/2020-01-01T09:00:00Z_host_subvolume"),
//             String::from("/snapshots/2019-12-31T09:00:00Z_host_subvolume"),
//         ];
//         assert_eq!(exp, utils::find_backups_to_be_deleted(&current.into(), &policy, &backups, &String::from("host_subvolume")).unwrap());
//     }

//     #[test]
//     fn get_local_snapshots() {
//         let input = r#"ID      gen     parent  top level       parent_uuid     received_uuid   uuid    path
// --      ---     ------  ---------       -----------     -------------   ----    ----
// 256     119496  5       5               -                                       -                                       11eed410-7829-744e-8288-35c21d278f8e    home
// 359     119496  5       5               -                                       -                                       32c672fa-d3ce-0b4e-8eaa-ab9205f377ca    root
// 360     119446  359     359             -                                       -                                       5f0b151b-52e4-4445-aa94-d07056733a1f    opt/btrfs_test
// 367     118687  359     359             5f0b151b-52e4-4445-aa94-d07056733a1f    -                                       7f305e3e-851b-974b-a476-e2f206e7a407    snapshots/2021-05-02T07:40:32Z_inf_btrfs_test
// 370     119446  359     359             5f0b151b-52e4-4445-aa94-d07056733a1f    -                                       1bd1da76-b61f-db41-a2d2-c3474a31f38f    snapshots/2021-05-02T13:38:49Z_inf_btrfs_test
// "#;

//         match utils::get_local_snapshots("/opt/btrfs_test", input) {
//             Err(e) => println!("{}", e),
//             Ok(_) => {}
//         }

//         assert_eq!(vec![
//             SnapshotLocal { path: "/snapshots/2021-05-02T07:40:32Z_inf_btrfs_test".into(), timestamp: Utc.ymd(2021, 5, 2).and_hms(7, 40, 32).into(), uuid: Uuid::parse_str("7f305e3e-851b-974b-a476-e2f206e7a407").unwrap(), parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap() },
//             SnapshotLocal { path: "/snapshots/2021-05-02T13:38:49Z_inf_btrfs_test".into(), timestamp: Utc.ymd(2021, 5, 2).and_hms(13, 38, 49).into(), uuid: Uuid::parse_str("1bd1da76-b61f-db41-a2d2-c3474a31f38f").unwrap(), parent_uuid: Uuid::parse_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap() },
//         ], utils::get_local_snapshots("/opt/btrfs_test", input).unwrap());
//     }

//     #[test]
//     fn get_timestamp_suffix_from_snapshot_path() {
//         let (timestamp, suffix) = utils::get_timestamp_suffix_from_snapshot_path(&String::from("/opt/snapshots/2021-05-12T04:23:12Z_exo_btrfs_test")).unwrap();

//         assert_eq!(Utc.ymd(2021, 05, 12).and_hms(4, 23, 12), timestamp);
//         assert_eq!(String::from("exo_btrfs_test"), suffix);
//     }
