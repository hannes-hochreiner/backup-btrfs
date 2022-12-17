use uuid::Uuid;

/// # SubvolumeInfo
///
/// Information obtained from the `btrfs subvolume show <path>` command.
///
/// * `fs_path` - filesystem path
/// * `btrfs_path` - btrfs path
/// * `uuid` - btrfs subvolume uuid
#[derive(Debug, PartialEq, Clone)]
pub struct SubvolumeInfo {
    pub fs_path: String,
    pub btrfs_path: String,
    pub uuid: Uuid,
}
