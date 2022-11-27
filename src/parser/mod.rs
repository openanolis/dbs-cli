// Copyright (C) 2020-2022 Alibaba Cloud. All rights reserved.
// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::{
    fs::OpenOptions,
    os::unix::io::IntoRawFd,
    sync::{mpsc::channel, Arc, Mutex},
    thread,
};

use anyhow::Result;

pub use args::Commands;
pub use args::DBSArgs;
use dragonball::{api::v1::VmmService, Vmm};

use crate::api_server::ApiServer;
use crate::cli_instance::CliInstance;

pub mod args;

const KVM_DEVICE: &str = "/dev/kvm";

pub fn run_with_cli(args: DBSArgs) -> Result<i32> {
    let mut cli_instance = CliInstance::new("dbs-cli");

    let kvm = OpenOptions::new().read(true).write(true).open(KVM_DEVICE)?;

    let (to_vmm, from_runtime) = channel();
    let (to_runtime, from_vmm) = channel();

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
    let clone_args = args.clone();
    thread::Builder::new()
        .name("set_cfg".to_owned())
        .spawn(move || {
            cli_instance
                .run_vmm_server(clone_args)
                .expect("Failed to run server.");
        })
        .unwrap();

    println!("[CHAOTEST] 2");
    if !args.api_sock_path.is_empty() {
        thread::Builder::new()
            .name("api_server".to_owned())
            .spawn(move || {
                api_server
                    .run_api_server(&args.api_sock_path)
                    .expect("Failed to api server.");
            })
            .unwrap();
    }

    Ok(Vmm::run_vmm_event_loop(
        Arc::new(Mutex::new(vmm)),
        vmm_service,
    ))
}
