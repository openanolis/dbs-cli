// Copyright (c) 2022 Alibaba Cloud
//
// SPDX-License-Identifier: Apache-2.0
//

use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

use std::io::prelude::*;
use std::os::unix::net::{UnixListener, UnixStream};

use anyhow::{Context, Result};

use crate::vmm_comm_trait::VMMComm;
use dragonball::api::v1::{VmmRequest, VmmResponse};
use dragonball::vcpu::VcpuResizeInfo;
use serde_json::Value;

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
            _ => {
                println!("Unknown Actions");
            }
        }

        println!("{message}");
        Ok(())
    }
}
