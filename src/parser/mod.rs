// Copyright (C) 2020-2022 Alibaba Cloud. All rights reserved.
// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::{
    fs::OpenOptions,
    os::unix::io::IntoRawFd,
    sync::{mpsc::channel, Arc, Mutex},
    thread,
    //   path::Path
};

//use firec::{config::{network::Interface, Config}};

use anyhow::Result;

pub use args::{DBSArgs, FCArgs};
use dragonball::{api::v1::VmmService, Vmm};

use crate::cli_instance::CliInstance;

pub mod args;

const KVM_DEVICE: &str = "/dev/kvm";

pub fn run_with_cli(args: DBSArgs) -> Result<i32> {
    let mut cli_instance = CliInstance::new("fc-cli");

    let kvm = OpenOptions::new().read(true).write(true).open(KVM_DEVICE)?;

    let (to_vmm, from_runtime) = channel();
    let (to_runtime, from_vmm) = channel();

    let vmm_service = VmmService::new(from_runtime, to_runtime);

    cli_instance.to_vmm = Some(to_vmm);
    cli_instance.from_vmm = Some(from_vmm);

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

    thread::Builder::new()
        .name("set configuration".to_owned())
        .spawn(move || {
            cli_instance
                .run_vmm_server(args)
                .expect("Failed to run server.");
        })
        .unwrap();

    Ok(Vmm::run_vmm_event_loop(
        Arc::new(Mutex::new(vmm)),
        vmm_service,
    ))
}
/*
pub fn run_with_fc_cli(args: FCArgs)->Config{
    let iface = Interface::new("eth0", "tap0");

    let config = Config::builder(None, Path::new(&args.boot_args.kernel_path))
        .jailer_cfg()
            .chroot_base_dir(Path::new("/srv"))
            .exec_file(Path::new(&args.boot_args.kernel_path))
            .jailer_binary(Path::new(&args.boot_args.kernel_path))
            .build()
        .kernel_args(&args.boot_args.boot_args)
        .machine_cfg()
            .vcpu_count(args.create_args.vcpu_count)
            .mem_size_mib(args.create_args.mem_size_mib)
            .build()
        .add_drive("root", Path::new(&args.boot_args.rootfs_args.rootfs))
            .is_root_device(args.boot_args.rootfs_args.is_root)
            .build()
        .add_network_interface(iface)
        .socket_path(Path::new(&args.create_args.socket_path))
        .build();
    config
}*/
