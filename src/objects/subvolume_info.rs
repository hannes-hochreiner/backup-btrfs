use uuid::Uuid;

#[derive(Debug, PartialEq, Clone)]
pub struct SubvolumeInfo {
    pub fs_path: String,
    pub btrfs_path: String,
    pub uuid: Uuid,
}
