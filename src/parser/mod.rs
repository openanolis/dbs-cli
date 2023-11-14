// Copyright (C) 2020-2022 Alibaba Cloud. All rights reserved.
// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::{
    fs::OpenOptions,
    os::unix::io::IntoRawFd,
    sync::{Arc, Mutex},
    thread,
};

use anyhow::Result;
use crossbeam_channel::unbounded;
use dragonball::{api::v1::VmmService, Vmm};

use crate::api_server::ApiServer;
use crate::cli_instance::CliInstance;
use crate::parser::args::CreateArgs;

pub mod args;

const KVM_DEVICE: &str = "/dev/kvm";

pub fn run_with_cli(create_args: CreateArgs, api_sock_path: &String) -> Result<i32> {
    let mut cli_instance = CliInstance::new("dbs-cli");

    let kvm = OpenOptions::new().read(true).write(true).open(KVM_DEVICE)?;

    let (to_vmm, from_runtime) = unbounded();
    let (to_runtime, from_vmm) = unbounded();

    let vmm_service = VmmService::new(from_runtime, to_runtime);

    cli_instance.to_vmm = Some(to_vmm);
    cli_instance.from_vmm = Some(Arc::new(Mutex::new(from_vmm)));

    let api_event_fd2 = cli_instance
        .to_vmm_fd
        .try_clone()
        .expect("Failed to dup eventfd");
    let vmm = Vmm::new(
        cli_instance.vmm_shared_info.clone(),
        api_event_fd2,
        cli_instance.seccomp.clone(),
        cli_instance.seccomp.clone(),
        Some(kvm.into_raw_fd()),
    )
    .expect("Failed to start vmm");

    let api_event_fd3 = cli_instance
        .to_vmm_fd
        .try_clone()
        .expect("Failed to dup eventfd");

    let mut api_server = ApiServer::new(
        cli_instance.to_vmm.clone(),
        cli_instance.from_vmm.clone(),
        api_event_fd3,
    );

    // clone the arguments for other thread to use
    let clone_args = create_args.clone();
    thread::Builder::new()
        .name("set_cfg".to_owned())
        .spawn(move || {
            cli_instance
                .run_vmm_server(clone_args)
                .expect("Failed to run server.");
        })
        .unwrap();

    if !api_sock_path.is_empty() {
        let clone_api_sock_path = api_sock_path.to_string().clone();
        thread::Builder::new()
            .name("api_server".to_owned())
            .spawn(move || {
                api_server
                    .run_api_server(clone_api_sock_path)
                    .expect("Failed to api server.");
            })
            .unwrap();
    } else {
        println!("Warning: api server is not created because --api-sock-path is not provided when creating VM. Update command is not supported.");
    }

    Ok(Vmm::run_vmm_event_loop(
        Arc::new(Mutex::new(vmm)),
        vmm_service,
    ))
}
