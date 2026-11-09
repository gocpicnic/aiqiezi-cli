/*
  crates/aiqiezi-cli/src/main.rs
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
*/

mod cli;
mod commands;
mod daemon;
mod daemon_commands;
mod lock;
mod logging;
mod shutdown;
mod version;

use std::process::ExitCode;

use aiqiezi_client::ClientConfig;
use clap::Parser;

use crate::cli::{Cli, Command};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    logging::init()?;
    let config = ClientConfig::from_env()?;
    let json = cli.json;

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    runtime.block_on(async {
        match cli.command {
            Command::Start { endpoint_id } => commands::start::run(config, endpoint_id).await,
            Command::Stop => commands::stop::run(&config).await,
            Command::Status => commands::status::run(&config).await,
            Command::Reconnect => commands::reconnect::run(&config).await,
            Command::Whoami => commands::whoami::run(&config, json),
            Command::SetServer { endpoint_id } => {
                commands::set_server::run(&config, &endpoint_id, json).await
            }
            Command::Login { password_stdin } => {
                commands::login::run(&config, password_stdin).await
            }
            Command::ChangePassword { password_stdin } => {
                commands::change_password::run(&config, password_stdin).await
            }
            Command::ServerInfo => commands::server_info::run(&config).await,
        }
    })
}
