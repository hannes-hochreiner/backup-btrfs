use crate::{backup_error::BackupError, objects::Subvolume};
use exec_rs::{Context, Exec};
use std::str::FromStr;
use uuid::Uuid;

pub trait CommandGetSubvolumes {
    /// Get subvolumes
    ///
    /// * `subvolume_path` - path of the btrfs subvolume
    /// * `exec` - command executor
    /// * `context` - context in which to execute the command
    ///
    fn get_subvolumes(
        &mut self,
        subvolume_path: &str,
        context: &Context,
    ) -> Result<Vec<Subvolume>, BackupError>;
}

impl<T: Exec> CommandGetSubvolumes for super::Commander<T> {
    fn get_subvolumes(
        &mut self,
        subvolume_path: &str,
        context: &Context,
    ) -> Result<Vec<Subvolume>, BackupError> {
        let command_output = self.exec.exec(
            "sudo",
            &[
                "btrfs",
                "subvolume",
                "list",
                "-tupqRo",
                "--sort=rootid",
                subvolume_path,
            ],
            Some(context),
        )?;

        let mut subvolumes: Vec<Subvolume> = Vec::new();
        let mut lines = command_output.split("\n");

        if lines
            .next()
            .ok_or(BackupError::SubvolumeParsing(String::from(
                "could not find header line",
            )))?
            .split_ascii_whitespace()
            .collect::<Vec<&str>>()
            != vec![
                "ID",
                "gen",
                "parent",
                "top",
                "level",
                "parent_uuid",
                "received_uuid",
                "uuid",
                "path",
            ]
        {
            return Err(BackupError::SubvolumeParsing(String::from(
                "unexpected header line",
            )));
        }

        for line in lines.skip(1).into_iter() {
            let tokens: Vec<&str> = line.split_ascii_whitespace().collect();

            if tokens.len() != 8 {
                continue;
            }

            subvolumes.push(Subvolume {
                btrfs_path: match tokens[7].starts_with('/') {
                    true => tokens[7].into(),
                    false => format!("/{}", tokens[7]),
                },
                uuid: Uuid::from_str(tokens[6])?,
                parent_uuid: match Uuid::from_str(tokens[4]) {
                    Ok(pu) => Some(pu),
                    Err(_) => None,
                },
                received_uuid: match Uuid::from_str(tokens[5]) {
                    Ok(ru) => Some(ru),
                    Err(_) => None,
                },
            });
        }

        Ok(subvolumes)
    }
}

#[cfg(test)]
mod test {
    use exec_rs::MockExec;

    use crate::commands::Commander;

    use super::*;
    #[test]
    fn get_local_subvolumes() {
        let ctx = Context::Local {
            user: "test".into(),
        };
        let mut mock = MockExec::new();

        mock.expect_exec()
            .once()
            .withf(|command, args, _| {
                assert_eq!(command, "sudo");
                assert_eq!(args, &["btrfs", "subvolume", "list", "-tupqRo", "--sort=rootid", "/"]);
                true
            })
            .returning(|_, _, _| Ok(String::from(r#"ID      gen     parent  top level       parent_uuid     received_uuid   uuid    path
--      ---     ------  ---------       -----------     -------------   ----    ----
256     119496  5       5               -                                       -                                       11eed410-7829-744e-8288-35c21d278f8e    home
359     119496  5       5               -                                       -                                       32c672fa-d3ce-0b4e-8eaa-ab9205f377ca    root
360     119446  359     359             -                                       -                                       5f0b151b-52e4-4445-aa94-d07056733a1f    opt/btrfs_test
367     118687  359     359             5f0b151b-52e4-4445-aa94-d07056733a1f    -                                       7f305e3e-851b-974b-a476-e2f206e7a407    snapshots/2021-05-02T07:40:32Z_inf_btrfs_test
370     119446  359     359             5f0b151b-52e4-4445-aa94-d07056733a1f    -                                       1bd1da76-b61f-db41-a2d2-c3474a31f38f    snapshots/2021-05-02T13:38:49Z_inf_btrfs_test
"#)));

        let mut commander = Commander::new_with_exec(mock);

        assert_eq!(
            commander.get_subvolumes("/", &ctx).unwrap(),
            vec![
                Subvolume {
                    uuid: Uuid::from_str("11eed410-7829-744e-8288-35c21d278f8e").unwrap(),
                    btrfs_path: String::from("/home"),
                    parent_uuid: None,
                    received_uuid: None
                },
                Subvolume {
                    uuid: Uuid::from_str("32c672fa-d3ce-0b4e-8eaa-ab9205f377ca").unwrap(),
                    btrfs_path: String::from("/root"),
                    parent_uuid: None,
                    received_uuid: None
                },
                Subvolume {
                    uuid: Uuid::from_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap(),
                    btrfs_path: String::from("/opt/btrfs_test"),
                    parent_uuid: None,
                    received_uuid: None
                },
                Subvolume {
                    uuid: Uuid::from_str("7f305e3e-851b-974b-a476-e2f206e7a407").unwrap(),
                    btrfs_path: String::from("/snapshots/2021-05-02T07:40:32Z_inf_btrfs_test"),
                    parent_uuid: Some(
                        Uuid::from_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap()
                    ),
                    received_uuid: None
                },
                Subvolume {
                    uuid: Uuid::from_str("1bd1da76-b61f-db41-a2d2-c3474a31f38f").unwrap(),
                    btrfs_path: String::from("/snapshots/2021-05-02T13:38:49Z_inf_btrfs_test"),
                    parent_uuid: Some(
                        Uuid::from_str("5f0b151b-52e4-4445-aa94-d07056733a1f").unwrap()
                    ),
                    received_uuid: None
                },
            ]
        );
    }

    #[test]
    fn get_remote_subvolumes() {
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
                assert_eq!(
                    args,
                    &[
                        "btrfs",
                        "subvolume",
                        "list",
                        "-tupqRo",
                        "--sort=rootid",
                        "/"
                    ]
                );
                true
            })
            .returning(|_,_,_| Ok(String::from(r#"ID      gen     parent  top level       parent_uuid     received_uuid   uuid    path
--      ---     ------  ---------       -----------     -------------   ----    ----
256     10789   5       5               -                                       -                                       0b5cc138-af8e-2744-be4f-bdede1b509ef    root
259     9051    256     256             -                                       -                                       574fef8d-7951-3e45-aa29-7167b9d4590a    var/lib/portables
270     4965    256     256             -                                       dc4e1039-9241-cd47-9c10-a5d1ce15ba20    d1bd727c-8a02-bb44-bdd2-bae468651e98    backups/2021-05-04T19:48:42Z_inf_btrfs_test
328     7505    256     256             19391f90-9007-3e4b-b757-6e5d2421b9bd    53bb5cfa-f45e-d147-9407-006271609062    54b52286-8265-9444-8603-214e7e0533e0    backups/2021-05-10T06:14:04Z_inf_btrfs_test
"#)));

        let mut commander = Commander::new_with_exec(mock);

        assert_eq!(
            commander.get_subvolumes("/", &ctx).unwrap(),
            vec![
                Subvolume {
                    uuid: Uuid::from_str("0b5cc138-af8e-2744-be4f-bdede1b509ef").unwrap(),
                    btrfs_path: String::from("/root"),
                    parent_uuid: None,
                    received_uuid: None
                },
                Subvolume {
                    uuid: Uuid::from_str("574fef8d-7951-3e45-aa29-7167b9d4590a").unwrap(),
                    btrfs_path: String::from("/var/lib/portables"),
                    parent_uuid: None,
                    received_uuid: None
                },
                Subvolume {
                    uuid: Uuid::from_str("d1bd727c-8a02-bb44-bdd2-bae468651e98").unwrap(),
                    btrfs_path: String::from("/backups/2021-05-04T19:48:42Z_inf_btrfs_test"),
                    parent_uuid: None,
                    received_uuid: Some(
                        Uuid::from_str("dc4e1039-9241-cd47-9c10-a5d1ce15ba20").unwrap()
                    )
                },
                Subvolume {
                    uuid: Uuid::from_str("54b52286-8265-9444-8603-214e7e0533e0").unwrap(),
                    btrfs_path: String::from("/backups/2021-05-10T06:14:04Z_inf_btrfs_test"),
                    parent_uuid: Some(
                        Uuid::from_str("19391f90-9007-3e4b-b757-6e5d2421b9bd").unwrap()
                    ),
                    received_uuid: Some(
                        Uuid::from_str("53bb5cfa-f45e-d147-9407-006271609062").unwrap()
                    )
                },
            ]
        );
    }
}
