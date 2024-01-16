// #![allow(unused)]

extern crate slog_term;

use anyhow::Result;
use api_client::run_api_client;
use clap::Parser;
use parser::run_with_cli;

use crate::parser::args::{Commands, DBSArgs};

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
