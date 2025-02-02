mod get_mount_information;
mod get_subvolumes;
use exec_rs::{CommandExec, Exec};
mod delete_subvolume;
mod get_subvolume_info;
mod read_link;
mod send_snapshot;
mod snapshot_subvolume;

pub trait Commands:
    get_mount_information::CommandGetMountInformation
    + snapshot_subvolume::CommandSnapshotSubvolume
    + get_subvolumes::CommandGetSubvolumes
    + get_subvolume_info::CommandGetSubvolumeInfo
    + delete_subvolume::CommandDeleteSubvolume
    + send_snapshot::CommandSendSnapshot
    + read_link::CommandReadLink
{
}

#[cfg(test)]
mockall::mock! {
    pub Commander {}
    impl get_mount_information::CommandGetMountInformation for Commander {
        fn get_mount_information(
            &mut self,
            context: &exec_rs::Context,
        ) -> Result<Vec<crate::objects::MountInformation>, crate::backup_error::BackupError>;
    }
    impl snapshot_subvolume::CommandSnapshotSubvolume for Commander {
        fn snapshot_subvolume(
            &mut self,
            subvolume_path: &str,
            snapshot_path: &str,
            snapshot_suffix: &str,
            timestamp: &chrono::DateTime<chrono::Utc>,
            context: &exec_rs::Context,
        ) -> Result<String, crate::backup_error::BackupError>;
    }
    impl get_subvolumes::CommandGetSubvolumes for Commander {
        fn get_subvolumes(
            &mut self,
            subvolume_path: &str,
            context: &exec_rs::Context,
        ) -> Result<Vec<crate::objects::Subvolume>, crate::backup_error::BackupError>;
    }
    impl get_subvolume_info::CommandGetSubvolumeInfo for Commander {
        fn get_subvolume_info(
            &mut self,
            subvolume_path: &str,
            context: &exec_rs::Context,
        ) -> Result<crate::objects::SubvolumeInfo, crate::backup_error::BackupError>;
    }
    impl delete_subvolume::CommandDeleteSubvolume for Commander {
        fn delete_subvolume(&mut self, subvolume: &str, context: &exec_rs::Context) -> Result<(), crate::backup_error::BackupError>;
    }
    impl send_snapshot::CommandSendSnapshot for Commander {
        fn send_snapshot<'a>(
            &mut self,
            local_snapshot: &crate::objects::SubvolumeInfo,
            common_parent: Option<&'a crate::objects::SubvolumeInfo>,
            context_local: &exec_rs::Context,
            backup_path: &str,
            context_remote: &exec_rs::Context,
        ) -> Result<(), crate::backup_error::BackupError>;
    }
    impl read_link::CommandReadLink for Commander {
        fn read_link(&mut self, path: &str, context: &exec_rs::Context) -> Result<Vec<String>, crate::backup_error::BackupError>;
    }
    impl Commands for Commander {}
}

pub struct Commander<T: Exec> {
    exec: T,
}

#[cfg(test)]
impl<T: Exec> Commander<T> {
    fn new_with_exec(exec: T) -> Self {
        Self { exec }
    }
}

impl Default for Commander<CommandExec> {
    fn default() -> Self {
        Self {
            exec: CommandExec {},
        }
    }
}

impl<T: Exec> Commands for Commander<T> {}
