// Copyright (c) 2022 Alibaba Cloud
//
// SPDX-License-Identifier: Apache-2.0
//

use std::io::prelude::*;
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Context, Result};
use crossbeam_channel::{Receiver, Sender};
use dragonball::api::v1::{NetworkInterfaceConfig, VmmRequest, VmmResponse};
use dragonball::device_manager::blk_dev_mgr::BlockDeviceConfigInfo;
use dragonball::device_manager::fs_dev_mgr::FsMountConfigInfo;
use dragonball::device_manager::vfio_dev_mgr::{HostDeviceConfig, VfioPciDeviceConfig};
use dragonball::vcpu::VcpuResizeInfo;
use serde_json::Value;
use vmm_sys_util::eventfd::EventFd;

use crate::utils;
use crate::vmm_comm_trait::VMMComm;

pub struct ApiServer {
    pub to_vmm: Option<Sender<VmmRequest>>,
    pub from_vmm: Option<Arc<Mutex<Receiver<VmmResponse>>>>,
    pub to_vmm_fd: EventFd,
}

impl VMMComm for ApiServer {
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
impl ApiServer {
    pub fn new(
        to_vmm: Option<Sender<VmmRequest>>,
        from_vmm: Option<Arc<Mutex<Receiver<VmmResponse>>>>,
        to_vmm_fd: EventFd,
    ) -> Self {
        ApiServer {
            to_vmm,
            from_vmm,
            to_vmm_fd,
        }
    }

    pub fn run_api_server(&mut self, api_sock_path: String) -> Result<()> {
        println!("dbs-cli: api server created in api_sock_path {:?}. Start waiting for connections from the client side.", &api_sock_path);
        let unix_listener = UnixListener::bind(api_sock_path)?;

        // put the server logic in a loop to accept several connections
        loop {
            let (unix_stream, _socket_address) = unix_listener
                .accept()
                .context("Failed at accepting a connection on the unix listener")?;
            self.handle_stream(unix_stream)?;
        }
    }

    pub fn handle_stream(&mut self, mut unix_stream: UnixStream) -> Result<()> {
        let mut message = String::new();
        unix_stream
            .read_to_string(&mut message)
            .context("Failed at reading the unix stream")?;

        // Parse the string of data into serde_json::Value.
        let v: Value = serde_json::from_str(&message)?;

        match v["action"].as_str() {
            Some("resize_vcpu") => {
                let resize_vcpu_cfg = VcpuResizeInfo {
                    vcpu_count: v["vcpu_count"].as_u64().map(|count| count as u8),
                };
                return self.resize_vcpu(resize_vcpu_cfg);
            }
            Some("insert_host_device") => {
                // TODO: add customize support for sysfs_path, vendor_device_id, guest_dev_id and clique_id.
                // ignore them now since they are not the must parameters for hotplugging a host device.
                // issue: #31
                let host_device_config = HostDeviceConfig {
                    hostdev_id: v["hostdev-id"].as_str().unwrap().to_owned(),
                    sysfs_path: "".to_string(),
                    dev_config: VfioPciDeviceConfig {
                        bus_slot_func: v["bus-slot-func"].as_str().unwrap().to_owned(),
                        vendor_device_id: 0,
                        guest_dev_id: None,
                        clique_id: None,
                    },
                };
                self.insert_host_device(host_device_config)
                    .expect("Failed to insert a host device");
            }
            Some("prepare_remove_host_device") => {
                self.prepare_remove_host_device(v["hostdev-id"].as_str().unwrap().to_owned())
                    .expect("Failed to insert a host device");
            }
            Some("remove_host_device") => {
                self.remove_host_device(v["hostdev-id"].as_str().unwrap().to_owned())
                    .expect("Failed to insert a host device");
            }
            Some("insert_virnets") => {
                let config_json = match v["config"].as_str() {
                    Some(config_json) => config_json,
                    None => return Err(anyhow!("The config of virtio-net device is required")),
                };
                let configs: Vec<NetworkInterfaceConfig> = serde_json::from_str(config_json)
                    .context("Parse NetworkInterfaceConfig from JSON")?;
                for config in configs.iter() {
                    self.insert_virnet(config.clone()).with_context(|| {
                        format!(
                            "Insert a {} device to the Dragonball",
                            utils::net_device_name(config)
                        )
                    })?;
                }
            }
            Some("insert_virblks") => {
                let config_json = match v["config"].as_str() {
                    Some(config_json) => config_json,
                    None => return Err(anyhow!("The config of virtio-blk device is required")),
                };
                let configs: Vec<BlockDeviceConfigInfo> = serde_json::from_str(config_json)
                    .context("Parse virtio-blk device config from json")?;
                for config in configs.iter() {
                    self.insert_virblk(config.clone())
                        .context("Insert a virtio-blk device to the Dragonball")?;
                }
            }
            Some("patch_fs") => {
                let config_json = match v["config"].as_str() {
                    Some(config_json) => config_json,
                    None => return Err(anyhow!("The config of patch fs is required")),
                };
                let config: FsMountConfigInfo =
                    serde_json::from_str(config_json).context("Parse patch fs config from json")?;
                self.patch_fs(config)
                    .context("Insert a patch fs to the Dragonball")?;
            }
            _ => {
                println!("Unknown Actions");
            }
        }

        println!("{message}");
        Ok(())
    }
}
