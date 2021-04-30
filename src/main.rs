mod custom_error;
use custom_error::CustomError;

fn main() {
    println!("Hello, world!");
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
    let mut sv_uuid: Option<String> = None;

    for line in lines.skip(1).into_iter() {
        let tokens: Vec<&str> = line.split_ascii_whitespace().collect();

        match &sv_uuid {
            Some(s) => {
                if tokens[4] == s {
                    snapshots.push(root.clone() + tokens[6]);
                }
            },
            None => {
                if root.clone() + tokens[6] == path {
                    sv_uuid = Some(tokens[5].into());
                }
            }
        }
    }

    Ok(snapshots)
}

#[cfg(test)]
mod tests {
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
