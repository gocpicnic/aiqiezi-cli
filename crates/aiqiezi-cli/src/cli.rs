/*
  crates/aiqiezi-cli/src/cli.rs
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
*/

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "aiqiezi",
    version = crate::version::VERSION,
    about = "aiqiezi photo and video archive client"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Start the local client daemon in the foreground
    #[command(visible_alias = "run")]
    Start,
    /// Stop the running local client daemon
    Stop,
    /// Query the local client daemon status
    Status,
    /// Print the client's persistent EndpointId
    Whoami,
    /// Set the backend server EndpointId
    SetServer {
        /// Backend EndpointId
        endpoint_id: String,
    },
    /// Log in to the backend with the bootstrap password
    Login,
    /// Change the backend password
    ChangePassword,
    /// Request server-info from the backend
    ServerInfo,
}
