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
    /// Print machine-readable JSON where the command supports it
    #[arg(long, global = true)]
    pub json: bool,

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
    Login {
        /// Read the password from stdin instead of prompting
        #[arg(long)]
        password_stdin: bool,
    },
    /// Change the backend password
    ChangePassword {
        /// Read old password, new password, and confirmation from stdin
        #[arg(long)]
        password_stdin: bool,
    },
    /// Request server-info from the backend
    ServerInfo,
}

#[cfg(test)]
mod tests {
    use super::{Cli, Command};
    use clap::Parser;

    #[test]
    fn parses_login_password_stdin() {
        let cli = Cli::try_parse_from(["aiqiezi", "login", "--password-stdin"]).unwrap();
        assert!(!cli.json);
        assert!(matches!(
            cli.command,
            Command::Login {
                password_stdin: true
            }
        ));
    }

    #[test]
    fn parses_global_json_after_subcommand() {
        let cli = Cli::try_parse_from(["aiqiezi", "whoami", "--json"]).unwrap();
        assert!(cli.json);
        assert!(matches!(cli.command, Command::Whoami));
    }
}
