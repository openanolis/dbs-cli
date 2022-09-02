// Copyright (c) 2019-2022 Alibaba Cloud
// Copyright (c) 2019-2022 Ant Group
//
// SPDX-License-Identifier: Apache-2.0
//

use std::{
    path::{Path, PathBuf},
    sync::{
        mpsc::{Receiver, Sender},
        Arc, RwLock,
    },
};

use anyhow::{anyhow, Context, Result};
use seccompiler::BpfProgram;
use vmm_sys_util::eventfd::EventFd;

use dragonball::{
    api::v1::{
        BlockDeviceConfigInfo, BootSourceConfig, InstanceInfo, VmmAction, VmmActionError, VmmData,
        VmmRequest, VmmResponse,
    },
    vm::{CpuTopology, VmConfigInfo},
};

use crate::parser::DBSArgs;

const DRAGONBALL_VERSION: &str = env!("CARGO_PKG_VERSION");
const REQUEST_RETRY: u32 = 500;

pub enum Request {
    Sync(VmmAction),
}

pub struct CliInstance {
    /// VMM instance info directly accessible from runtime
    pub vmm_shared_info: Arc<RwLock<InstanceInfo>>,
    pub to_vmm: Option<Sender<VmmRequest>>,
    pub from_vmm: Option<Receiver<VmmResponse>>,
    pub to_vmm_fd: EventFd,
    pub seccomp: BpfProgram,
}

impl CliInstance {
    pub fn new(id: &str) -> Self {
        let vmm_shared_info = Arc::new(RwLock::new(InstanceInfo::new(
            String::from(id),
            DRAGONBALL_VERSION.to_string(),
        )));

        let to_vmm_fd = EventFd::new(libc::EFD_NONBLOCK)
            .unwrap_or_else(|_| panic!("Failed to create eventfd for vmm {}", id));

        CliInstance {
            vmm_shared_info,
            to_vmm: None,
            from_vmm: None,
            to_vmm_fd,
            seccomp: vec![],
        }
    }

    pub fn run_vmm_server(&mut self, args: DBSArgs) -> Result<()> {
        // configuration
        let vm_config = VmConfigInfo {
            vcpu_count: args.create_args.vcpu,
            max_vcpu_count: args.create_args.max_vcpu,
            cpu_pm: args.create_args.cpu_pm.clone(),
            cpu_topology: CpuTopology {
                threads_per_core: args.create_args.cpu_topology.threads_per_core,
                cores_per_die: args.create_args.cpu_topology.cores_per_die,
                dies_per_socket: args.create_args.cpu_topology.dies_per_socket,
                sockets: args.create_args.cpu_topology.sockets,
            },
            vpmu_feature: 0,
            mem_type: args.create_args.mem_type.clone(),
            mem_file_path: args.create_args.mem_file_path.clone(),
            mem_size_mib: args.create_args.mem_size,
            // as in crate `dragonball` serial_path will be assigned with a default value,
            // we need a special token to enable the stdio console.
            serial_path: Some(args.create_args.serial_path.clone()),
        };

        // check the existence of the serial path (rm it if exist)
        let serial_file = Path::new(&args.create_args.serial_path);
        if args.create_args.serial_path != *"stdio" && serial_file.exists() {
            std::fs::remove_file(serial_file).unwrap();
        }

        // boot source
        let boot_source_config = BootSourceConfig {
            kernel_path: args.boot_args.kernel_path.clone(),
            initrd_path: args.boot_args.initrd_path.clone(),
            boot_args: Some(args.boot_args.boot_args.clone()),
        };

        // rootfs
        let mut block_device_config_info = BlockDeviceConfigInfo::default();
        block_device_config_info = BlockDeviceConfigInfo {
            drive_id: String::from("rootfs"),
            path_on_host: PathBuf::from(&args.boot_args.rootfs_args.rootfs),
            is_root_device: args.boot_args.rootfs_args.is_root,
            is_read_only: args.boot_args.rootfs_args.is_read_only,
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

        // start micro-vm
        self.instance_start().expect("failed to start micro-vm");

        Ok(())
    }

    pub fn put_boot_source(&self, boot_source_cfg: BootSourceConfig) -> Result<()> {
        self.handle_request(Request::Sync(VmmAction::ConfigureBootSource(
            boot_source_cfg,
        )))
        .context("Failed to configure boot source")?;
        Ok(())
    }

    pub fn instance_start(&self) -> Result<()> {
        self.handle_request(Request::Sync(VmmAction::StartMicroVm))
            .context("Failed to start MicroVm")?;
        Ok(())
    }

    pub fn insert_block_device(&self, device_cfg: BlockDeviceConfigInfo) -> Result<()> {
        self.handle_request_with_retry(Request::Sync(VmmAction::InsertBlockDevice(
            device_cfg.clone(),
        )))
        .with_context(|| format!("Failed to insert block device {:?}", device_cfg))?;
        Ok(())
    }

    pub fn set_vm_configuration(&self, vm_config: VmConfigInfo) -> Result<()> {
        self.handle_request(Request::Sync(VmmAction::SetVmConfiguration(
            vm_config.clone(),
        )))
        .with_context(|| format!("Failed to set vm configuration {:?}", vm_config))?;
        Ok(())
    }

    fn send_request(&self, vmm_action: VmmAction) -> Result<VmmResponse> {
        if let Some(ref to_vmm) = self.to_vmm {
            to_vmm
                .send(Box::new(vmm_action.clone()))
                .with_context(|| format!("Failed to send  {:?} via channel ", vmm_action))?;
        } else {
            return Err(anyhow!("to_vmm is None"));
        }

        //notify vmm action
        if let Err(e) = self.to_vmm_fd.write(1) {
            return Err(anyhow!("failed to notify vmm: {}", e));
        }

        if let Some(from_vmm) = self.from_vmm.as_ref() {
            match from_vmm.recv() {
                Err(e) => Err(anyhow!("vmm recv err: {}", e)),
                Ok(vmm_outcome) => Ok(vmm_outcome),
            }
        } else {
            Err(anyhow!("from_vmm is None"))
        }
    }

    fn handle_request(&self, req: Request) -> Result<VmmData> {
        let Request::Sync(vmm_action) = req;
        match self.send_request(vmm_action) {
            Ok(vmm_outcome) => match *vmm_outcome {
                Ok(vmm_data) => Ok(vmm_data),
                Err(vmm_action_error) => Err(anyhow!("vmm action error: {:?}", vmm_action_error)),
            },
            Err(e) => Err(e),
        }
    }

    fn handle_request_with_retry(&self, req: Request) -> Result<VmmData> {
        let Request::Sync(vmm_action) = req;
        for _ in 0..REQUEST_RETRY {
            match self.send_request(vmm_action.clone()) {
                Ok(vmm_outcome) => match *vmm_outcome {
                    Ok(vmm_data) => {
                        return Ok(vmm_data);
                    }
                    Err(vmm_action_error) => {
                        if let VmmActionError::UpcallNotReady = vmm_action_error {
                            std::thread::sleep(std::time::Duration::from_millis(10));
                            continue;
                        } else {
                            return Err(vmm_action_error.into());
                        }
                    }
                },
                Err(err) => {
                    return Err(err);
                }
            }
        }
        Err(anyhow::anyhow!(
            "After {} attempts, it still doesn't work.",
            REQUEST_RETRY
        ))
    }
}
