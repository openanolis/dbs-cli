use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Context, Result};
use crossbeam_channel::{Receiver, Sender};
use dragonball::api::v1::{
    BlockDeviceConfigInfo, BootSourceConfig, NetworkInterfaceConfig, VmmAction, VmmActionError,
    VmmData, VmmRequest, VmmResponse, VsockDeviceConfigInfo,
};
use dragonball::device_manager::fs_dev_mgr::{FsDeviceConfigInfo, FsMountConfigInfo};
use dragonball::device_manager::vfio_dev_mgr::HostDeviceConfig;
use dragonball::vcpu::VcpuResizeInfo;
use dragonball::vm::VmConfigInfo;
use vmm_sys_util::eventfd::EventFd;

use crate::utils;

pub enum Request {
    Sync(VmmAction),
}

const REQUEST_RETRY: u32 = 500;

pub trait VMMComm {
    // Method signatures; these will return a string.
    fn get_to_vmm(&self) -> Option<&Sender<VmmRequest>>;
    fn get_from_vmm(&self) -> Option<Arc<Mutex<Receiver<VmmResponse>>>>;
    fn get_to_vmm_fd(&self) -> &EventFd;

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

    fn send_request(&self, vmm_action: VmmAction) -> Result<VmmResponse> {
        if let Some(to_vmm) = self.get_to_vmm() {
            to_vmm
                .send(Box::new(vmm_action.clone()))
                .with_context(|| format!("Failed to send {vmm_action:?} via channel "))?;
        } else {
            return Err(anyhow!("to_vmm is None"));
        }

        //notify vmm action
        if let Err(e) = self.get_to_vmm_fd().write(1) {
            return Err(anyhow!("failed to notify vmm: {}", e));
        }

        if let Some(from_vmm) = self.get_from_vmm().as_ref() {
            match from_vmm.lock().unwrap().recv() {
                Err(e) => Err(anyhow!("vmm recv err: {}", e)),
                Ok(vmm_outcome) => Ok(vmm_outcome),
            }
        } else {
            Err(anyhow!("from_vmm is None"))
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
                        if let VmmActionError::UpcallServerNotReady = vmm_action_error {
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

    fn put_boot_source(&self, boot_source_cfg: BootSourceConfig) -> Result<()> {
        self.handle_request(Request::Sync(VmmAction::ConfigureBootSource(
            boot_source_cfg,
        )))
        .context("Failed to configure boot source")?;
        Ok(())
    }

    fn instance_start(&self) -> Result<()> {
        self.handle_request(Request::Sync(VmmAction::StartMicroVm))
            .context("Failed to start MicroVm")?;
        Ok(())
    }

    fn insert_block_device(&self, device_cfg: BlockDeviceConfigInfo) -> Result<()> {
        self.handle_request_with_retry(Request::Sync(VmmAction::InsertBlockDevice(
            device_cfg.clone(),
        )))
        .with_context(|| format!("Failed to insert block device {device_cfg:?}"))?;
        Ok(())
    }

    fn set_vm_configuration(&self, vm_config: VmConfigInfo) -> Result<()> {
        self.handle_request(Request::Sync(VmmAction::SetVmConfiguration(
            vm_config.clone(),
        )))
        .with_context(|| format!("Failed to set vm configuration {vm_config:?}"))?;
        Ok(())
    }

    fn insert_vsock(&self, vsock_cfg: VsockDeviceConfigInfo) -> Result<()> {
        self.handle_request(Request::Sync(VmmAction::InsertVsockDevice(
            vsock_cfg.clone(),
        )))
        .with_context(|| format!("Failed to insert vsock device {vsock_cfg:?}"))?;
        Ok(())
    }

    fn resize_vcpu(&self, resize_vcpu_cfg: VcpuResizeInfo) -> Result<()> {
        self.handle_request(Request::Sync(VmmAction::ResizeVcpu(
            resize_vcpu_cfg.clone(),
        )))
        .with_context(|| format!("Failed to resize vcpu {resize_vcpu_cfg:?}"))?;
        Ok(())
    }

    fn insert_virnet(&self, config: NetworkInterfaceConfig) -> Result<()> {
        self.handle_request(Request::Sync(VmmAction::InsertNetworkDevice(
            config.clone(),
        )))
        .with_context(|| {
            format!(
                "Request to insert a {} device",
                utils::net_device_name(&config)
            )
        })?;
        Ok(())
    }

    fn insert_virblk(&self, blk_cfg: BlockDeviceConfigInfo) -> Result<()> {
        self.handle_request(Request::Sync(VmmAction::InsertBlockDevice(blk_cfg.clone())))
            .with_context(|| format!("Failed to insert virtio-blk device {:?}", blk_cfg))?;
        Ok(())
    }

    fn insert_fs(&self, fs_cfg: FsDeviceConfigInfo) -> Result<()> {
        self.handle_request(Request::Sync(VmmAction::InsertFsDevice(fs_cfg.clone())))
            .with_context(|| format!("Failed to insert {} fs device {:?}", fs_cfg.mode, fs_cfg))?;
        Ok(())
    }

    fn patch_fs(&self, cfg: FsMountConfigInfo) -> Result<()> {
        self.handle_request(Request::Sync(VmmAction::ManipulateFsBackendFs(cfg.clone())))
            .with_context(|| {
                format!(
                    "Failed to {:?} backend {:?} at {} mount config {:?}",
                    cfg.ops, cfg.fstype, cfg.mountpoint, cfg
                )
            })?;
        Ok(())
    }

    fn insert_host_device(&self, host_device_cfg: HostDeviceConfig) -> Result<()> {
        self.handle_request(Request::Sync(VmmAction::InsertHostDevice(host_device_cfg.clone())))
            .with_context(|| {
                format!(
                    "Failed to insert host device hostdev_id {:?}, sysfs_path {:?}, host_device_cfg {:?}",
                    host_device_cfg.hostdev_id, host_device_cfg.sysfs_path, host_device_cfg.dev_config
                )
            })?;
        Ok(())
    }
}
