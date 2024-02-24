use crate::backup_error::BackupError;
use exec_rs::{Context, Exec};
use std::path::Path;

pub trait CommandDeleteSubvolume {
    /// Delete a subvolume
    ///
    /// Executes `sudo btrfs subvolume delete <subvolume_path>`.
    /// As a precaution, the subvolumes "home", "/home", "root", and "/" cannot be deleted.
    ///
    /// * `subvolume_path` - absolute path of the snapshot to be deleted
    /// * `context` - context in which to execute the command
    ///
    fn delete_subvolume(&mut self, subvolume: &str, context: &Context) -> Result<(), BackupError>;
}

impl<T: Exec> CommandDeleteSubvolume for super::Commander<T> {
    fn delete_subvolume(&mut self, subvolume: &str, context: &Context) -> Result<(), BackupError> {
        let subvolume = match context {
            Context::Local { user: _ } => {
                let subvolume_path = Path::new(subvolume).canonicalize()?;
                let subvolume =
                    subvolume_path
                        .as_path()
                        .to_str()
                        .ok_or(BackupError::DeleteSubvolume(String::from(
                            "cannot canonicalize subvolume path",
                        )))?;
                subvolume.to_owned()
            }
            Context::Remote { config: _ } => subvolume.to_owned(),
        };

        if vec!["home", "/home", "root", "/"].contains(&subvolume.as_str()) {
            return Err(BackupError::DeleteSubvolume(String::from("subvolume cannot be deleted as its name is on the restricted names list (home, /home, /, root)")));
        }

        log::info!("subvolume path: \"{}\"", subvolume);

        self.exec.exec(
            "sudo",
            &["btrfs", "subvolume", "delete", &subvolume],
            Some(context),
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use exec_rs::MockExec;

    use crate::commands::Commander;

    use super::*;

    #[test]
    fn delete_subvolume() {
        let mut mock = MockExec::new();

        mock.expect_exec()
            .once()
            .withf(|command, args, _| {
                assert_eq!(command, "sudo");
                assert_eq!(args, &["btrfs", "subvolume", "delete", "/tmp"]);
                true
            })
            .returning(|_, _, _| Ok(String::new()));

        let mut commands = Commander::new_with_exec(mock);

        assert!(commands
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
        let mock = MockExec::new();
        let mut commands = Commander::new_with_exec(mock);

        assert!(commands
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
        let mock = MockExec::new();
        let mut commands = Commander::new_with_exec(mock);

        assert!(commands
            .delete_subvolume(
                &String::from("/"),
                &Context::Local {
                    user: "test".into()
                }
            )
            .is_err());
    }
}
