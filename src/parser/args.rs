// Copyright (C) 2022 Alibaba Cloud. All rights reserved.
// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use clap::{Args, Parser, Subcommand};
use serde_derive::{Deserialize, Serialize};

/// A simple command-line tool to start DragonBall micro-VM
#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct DBSArgs {
    #[clap(subcommand)]
    pub command: Option<Commands>,

    #[clap(
        long,
        value_parser,
        default_value = "",
        help = "The path to the api server socket file (should be a unix domain socket in the host)",
        display_order = 2
    )]
    pub api_sock_path: String,
}

#[allow(clippy::large_enum_variant)]
#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Create Dragonball Instance
    Create {
        // create args are for setting up Dragonball CPU/
        #[clap(flatten)]
        create_args: CreateArgs,
    },
    /// Connect to Dragonball Api Server and update the Dragonball VM (Must create a api socket when creating the Dragonball VM)
    Update {
        #[clap(flatten)]
        update_args: UpdateArgs,
    },
}

/// CPU topology related configurations
#[derive(Args, Debug, Serialize, Deserialize, Clone)]
pub struct CpuTopologyArgs {
    #[clap(
        long,
        value_parser,
        default_value_t = 1,
        help = "Threads per core to indicate hyper-threading is enabled or not",
        display_order = 1
    )]
    pub threads_per_core: u8,

    #[clap(
        long,
        value_parser,
        default_value_t = 1,
        help = "Cores per die to guide guest cpu topology init",
        display_order = 1
    )]
    pub cores_per_die: u8,

    #[clap(
        long,
        value_parser,
        default_value_t = 1,
        help = "Dies per socket to guide guest cpu topology",
        display_order = 1
    )]
    pub dies_per_socket: u8,

    #[clap(
        long,
        value_parser,
        default_value_t = 1,
        help = "The number of sockets",
        display_order = 1
    )]
    pub sockets: u8,
}

/// Rootfs configuration
#[derive(Args, Debug, Serialize, Deserialize, Clone)]
pub struct RootfsArgs {
    #[clap(
        short,
        long,
        value_parser,
        help = "The path of rootfs file",
        display_order = 4
    )]
    pub rootfs: Option<String>,

    #[clap(
        long,
        value_parser,
        default_value_t = true,
        help = "Decide the device to be the root boot device or not [default: true]",
        display_order = 5
    )]
    pub is_root: bool,

    #[clap(
        long,
        value_parser,
        default_value_t = false,
        help = "The driver opened in read-only or not [default: false]",
        display_order = 6
    )]
    pub is_read_only: bool,
}

/// Configurations used for creating a VM.
#[derive(Args, Debug, Deserialize, Serialize, Clone)]
pub struct CreateArgs {
    /// features of cpu
    #[clap(flatten)]
    pub cpu: CpuArgs,

    /// features of mem
    #[clap(flatten)]
    pub mem: MemArgs,

    /// features of host devices
    #[clap(flatten)]
    pub host_device: HostDeviceArgs,

    // The serial path used to communicate with VM
    #[clap(
        short,
        long,
        value_parser,
        default_value = "stdio",
        help = "The serial path used to communicate with VM",
        display_order = 2
    )]
    pub serial_path: String,

    // The path to a vsock socket file
    // FIXME: add more params:
    // cid="contextid",socket_path="somepath",gid="guest_id"
    #[clap(
        short,
        long,
        value_parser,
        default_value = "",
        help = "Virtio VSOCK socket path",
        display_order = 2
    )]
    pub vsock: String,

    #[clap(
        long,
        value_parser,
        default_value = "",
        help = r#"Insert virtio devices into the Dragonball before launched. The supported devices are virtio-net and vhost-net."#,
        display_order = 2
    )]
    pub virnets: String,

    #[clap(
        long,
        value_parser,
        default_value = "",
        help = r#"Insert virtio-blk devices into the Dragonball.
The type of it is an array of BlockDeviceConfigInfo, e.g.
    --virblks '[{"drive_id":"testblk","device_type":"RawBlock","path_on_host":"/path/to/test.img","is_root_device":false,"is_read_only":false,"is_direct":false,"no_drop":false,"num_queues":1,"queue_size":1024}]'"#,
        display_order = 2
    )]
    pub virblks: String,

    #[clap(
        long,
        value_parser,
        default_value = "",
        help = r#"Insert virtio-fs devices into the Dragonball using the FsDeviceConfigInfo."#,
        display_order = 2
    )]
    pub fs: String,

    // feature for log
    #[clap(long, value_parser, default_value = "dbs-cli.log", display_order = 1)]
    pub log_file: String,

    #[clap(long, value_parser, default_value = "Debug", display_order = 1)]
    pub log_level: String,

    #[clap(
        short,
        long,
        value_parser,
        help = "The path of kernel image (Only uncompressed kernel is supported for Dragonball)",
        display_order = 1
    )]
    pub kernel_path: Option<String>,

    #[clap(
        short,
        long,
        value_parser,
        help = "The path of initrd (Optional)",
        display_order = 2
    )]
    pub initrd_path: Option<String>,

    // for kata_rootfs: 'root=/dev/vda1'
    #[clap(
        short,
        long,
        value_parser,
        default_value = "console=ttyS0 tty0 reboot=k debug panic=1 pci=off root=/dev/vda1",
        help = "The boot arguments passed to the kernel (Optional)",
        display_order = 3
    )]
    pub boot_args: String,

    /// rootfs
    #[clap(flatten)]
    pub rootfs_args: RootfsArgs,
}

#[derive(Args, Debug, Serialize, Deserialize, Clone)]
pub struct CpuArgs {
    #[clap(
        short = 'C',
        long,
        value_parser,
        default_value_t = 1,
        help = "The number of vcpu to start",
        display_order = 1
    )]
    pub vcpu: u8,
    #[clap(
        long,
        value_parser,
        default_value_t = 1,
        help = "The max number of vpu can be added",
        display_order = 1
    )]
    pub max_vcpu: u8,
    #[clap(
        long,
        value_parser,
        default_value = "on",
        help = "The cpu power management",
        display_order = 1
    )]
    pub cpu_pm: String,
    #[clap(
        long,
        value_parser,
        default_value_t = 0,
        help = "vpmu support level",
        display_order = 1
    )]
    pub vpmu_feature: u8,
    #[clap(flatten)]
    pub cpu_topology: CpuTopologyArgs,
}

#[derive(Args, Debug, Serialize, Deserialize, Clone)]
pub struct MemArgs {
    #[clap(
        long,
        value_parser,
        default_value = "shmem",
        help = "Memory type that can be either hugetlbfs or shmem, default is shmem",
        display_order = 2
    )]
    pub mem_type: String,
    #[clap(
        long,
        value_parser,
        default_value = "",
        help = "Memory file path",
        display_order = 2
    )]
    pub mem_file_path: String,
    #[clap(
        short,
        long,
        value_parser,
        default_value_t = 128,
        help = "The memory size in Mib",
        display_order = 2
    )]
    pub mem_size: usize,
}

#[derive(Args, Debug, Serialize, Deserialize, Clone)]
pub struct HostDeviceArgs {
    #[clap(
        long,
        value_parser,
        help = "whether pci hotplug ability is enabled or not",
        default_value_t = false,
        display_order = 2
    )]
    pub pci_hotplug_enabled: bool,
    #[clap(long, value_parser, help = "host dev id", display_order = 2)]
    pub hostdev_id: Option<String>,
    #[clap(long, value_parser, help = "sys fs path", display_order = 2)]
    pub sysfs_path: Option<String>,
    #[clap(long, value_parser, help = "bus slot function", display_order = 2)]
    pub bus_slot_func: Option<String>,
    #[clap(long, value_parser, help = "vendor_device_id", display_order = 2)]
    pub vendor_device_id: Option<u32>,
    #[clap(long, value_parser, help = "guest_dev_id", display_order = 2)]
    pub guest_dev_id: Option<u8>,
    #[clap(long, value_parser, help = "clique_id", display_order = 2)]
    pub clique_id: Option<u8>,
}

#[derive(Args, Debug, Serialize, Deserialize, Clone)]
pub struct UpdateArgs {
    #[clap(
        long,
        value_parser,
        help = "Resize Vcpu through connection with dbs-cli api server",
        display_order = 2
    )]
    pub vcpu_resize: Option<usize>,

    #[clap(
        long,
        value_parser,
        help = r#"Insert hotplug virtio-net devices into the Dragonball. 
The type of it is an array of VirtioNetDeviceConfigInfo, e.g.
    --virnets '[{"iface_id":"eth0","host_dev_name":"tap0","num_queues":2,"queue_size":0,"allow_duplicate_mac":true}]'"#,
        display_order = 2
    )]
    pub virnets: Option<String>,

    #[clap(
        long,
        value_parser,
        help = r#"Insert virtio-blk devices into the Dragonball.
The type of it is an array of BlockDeviceConfigInfo, e.g.
    --virblks '[{"drive_id":"testblk","device_type":"RawBlock","path_on_host":"/path/to/test.img","is_root_device":false,"is_read_only":false,"is_direct":false,"no_drop":false,"num_queues":1,"queue_size":1024}]'"#,
        display_order = 2
    )]
    pub virblks: Option<String>,

    #[clap(
        long,
        value_parser,
        help = r#"attach or detach a virtiofs backend fs using the FsMountConfig."#,
        display_order = 2
    )]
    pub patch_fs: Option<String>,

    #[clap(flatten)]
    pub insert_host_device: Option<HostDeviceArgs>,

    #[clap(
        long,
        value_parser,
        help = r#"prepare remove host device with hostdev_id."#,
        display_order = 2
    )]
    pub prepare_remove_host_device: Option<String>,

    #[clap(
        long,
        value_parser,
        help = r#"remove host device with hostdev_id."#,
        display_order = 2
    )]
    pub remove_host_device: Option<String>,
}
