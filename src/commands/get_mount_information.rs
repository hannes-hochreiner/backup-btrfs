use crate::backup_error::BackupError;
use crate::objects::MountInformation;
use exec_rs::{Context, Exec};

pub trait CommandGetMountInformation {
    /// Get the mount information of all btrfs mounts.
    ///
    /// * `exec` - command executor
    /// * `context` - context in which to run the command
    ///
    /// References:
    /// * https://mpdesouza.com/blog/btrfs-differentiating-bind-mounts-on-subvolumes/
    /// * https://www.kernel.org/doc/Documentation/filesystems/proc.txt
    fn get_mount_information(
        &mut self,
        context: &Context,
    ) -> Result<Vec<MountInformation>, BackupError>;
}

impl<T: Exec> CommandGetMountInformation for super::Commander<T> {
    fn get_mount_information(
        &mut self,
        context: &Context,
    ) -> Result<Vec<MountInformation>, BackupError> {
        let command_output = self
            .exec
            .exec("cat", &["/proc/self/mountinfo"], Some(context))?;

        command_output
            .lines()
            .filter(|&l| !l.is_empty())
            .map(|l| {
                let mut iter = l.split(' ');

                Ok(MountInformation {
                    root: iter
                        .nth(3)
                        .ok_or(BackupError::MountParsing("could not find root".to_string()))?
                        .to_string(),
                    mount_point: iter
                        .next()
                        .ok_or(BackupError::MountParsing(
                            "could not find mount point".to_string(),
                        ))?
                        .to_string(),
                    fs_type: iter
                        .nth(3)
                        .ok_or(BackupError::MountParsing(
                            "could not find fs type".to_string(),
                        ))?
                        .to_string(),
                    device: iter
                        .next()
                        .ok_or(BackupError::MountParsing(
                            "could not find device".to_string(),
                        ))?
                        .to_string(),
                    properties: iter
                        .next()
                        .ok_or(BackupError::MountParsing(
                            "could not find properties".to_string(),
                        ))?
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::Commander;
    use exec_rs::{Context, MockExec};
    use std::collections::HashMap;

    #[test]
    fn get_mount_information_btrfs_1() {
        let mut mock = MockExec::new();

        mock.expect_exec().once().withf(|_,_,_| true).returning(|_,_,_| Ok(String::from(r#"65 1 0:32 /root / rw,relatime shared:1 - btrfs /dev/mapper/luks-0f3f6c5e-621a-40d8-8be8-c372eaf2d616 rw,seclabel,compress=zstd:1,ssd,space_cache,subvolid=11858,subvol=/root
89 65 0:32 /home /home rw,relatime shared:40 - btrfs /dev/mapper/luks-0f3f6c5e-621a-40d8-8be8-c372eaf2d616 rw,seclabel,compress=zstd:1,ssd,space_cache,subvolid=256,subvol=/home
1140 65 0:32 /root/var/lib/docker/btrfs /var/lib/docker/btrfs rw,relatime shared:1 - btrfs /dev/mapper/luks-0f3f6c5e-621a-40d8-8be8-c372eaf2d616 rw,seclabel,compress=zstd:1,ssd,space_cache,subvolid=11858,subvol=/root"#)));

        let mut commands = Commander::new_with_exec(mock);

        assert_eq!(
            commands
                .get_mount_information(&Context::Local {
                    user: String::from("test"),
                },)
                .unwrap(),
            vec![
                MountInformation {
                    device: String::from("/dev/mapper/luks-0f3f6c5e-621a-40d8-8be8-c372eaf2d616"),
                    fs_type: String::from("btrfs"),
                    mount_point: String::from("/"),
                    root: String::from("/root"),
                    properties: vec![
                        (String::from("rw"), None),
                        (String::from("seclabel"), None),
                        (String::from("compress"), Some(String::from("zstd:1"))),
                        (String::from("ssd"), None),
                        (String::from("space_cache"), None),
                        (String::from("subvolid"), Some(String::from("11858"))),
                        (String::from("subvol"), Some(String::from("/root")))
                    ]
                    .iter()
                    .cloned()
                    .collect::<HashMap<String, Option<String>>>()
                },
                MountInformation {
                    device: String::from("/dev/mapper/luks-0f3f6c5e-621a-40d8-8be8-c372eaf2d616"),
                    fs_type: String::from("btrfs"),
                    mount_point: String::from("/home"),
                    root: String::from("/home"),
                    properties: vec![
                        (String::from("rw"), None),
                        (String::from("seclabel"), None),
                        (String::from("compress"), Some(String::from("zstd:1"))),
                        (String::from("ssd"), None),
                        (String::from("space_cache"), None),
                        (String::from("subvolid"), Some(String::from("256"))),
                        (String::from("subvol"), Some(String::from("/home")))
                    ]
                    .iter()
                    .cloned()
                    .collect::<HashMap<String, Option<String>>>()
                },
                MountInformation {
                    device: String::from("/dev/mapper/luks-0f3f6c5e-621a-40d8-8be8-c372eaf2d616"),
                    fs_type: String::from("btrfs"),
                    mount_point: String::from("/var/lib/docker/btrfs"),
                    root: String::from("/root/var/lib/docker/btrfs"),
                    properties: vec![
                        (String::from("rw"), None),
                        (String::from("seclabel"), None),
                        (String::from("compress"), Some(String::from("zstd:1"))),
                        (String::from("ssd"), None),
                        (String::from("space_cache"), None),
                        (String::from("subvolid"), Some(String::from("11858"))),
                        (String::from("subvol"), Some(String::from("/root")))
                    ]
                    .iter()
                    .cloned()
                    .collect::<HashMap<String, Option<String>>>()
                },
            ]
        );
    }

    #[test]
    fn get_mount_information_btrfs_2() {
        let context = Context::Local {
            user: String::from("test"),
        };
        let mut mock = MockExec::new();

        mock.expect_exec().once().withf(|_,_,_| true).returning(|_,_,_| Ok(String::from(r#"415 29 0:45 / /data rw,relatime shared:241 - btrfs /dev/mapper/data rw,space_cache=v2,subvolid=5,subvol=/"#)));

        let mut commands = Commander::new_with_exec(mock);

        assert_eq!(
            commands.get_mount_information(&context).unwrap(),
            vec![MountInformation {
                device: String::from("/dev/mapper/data"),
                fs_type: String::from("btrfs"),
                mount_point: String::from("/data"),
                root: String::from("/"),
                properties: vec![
                    (String::from("rw"), None),
                    (String::from("space_cache"), Some(String::from("v2"))),
                    (String::from("subvolid"), Some(String::from("5"))),
                    (String::from("subvol"), Some(String::from("/")))
                ]
                .iter()
                .cloned()
                .collect::<HashMap<String, Option<String>>>()
            },]
        );
    }

    #[test]
    fn get_mount_information_any_1() {
        let context = Context::Local {
            user: String::from("test"),
        };
        let mut mock = MockExec::new();

        mock.expect_exec().once().withf(|_,_,_| true).returning(|_,_,_| Ok(String::from(r#"22 65 0:21 / /proc rw,nosuid,nodev,noexec,relatime shared:13 - proc proc rw
        23 65 0:22 / /sys rw,nosuid,nodev,noexec,relatime shared:2 - sysfs sysfs rw,seclabel
        24 65 0:5 / /dev rw,nosuid shared:9 - devtmpfs devtmpfs rw,seclabel,size=4096k,nr_inodes=1048576,mode=755,inode64
        25 23 0:6 / /sys/kernel/security rw,nosuid,nodev,noexec,relatime shared:3 - securityfs securityfs rw
        26 24 0:23 / /dev/shm rw,nosuid,nodev shared:10 - tmpfs tmpfs rw,seclabel,inode64
        27 24 0:24 / /dev/pts rw,nosuid,noexec,relatime shared:11 - devpts devpts rw,seclabel,gid=5,mode=620,ptmxmode=000
        28 65 0:25 / /run rw,nosuid,nodev shared:12 - tmpfs tmpfs rw,seclabel,size=3233680k,nr_inodes=819200,mode=755,inode64
        29 23 0:26 / /sys/fs/cgroup rw,nosuid,nodev,noexec,relatime shared:4 - cgroup2 cgroup2 rw,seclabel,nsdelegate,memory_recursiveprot
        30 23 0:27 / /sys/fs/pstore rw,nosuid,nodev,noexec,relatime shared:5 - pstore pstore rw,seclabel
        31 23 0:28 / /sys/firmware/efi/efivars rw,nosuid,nodev,noexec,relatime shared:6 - efivarfs efivarfs rw
        32 23 0:29 / /sys/fs/bpf rw,nosuid,nodev,noexec,relatime shared:7 - bpf bpf rw,mode=700
        65 1 0:32 /root / rw,relatime shared:1 - btrfs /dev/mapper/luks-0f3f6c5e-621a-40d8-8be8-c372eaf2d616 rw,seclabel,compress=zstd:1,ssd,space_cache,subvolid=11858,subvol=/root
        34 23 0:19 / /sys/fs/selinux rw,nosuid,noexec,relatime shared:8 - selinuxfs selinuxfs rw
        33 22 0:30 / /proc/sys/fs/binfmt_misc rw,relatime shared:14 - autofs systemd-1 rw,fd=35,pgrp=1,timeout=0,minproto=5,maxproto=5,direct,pipe_ino=1770
        35 24 0:18 / /dev/mqueue rw,nosuid,nodev,noexec,relatime shared:15 - mqueue mqueue rw,seclabel
        36 24 0:35 / /dev/hugepages rw,relatime shared:16 - hugetlbfs hugetlbfs rw,seclabel,pagesize=2M
        37 23 0:7 / /sys/kernel/debug rw,nosuid,nodev,noexec,relatime shared:17 - debugfs debugfs rw,seclabel
        38 23 0:12 / /sys/kernel/tracing rw,nosuid,nodev,noexec,relatime shared:18 - tracefs tracefs rw,seclabel
        39 23 0:36 / /sys/fs/fuse/connections rw,nosuid,nodev,noexec,relatime shared:19 - fusectl fusectl rw
        40 23 0:37 / /sys/kernel/config rw,nosuid,nodev,noexec,relatime shared:20 - configfs configfs rw
        89 65 0:32 /home /home rw,relatime shared:40 - btrfs /dev/mapper/luks-0f3f6c5e-621a-40d8-8be8-c372eaf2d616 rw,seclabel,compress=zstd:1,ssd,space_cache,subvolid=256,subvol=/home
        88 65 259:2 / /boot rw,relatime shared:46 - ext4 /dev/nvme0n1p2 rw,seclabel
        94 88 259:1 / /boot/efi rw,relatime shared:48 - vfat /dev/nvme0n1p1 rw,fmask=0077,dmask=0077,codepage=437,iocharset=ascii,shortname=winnt,errors=remount-ro
        97 65 0:39 / /tmp rw,nosuid,nodev shared:50 - tmpfs tmpfs rw,seclabel,nr_inodes=1048576,inode64
        344 65 0:47 / /var/lib/nfs/rpc_pipefs rw,relatime shared:164 - rpc_pipefs sunrpc rw
        1140 65 0:32 /root/var/lib/docker/btrfs /var/lib/docker/btrfs rw,relatime shared:1 - btrfs /dev/mapper/luks-0f3f6c5e-621a-40d8-8be8-c372eaf2d616 rw,seclabel,compress=zstd:1,ssd,space_cache,subvolid=11858,subvol=/root
        1276 28 0:57 / /run/user/1000 rw,nosuid,nodev,relatime shared:702 - tmpfs tmpfs rw,seclabel,size=1616836k,nr_inodes=404209,mode=700,uid=1000,gid=1000,inode64
        1317 1276 0:58 / /run/user/1000/doc rw,nosuid,nodev,relatime shared:724 - fuse.portal portal rw,user_id=1000,group_id=1000"#)));

        let mut commands = Commander::new_with_exec(mock);

        commands.get_mount_information(&context).unwrap();
    }

    #[test]
    fn get_mount_information_any_2() {
        let context = Context::Local {
            user: String::from("test"),
        };
        let mut mock = MockExec::new();

        mock.expect_exec().once().withf(|_,_,_| true).returning(|_,_,_| Ok(String::from(r#"21 29 0:5 / /dev rw,nosuid shared:8 - devtmpfs devtmpfs rw,size=398300k,nr_inodes=993178,mode=755
22 21 0:20 / /dev/pts rw,nosuid,noexec,relatime shared:9 - devpts devpts rw,gid=3,mode=620,ptmxmode=666
23 21 0:21 / /dev/shm rw,nosuid,nodev shared:10 - tmpfs tmpfs rw,size=3982980k
24 29 0:22 / /proc rw,nosuid,nodev,noexec,relatime shared:2 - proc proc rw
25 29 0:23 / /run rw,nosuid,nodev shared:11 - tmpfs tmpfs rw,size=1991492k,mode=755
26 25 0:24 / /run/keys rw,nosuid,nodev,relatime shared:12 - ramfs none rw,mode=750
27 25 0:25 / /run/wrappers rw,nodev,relatime shared:13 - tmpfs tmpfs rw,mode=755
28 29 0:26 / /sys rw,nosuid,nodev,noexec,relatime shared:3 - sysfs sysfs rw
29 1 8:17 / / rw,relatime shared:1 - ext4 /dev/disk/by-uuid/5e3c62a8-05d2-445a-9eeb-82047d9eaa24 rw
30 29 8:17 /nix/store /nix/store ro,relatime shared:14 - ext4 /dev/disk/by-uuid/5e3c62a8-05d2-445a-9eeb-82047d9eaa24 rw
31 28 0:6 / /sys/kernel/security rw,nosuid,nodev,noexec,relatime shared:4 - securityfs securityfs rw
32 28 0:27 / /sys/fs/cgroup rw,nosuid,nodev,noexec,relatime shared:5 - cgroup2 cgroup2 rw,nsdelegate,memory_recursiveprot
33 28 0:28 / /sys/firmware/efi/efivars rw,nosuid,nodev,noexec,relatime shared:6 - efivarfs efivarfs rw
34 28 0:29 / /sys/fs/bpf rw,nosuid,nodev,noexec,relatime shared:7 - bpf bpf rw,mode=700
35 21 0:30 / /dev/hugepages rw,relatime shared:15 - hugetlbfs hugetlbfs rw,pagesize=2M
36 21 0:18 / /dev/mqueue rw,nosuid,nodev,noexec,relatime shared:16 - mqueue mqueue rw
37 28 0:7 / /sys/kernel/debug rw,nosuid,nodev,noexec,relatime shared:17 - debugfs debugfs rw
38 28 0:31 / /sys/fs/fuse/connections rw,nosuid,nodev,noexec,relatime shared:18 - fusectl fusectl rw
39 28 0:32 / /sys/kernel/config rw,nosuid,nodev,noexec,relatime shared:19 - configfs configfs rw
40 28 0:33 / /sys/fs/pstore rw,nosuid,nodev,noexec,relatime shared:20 - pstore pstore rw
88 29 8:19 / /boot rw,relatime shared:43 - vfat /dev/sdb3 rw,fmask=0022,dmask=0022,codepage=437,iocharset=iso8859-1,shortname=mixed,errors=remount-ro
386 25 0:43 / /run/user/78 rw,nosuid,nodev,relatime shared:209 - tmpfs tmpfs rw,size=796592k,nr_inodes=199148,mode=700,uid=78,gid=78
415 29 0:45 / /data rw,relatime shared:241 - btrfs /dev/mapper/data rw,space_cache=v2,subvolid=5,subvol=/
400 25 0:44 / /run/user/1000 rw,nosuid,nodev,relatime shared:113 - tmpfs tmpfs rw,size=796592k,nr_inodes=199148,mode=700,uid=1000,gid=100"#)));

        let mut commands = Commander::new_with_exec(mock);

        commands.get_mount_information(&context).unwrap();
    }
}
