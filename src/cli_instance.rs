// Copyright (c) 2019-2022 Alibaba Cloud
// Copyright (c) 2019-2022 Ant Group
//
// SPDX-License-Identifier: Apache-2.0
//

use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock},
};

use crate::{parser::args::CreateArgs, vmm_comm_trait::VMMComm};
use anyhow::{anyhow, Result};
use crossbeam_channel::{Receiver, Sender};
use seccompiler::BpfProgram;
use vmm_sys_util::eventfd::EventFd;

use dragonball::{
    api::v1::{
        BlockDeviceConfigInfo, BootSourceConfig, InstanceInfo, NetworkInterfaceConfig, VmmRequest,
        VmmResponse, VsockDeviceConfigInfo,
    },
    device_manager::{
        fs_dev_mgr::FsDeviceConfigInfo,
        vfio_dev_mgr::{HostDeviceConfig, VfioPciDeviceConfig},
    },
    vm::{CpuTopology, VmConfigInfo},
};

const DRAGONBALL_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct CliInstance {
    /// VMM instance info directly accessible from runtime
    pub vmm_shared_info: Arc<RwLock<InstanceInfo>>,
    pub to_vmm: Option<Sender<VmmRequest>>,
    pub from_vmm: Option<Arc<Mutex<Receiver<VmmResponse>>>>,
    pub to_vmm_fd: EventFd,
    pub seccomp: BpfProgram,
}

impl VMMComm for CliInstance {
    fn get_to_vmm(&self) -> Option<&Sender<VmmRequest>> {
        self.to_vmm.as_ref()
    }

    fn get_from_vmm(&self) -> Option<Arc<Mutex<Receiver<VmmResponse>>>> {
        self.from_vmm.clone()
    }

    fn get_to_vmm_fd(&self) -> &EventFd {
        &self.to_vmm_fd
    }
}
impl CliInstance {
    pub fn new(id: &str) -> Self {
        let vmm_shared_info = Arc::new(RwLock::new(InstanceInfo::new(
            String::from(id),
            DRAGONBALL_VERSION.to_string(),
        )));

        let to_vmm_fd = EventFd::new(libc::EFD_NONBLOCK)
            .unwrap_or_else(|_| panic!("Failed to create eventfd for vmm {id}"));

        CliInstance {
            vmm_shared_info,
            to_vmm: None,
            from_vmm: None,
            to_vmm_fd,
            seccomp: vec![],
        }
    }

    pub fn run_vmm_server(&self, args: CreateArgs) -> Result<()> {
        if args.kernel_path.is_none() || args.rootfs_args.rootfs.is_none() {
            return Err(anyhow!(
                "kernel path or rootfs path cannot be None when creating the VM"
            ));
        }
        let mut serial_path: Option<String> = None;

        if args.serial_path != "stdio" {
            serial_path = Some(args.serial_path);
        }

        // configuration
        let vm_config = VmConfigInfo {
            vcpu_count: args.cpu.vcpu,
            max_vcpu_count: args.cpu.max_vcpu,
            cpu_pm: args.cpu.cpu_pm.clone(),
            cpu_topology: CpuTopology {
                threads_per_core: args.cpu.cpu_topology.threads_per_core,
                cores_per_die: args.cpu.cpu_topology.cores_per_die,
                dies_per_socket: args.cpu.cpu_topology.dies_per_socket,
                sockets: args.cpu.cpu_topology.sockets,
            },
            vpmu_feature: args.cpu.vpmu_feature,
            mem_type: args.mem.mem_type.clone(),
            mem_file_path: args.mem.mem_file_path.clone(),
            mem_size_mib: args.mem.mem_size,
            // as in crate `dragonball` serial_path will be assigned with a default value,
            // we need a special token to enable the stdio console.
            serial_path: serial_path.clone(),
            pci_hotplug_enabled: args.host_device.pci_hotplug_enabled,
        };

        if let Some(com1_sock_path) = serial_path {
            // check the existence of the serial path (rm it if exist)
            // unwrap is safe  because we have check it is Some above.
            let serial_file = Path::new(com1_sock_path.as_str());
            if serial_file.exists() {
                std::fs::remove_file(serial_file)?;
            }
        }

        // boot source
        let boot_source_config = BootSourceConfig {
            // unwrap is safe because we have checked kernel_path in the beginning of run_vmm_server
            kernel_path: args.kernel_path.unwrap(),
            initrd_path: args.initrd_path.clone(),
            boot_args: Some(args.boot_args.clone()),
        };

        // rootfs
        let mut block_device_config_info = BlockDeviceConfigInfo::default();
        block_device_config_info = BlockDeviceConfigInfo {
            drive_id: String::from("rootfs"),
            // unwrap is safe because we have checked rootfs path in the beginning of run_vmm_server
            path_on_host: PathBuf::from(&args.rootfs_args.rootfs.unwrap()),
            is_root_device: args.rootfs_args.is_root,
            is_read_only: args.rootfs_args.is_read_only,
            ..block_device_config_info
        };

        // set vm configuration
        self.set_vm_configuration(vm_config)
            .expect("failed to set vm configuration");

        // set boot source config
        self.put_boot_source(boot_source_config)
            .expect("failed to set boot source");

        // set rootfs
        self.insert_block_device(block_device_config_info)
            .expect("failed to set block device");

        if !args.vsock.is_empty() {
            // VSOCK config
            let mut vsock_config_info = VsockDeviceConfigInfo::default();
            vsock_config_info = VsockDeviceConfigInfo {
                guest_cid: 42, // dummy value
                uds_path: Some(args.vsock),
                ..vsock_config_info
            };

            // set vsock
            self.insert_vsock(vsock_config_info)
                .expect("failed to set vsock socket path");
        }

        // users should at least provide hostdev_id and bus_slot_func to insert a host device
        if args.host_device.hostdev_id.is_some() && args.host_device.bus_slot_func.is_some() {
            let host_device_config = HostDeviceConfig {
                hostdev_id: args
                    .host_device
                    .hostdev_id
                    .expect("There has to be hostdev_id if you want to add host device."),
                sysfs_path: args.host_device.sysfs_path.unwrap_or_default(),
                dev_config: VfioPciDeviceConfig {
                    bus_slot_func: args
                        .host_device
                        .bus_slot_func
                        .expect("There has to be bus_slot_func if you want to add host device."),
                    vendor_device_id: args.host_device.vendor_device_id.unwrap_or_default(),
                    guest_dev_id: args.host_device.guest_dev_id,
                    clique_id: args.host_device.clique_id,
                },
            };
            self.insert_host_device(host_device_config)
                .expect("Failed to insert a host device");
        }
        // Virtio devices
        if !args.virnets.is_empty() {
            let configs: Vec<NetworkInterfaceConfig> = serde_json::from_str(&args.virnets)
                .unwrap_or_else(|err| {
                    panic!("Failed to parse NetworkInterfaceConfig from JSON: {}", err)
                });
            for config in configs.into_iter() {
                self.insert_virnet(config)
                    .expect("Failed to insert a virtio device");
            }
        }

        if !args.virblks.is_empty() {
            let configs: Vec<BlockDeviceConfigInfo> = serde_json::from_str(&args.virblks)
                .expect("failed to parse virtio-blk devices from JSON");
            for config in configs.into_iter() {
                self.insert_virblk(config)
                    .expect("failed to insert a virtio-blk device");
            }
        }

        if !args.fs.is_empty() {
            let fs_config: FsDeviceConfigInfo = serde_json::from_str(&args.fs)
                .expect("failed to parse virtio-fs devices from JSON");
            self.insert_fs(fs_config)
                .expect("failed to insert a virtio-fs device");
        }

        // start micro-vm
        self.instance_start().expect("failed to start micro-vm");

        Ok(())
    }
}
