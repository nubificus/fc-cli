// #![allow(unused)]

extern crate slog_term;

use std::str::FromStr;
use std::sync::Mutex;

use firec::{
    Action,
    FreeMachine,
};
use hyperlocal::Uri;
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
    let mut machine = FreeMachine::create_with_args(
        Path::new(&args.boot_args.fc_path),
        Path::new(&args.create_args.socket_path),
        ).await?;

    machine.start().await?;

    let url_kernel = Uri::new(machine.as_path(), "/boot-source").into();
    let url_rootfs = Uri::new(machine.as_path(), "/drives/rootfs").into();
    let body_kernel: String = "
        {
            \"kernel_image_path\":\"".to_string()+&args.boot_args.kernel_path+"\" ,
            \"boot_args\": \"console=ttyS0 reboot=k panic=1 pci=off\"
        }";
    let body_rootfs: String = "
        {
            \"drive_id\": \"rootfs\",
            \"path_on_host\": \"".to_string()+&args.boot_args.rootfs_args.rootfs+"\",
            \"is_root_device\": true,
            \"is_read_only\": false
        }";

    while !machine.as_path().exists() {}

    machine.send_request(url_kernel, body_kernel).await?;
    machine.send_request(url_rootfs, body_rootfs).await?;

    machine.send_action(Action::InstanceStart).await?;

    //it
    //should
    //be
    //sleep
    //tbh

    sleep(Duration::from_secs(500)).await;
    //machine.force_shutdown().await?;

    //run_with_cli(args)?;
    Ok(())
}
