// #![allow(unused)]

extern crate slog_term;

use std::str::FromStr;
use std::sync::Mutex;

use firec::{
    config::{network::Interface, Config}, //Drive, Jailer, Machine as MachineCfg
    Machine,
};
use std::path::Path;
use tokio::time::{sleep, Duration};

use anyhow::Result;
use clap::Parser;
use slog::Drain;
use slog::*;
use slog_scope::set_global_logger;

use parser::run_with_cli;
use parser::FCArgs;

mod cli_instance;
mod parser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: FCArgs = FCArgs::parse();
    /*
    let log_file = &args.log_file;
    let log_level = Level::from_str(&args.log_level).unwrap();

    let file = std::fs::OpenOptions::new()
        .truncate(true)
        .read(true)
        .create(true)
        .write(true)
        .open(log_file)
        .expect("Cannot write to the log file.");

    let root = slog::Logger::root(
        Mutex::new(slog_json::Json::default(file).filter_level(log_level)).map(slog::Fuse),
        o!("version" => env!("CARGO_PKG_VERSION")),
    );

    let _guard = set_global_logger(root);
    */
    let iface = Interface::new("eth0", "tap0");

    let config = Config::builder(None, Path::new(&args.boot_args.kernel_path))
        .jailer_cfg()
            .chroot_base_dir(Path::new("/srv"))
            .exec_file(Path::new(&args.boot_args.fc_path))
            .jailer_binary(Path::new(&args.boot_args.jailer_path))
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

    let mut machine = Machine::create(config).await?;

    machine.start().await?;
    //sleep(Duration::from_secs(5)).await;
    //machine.force_shutdown().await?;

    //run_with_cli(args)?;
    Ok(())
}
