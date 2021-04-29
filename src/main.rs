use std::collections::HashMap;
mod custom_error;
use custom_error::CustomError;


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
}
