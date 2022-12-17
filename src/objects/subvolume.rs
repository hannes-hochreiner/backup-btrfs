use uuid::Uuid;

/// # Subvolume
///
/// Information obtained from the `btrfs subvolume list` command
///
/// * `btrfs_path` - btrfs path
/// * `uuid` - btrfs subvolume uuid
/// * `parent_uuid` - btrfs uuid of the parent of the subvolume
/// * `received_uuid` - btrfs uuid of the subvolume, which was sent
#[derive(Debug, PartialEq, Clone)]
pub struct Subvolume {
    pub btrfs_path: String,
    pub uuid: Uuid,
    pub parent_uuid: Option<Uuid>,
    pub received_uuid: Option<Uuid>,
}
