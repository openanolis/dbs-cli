// Copyright (C) 2022 Alibaba Cloud. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

use std::io::Write;
use std::os::unix::net::UnixStream;

use anyhow::{Context, Result};
use serde_json::{json, Value};

use crate::parser::DBSArgs;

pub fn run_api_client(args: DBSArgs) -> Result<()> {
    let request;
    if let Some(vcpu_resize_num) = args.update_args.vcpu_resize {
        request = request_cpu_resize(vcpu_resize_num);
        send_request(request, args.api_sock_path)?;
    }

    Ok(())
}

fn request_cpu_resize(vcpu_resize_num: usize) -> Value {
    json!({
        "action": "resize_vcpu",
        "vcpu_count": vcpu_resize_num,
    })
}

fn send_request(request: Value, api_sock_path: String) -> Result<()> {
    let mut unix_stream = UnixStream::connect(api_sock_path).context("Could not create stream")?;

    unix_stream
        .write(request.to_string().as_bytes()) // we write bytes, &[u8]
        .context("Failed at writing onto the unix stream")?;

    Ok(())
}
