use std::collections::HashMap;
mod custom_error;
use custom_error::CustomError;

struct Subvolume {
    id: String,
    uuid: String,
}

fn main() {
    println!("Hello, world!");
}

fn get_mount_points(mounts: &str) -> Result<HashMap<String, String>, CustomError> {
    let mut hm: HashMap<String, String> = HashMap::new();

    for line in mounts.split("\n").into_iter() {
        let mut tokens = line.split(" ");

        let mount_point = tokens.nth(1).ok_or("could not find mount point")?;

        if tokens.next().ok_or("could not find filesystem")? == "btrfs" {
            let subvolid = tokens.next().ok_or("could not find options")?.split(",").into_iter().find(|st| st.starts_with("subvolid=")).ok_or("could not find sub-volume id")?.strip_prefix("subvolid=").ok_or("could not extract sub-volume id")?;
            hm.insert(subvolid.into(), mount_point.into());
        }
    }

    Ok(hm)
}

/// Extract the snapshots for a given subvolume.
///
/// * `path` - path of the subvolume
/// * `subvolume_list` - output of the commant `sudo btrfs subvolume list -tupq --sort=rootid /`
fn get_snapshots(path: &str, subvolume_list: &str) -> Result<Vec<String>, CustomError> {
    let mut snapshots: Vec<String> = Vec::new();

    let mut lines = subvolume_list.split("\n");

    if lines.next().ok_or("could not find header line")?.split_ascii_whitespace().collect::<Vec<&str>>() != vec!["ID", "gen", "parent", "top", "level", "parent_uuid", "uuid", "path"] {
        return Err("unexpected header line".into());
    }

    let root = String::from("/");
    let mut sv: Option<Subvolume> = None;

    for line in lines.skip(1).into_iter() {
        let tokens: Vec<&str> = line.split_ascii_whitespace().collect();

        match &sv {
            Some(s) => {
                if tokens[4] == s.uuid {
                    snapshots.push(root.clone() + tokens[6]);
                }
            },
            None => {
                if root.clone() + tokens[6] == path {
                    sv = Some(Subvolume {
                        id: tokens[0].into(),
                        uuid: tokens[5].into(),
                    });
                }
            }
        }
    }

    Ok(snapshots)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    #[test]
    fn test_get_mount_points() {
        let mut exp: HashMap<String, String> = HashMap::new();
        exp.insert("359".into(), "/".into());
        exp.insert("256".into(), "/home".into());

        let input = "efivarfs /sys/firmware/efi/efivars efivarfs rw,nosuid,nodev,noexec,relatime 0 0\nnone /sys/fs/bpf bpf rw,nosuid,nodev,noexec,relatime,mode=700 0 0\n/dev/mapper/luks-0f3f6c5e-621a-40d8-8be8-c372eaf2d616 / btrfs rw,seclabel,relatime,compress=zstd:1,ssd,space_cache,subvolid=359,subvol=/root 0 0\nselinuxfs /sys/fs/selinux selinuxfs rw,nosuid,noexec,relatime 0 0\n/dev/mapper/luks-0f3f6c5e-621a-40d8-8be8-c372eaf2d616 /home btrfs rw,seclabel,relatime,compress=zstd:1,ssd,space_cache,subvolid=256,subvol=/home 0 0";
        let res = crate::get_mount_points(input).unwrap();
        assert_eq!(exp, res);
    }

    #[test]
    fn test_get_snapshots() {
        let input = r#"ID      gen     parent  top level       parent_uuid     uuid    path
--      ---     ------  ---------       -----------     ----    ----
256     112747  5       5               -                                       11eed410-7829-744e-8288-35c21d278f8e    home
359     112747  5       5               -                                       32c672fa-d3ce-0b4e-8eaa-ab9205f377ca    root
360     112737  359     359             -                                       5f0b151b-52e4-4445-aa94-d07056733a1f    opt/btrfs_test
361     107324  359     359             5f0b151b-52e4-4445-aa94-d07056733a1f    8d5c1a34-2c33-c646-8bb6-0723e2c5c356    snapshots/2021-04-29T15:54:00Z_inf_btrfs_test
362     112737  360     360             -                                       099b9497-11ad-b14b-838a-79e5e7b6084e    opt/btrfs_test/test2
363     112744  256     256             -                                       d7a747f8-aed0-9846-82d1-7dd2ed38705f    home/test"#;

        assert_eq!(vec!["/snapshots/2021-04-29T15:54:00Z_inf_btrfs_test"], crate::get_snapshots("/opt/btrfs_test", input).unwrap());
    }
}
