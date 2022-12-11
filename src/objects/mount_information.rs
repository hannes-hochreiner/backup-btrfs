use std::collections::HashMap;

use crate::custom_error::CustomError;

#[derive(Debug, PartialEq)]
pub struct MountInformation {
    device: String,
    mount_point: String,
    fs_type: String,
    properties: HashMap<String, Option<String>>,
}

impl MountInformation {
    /// Get the mount information of all btrfs mounts.
    ///
    /// * `command_output` - output of the command `mount -t btrfs`
    ///
    /// dev/mapper/data on /data type btrfs (rw,relatime,space_cache=v2,subvolid=5,subvol=/)
    pub fn new_with_command_output(
        command_output: &str,
    ) -> Result<Vec<MountInformation>, CustomError> {
        command_output
            .lines()
            .filter(|&l| !l.is_empty())
            .map(|l| {
                let on_idx = l.find(" on ").ok_or(CustomError::MountParsingError(
                    "could not find \" on \" in mount output".to_string(),
                ))?;
                let type_idx = l.find(" type ").ok_or(CustomError::MountParsingError(
                    "could not find \" type \" in mount output".to_string(),
                ))?;
                let bracket_idx = l.find(" (").ok_or(CustomError::MountParsingError(
                    "could not find \" (\" in mount output".to_string(),
                ))?;

                Ok(MountInformation {
                    device: l[0..on_idx].to_string(),
                    mount_point: l[on_idx + 4..type_idx].to_string(),
                    fs_type: l[type_idx + 6..bracket_idx].to_string(),
                    properties: l[bracket_idx + 2..l.len() - 1]
                        .split(",")
                        .map(|s| match s.find("=") {
                            Some(equal_idx) => (
                                s[..equal_idx].to_string(),
                                Some(s[equal_idx + 1..].to_string()),
                            ),
                            None => (s.to_string(), None),
                        })
                        .collect(),
                })
            })
            .collect()
    }
}
