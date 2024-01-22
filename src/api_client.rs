// Copyright (C) 2022 Alibaba Cloud. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

use std::io::Write;
use std::os::unix::net::UnixStream;

use anyhow::{Context, Result};
use serde_json::{json, Value};

use crate::parser::args::{HostDeviceArgs, UpdateArgs};

pub fn run_api_client(args: UpdateArgs, api_sock_path: &str) -> Result<()> {
    if let Some(vcpu_resize_num) = args.vcpu_resize {
        let request = request_cpu_resize(vcpu_resize_num);
        send_request(request, api_sock_path)?;
    }

    if let Some(config) = args.virnets {
        let request = request_virtio_net(&config);
        send_request(request, api_sock_path)?;
    }

    if let Some(config) = args.virblks {
        let request = request_virtio_blk(&config);
        send_request(request, api_sock_path)?;
    }

    if let Some(config) = args.patch_fs {
        let request = request_patch_fs(&config);
        send_request(request, api_sock_path)?;
    }

    if let Some(host_device_args) = args.insert_host_device {
        if host_device_args.bus_slot_func.is_some() {
            let request = request_insert_host_device(host_device_args.clone());
            send_request(request, api_sock_path)?;
        }
    }

    if let Some(host_device_id) = args.prepare_remove_host_device {
        let request = request_prepare_remove_host_device(host_device_id.clone());
        send_request(request, api_sock_path)?;
    }

    if let Some(host_device_id) = args.remove_host_device {
        let request = request_remove_host_device(host_device_id.clone());
        send_request(request, api_sock_path)?;
    }
    Ok(())
}

fn request_cpu_resize(vcpu_resize_num: usize) -> Value {
    json!({
        "action": "resize_vcpu",
        "vcpu_count": vcpu_resize_num,
    })
}

/// Insert virtio network devices
fn request_virtio_net(net_config: &str) -> Value {
    json!({
        "action": "insert_virnets",
        "config": net_config,
    })
}

/// Insert virtio-blk devices
fn request_virtio_blk(virtio_blk_config: &str) -> Value {
    json!({
        "action": "insert_virblks",
        "config": virtio_blk_config,
    })
}

fn request_patch_fs(patch_fs_config: &str) -> Value {
    json!({
        "action": "patch_fs",
        "config": patch_fs_config,
    })
}

fn request_insert_host_device(host_device_args: HostDeviceArgs) -> Value {
    json!({
        "action": "insert_host_device",
        "hostdev-id": host_device_args.hostdev_id.expect("host device arg should be provided to insert host device."),
        "bus-slot-func": host_device_args.bus_slot_func.expect("bus_slot_func should be provided to insert host device."),
    })
}

fn request_prepare_remove_host_device(host_device_id: String) -> Value {
    json!({
        "action": "prepare_remove_host_device",
        "hostdev-id": host_device_id.clone(),
    })
}

fn request_remove_host_device(host_device_id: String) -> Value {
    json!({
        "action": "remove_host_device",
        "hostdev-id": host_device_id.clone(),
    })
}

fn send_request(request: Value, api_sock_path: &str) -> Result<()> {
    let mut unix_stream = UnixStream::connect(api_sock_path).context("Could not create stream")?;

    unix_stream
        .write(request.to_string().as_bytes()) // we write bytes, &[u8]
        .context("Failed at writing onto the unix stream")?;

    Ok(())
}
