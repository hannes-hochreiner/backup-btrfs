use uuid::Uuid;

#[derive(Debug, PartialEq, Clone)]
pub struct Subvolume {
    pub btrfs_path: String,
    pub uuid: Uuid,
    pub parent_uuid: Option<Uuid>,
    pub received_uuid: Option<Uuid>,
}
