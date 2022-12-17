use std::collections::HashMap;

/// # MountInformation
///
/// * `device` - device string
/// * `root` - root of the filesystem at the time of mounting
/// * `mount_point` - mount point
/// * `fs_type` - type of filesystem
/// * `properties` - additional properties
///
/// References:
/// * https://www.kernel.org/doc/Documentation/filesystems/proc.txt
#[derive(Debug, PartialEq)]
pub struct MountInformation {
    pub device: String,
    pub root: String,
    pub mount_point: String,
    pub fs_type: String,
    pub properties: HashMap<String, Option<String>>,
}
