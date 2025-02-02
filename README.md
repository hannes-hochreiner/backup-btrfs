# backup-btrfs

A simple script to coordinate local backups.

## User Story

As a user, I want backups of all the files in my home directory to be made automatically every 30 minutes so that I don't have to worry about loosing data.

## Requirements

ID | Description | Priority
---|---|---
01 | The system must be able to make a backup of all files every 30 minutes. | Mandatory
02 | The system must provide meaningful error reports to the user. | Mandatory
03 | The system should provide status information to the user. | Optional
04 | The system should provide the user the ability to exclude selected files. | Optional
05 | The system must be able to run computers based on x86 (64bit) and ARMv6 (32bit hf) architectures running Linux. | Mandatory

## Technical Specification

The system will be written in Rust.
It will use the btrfs tools for making the backup.
Execution of the system will be triggered by systemd.

## Architecture

The system involves two hosts:
* host 1: the host having the subvolume that should be backed up
* host 2: the host that will store the backups

First a snapshot of the subvolume will be created on host 1.
The snapshot will then be transferred to host 2.
If host 2 already contains a suitable parent snapshot, an incremental transfer will be made.
Otherwise, a complete transfer will be made.
Finally, the retention policies for snapshots will be applied to both hosts.
Typically, only a few recent snapshots will be kept on host 1, while host 2 keeps snapshots for a longer time.

![architecture diagram](docs/bld/architecture.svg)

## Deployment

The program can be installed via cargo from the GitHub repository.

```bash
cargo install --git https://github.com/hannes-hochreiner/backup-btrfs
```

### Sequence of actions

1. Read the configuration file
2. Create command execution contexts
    1. Create local context
    2. Create remote context
3. Create new local snapshot
    1. Create the snapshot (requires the subvolume path, snapshot path, and the suffix)
    ```shell
    btrfs subvolume snapshot -r <subvolume path> <snapshot path>
    ```
    2. Get snapshot/subvolume information (requires snapshot path)
    ```shell
    btrfs subvolume show <subvolume path>
    ```
4. Get device information
    1. Get local device information
    2. Get remote device information
    ```shell
    readlink -f <device>
    ```
5. Get mount information
    1. Get local mount information
    2. Get remove mount information
    ```shell
    findmnt -lnvt btrfs -o FSROOT,TARGET,FSTYPE,SOURCE,OPTIONS
    ```
6. Send snapshot
7. Apply retention policy to snapshots
    1. Apply retention policy to local snapshots
    2. Apply retention policy to remote snapshots

## License

This work is licensed under the MIT or Apache 2.0 license.

`SPDX-License-Identifier: MIT OR Apache-2.0`