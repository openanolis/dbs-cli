// #![allow(unused)]

extern crate slog_term;

use std::str::FromStr;
use std::sync::Mutex;

use anyhow::Result;
use api_client::run_api_client;
use clap::Parser;
use slog::Drain;
use slog::*;
use slog_scope::set_global_logger;

use parser::run_with_cli;
use parser::Commands;
use parser::DBSArgs;

mod api_client;
mod api_server;
mod cli_instance;
mod parser;
mod vmm_comm_trait;

fn main() -> Result<()> {
    let args: DBSArgs = DBSArgs::parse();
    match args.command {
        Some(Commands::Create) => {
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
            slog_stdlog::init().unwrap();
            run_with_cli(args)?;
        }
        Some(Commands::Update) => {
            run_api_client(args)?;
        }
        _ => {
            panic!("Invalid command provided for dbs-cli.");
        }
    }
    Ok(())
}
