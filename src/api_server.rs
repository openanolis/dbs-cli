// Copyright (c) 2022 Alibaba Cloud
//
// SPDX-License-Identifier: Apache-2.0
//

use std::sync::{Arc, Mutex};

use std::io::prelude::*;
use std::os::unix::net::{UnixListener, UnixStream};

use anyhow::{anyhow, Context, Result};
use dragonball::device_manager::blk_dev_mgr::BlockDeviceConfigInfo;
use dragonball::device_manager::mem_dev_mgr::MemDeviceConfigInfo;
use dragonball::device_manager::virtio_net_dev_mgr::VirtioNetDeviceConfigInfo;

use crate::vmm_comm_trait::VMMComm;
use dragonball::api::v1::{VmmRequest, VmmResponse};
use dragonball::vcpu::VcpuResizeInfo;
use serde_json::Value;

use crossbeam_channel::{Receiver, Sender};
use vmm_sys_util::eventfd::EventFd;

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

    pub fn run_api_server(&mut self, api_sock_path: &str) -> Result<()> {
        let unix_listener = UnixListener::bind(api_sock_path)?;
        println!("dbs-cli: api server created in api_sock_path {api_sock_path:?}. Start waiting for connections from the client side.");

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
            Some("insert_virnets") => {
                let config_json = match v["config"].as_str() {
                    Some(config_json) => config_json,
                    None => return Err(anyhow!("The config of virtio-net device is required")),
                };
                let configs: Vec<VirtioNetDeviceConfigInfo> = serde_json::from_str(config_json)
                    .context("Parse virtio-net device config from json")?;
                for config in configs.iter() {
                    self.insert_virnet(config.clone())
                        .context("Insert a virtio-net device to the Dragonball")?;
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
            Some("hotplug_memory") => {
                let mem_cfg = MemDeviceConfigInfo {
                    mem_id: "virtio-mem0".to_string(),
                    capacity_mib: 0,
                    size_mib: v["size_mib"]
                        .as_u64()
                        .context("Invalid virtio-mem size input")?,
                    multi_region: true,
                    host_numa_node_id: None,
                    guest_numa_node_id: None,
                    use_generic_irq: None,
                    use_shared_irq: None,
                };
                return self.insert_mem_device(mem_cfg);
            }
            _ => {
                println!("Unknown Actions");
            }
        }

        println!("{message}");
        Ok(())
    }
}
