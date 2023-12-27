// #![allow(unused)]

extern crate slog_term;

use anyhow::Result;
use api_client::run_api_client;
use clap::Parser;
use parser::run_with_cli;

use crate::parser::args::{Commands, DBSArgs};

use dragonball::api::v1::NetworkInterfaceConfig;
use slog::Drain;
use slog::*;
use slog_scope::set_global_logger;

use std::str::FromStr;
use std::sync::Mutex;

mod api_client;
mod api_server;
mod cli_instance;
mod parser;
mod utils;
mod vmm_comm_trait;

fn main() -> Result<()> {
    let args: DBSArgs = DBSArgs::parse();
    match args.command {
        Some(Commands::Create { create_args }) => {
            // let log_file = &create_args.log_file;
            // let log_level = Level::from_str(&create_args.log_level).unwrap();

            // let file = std::fs::OpenOptions::new()
            //     .truncate(true)
            //     .read(true)
            //     .create(true)
            //     .write(true)
            //     .open(log_file)
            //     .expect("Cannot write to the log file.");

            // let root = slog::Logger::root(
            //     Mutex::new(slog_json::Json::default(file).filter_level(log_level)).map(slog::Fuse),
            //     o!("version" => env!("CARGO_PKG_VERSION")),
            // );

            // let _guard = set_global_logger(root);
            // slog_stdlog::init().unwrap();
            utils::setup_db_log(&create_args.log_file, &create_args.log_level);
            run_with_cli(create_args, &args.api_sock_path)?;
        }
        Some(Commands::Update { update_args }) => {
            run_api_client(update_args, &args.api_sock_path)?;
        }
        _ => {
            panic!("Invalid command provided for dbs-cli.");
        }
    }
    Ok(())
}
