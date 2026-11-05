/*
  crates/aiqiezi-cli/src/commands/mod.rs
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
*/

pub mod change_password;
pub mod login;
pub mod server_info;
pub mod set_server;
pub mod start;
pub mod status;
pub mod stop;
pub mod whoami;

use aiqiezi_client::{local, ClientConfig};
use aiqiezi_proto::{Request, Response, PROTOCOL_VERSION};
use anyhow::Context;
use serde_json::Value;

pub async fn local_request(
    config: &ClientConfig,
    command: &str,
    params: Value,
) -> anyhow::Result<Response> {
    let request = Request {
        version: PROTOCOL_VERSION,
        request_id: Some(format!("cli-{}", std::process::id())),
        command: command.to_string(),
        params,
    };

    local::request(&config.socket_path, &request)
        .await
        .with_context(|| {
            format!(
                "client daemon is not running; start it with `aiqiezi start` (socket: {})",
                config.socket_path.display()
            )
        })
}

pub fn print_response(response: &Response) -> anyhow::Result<()> {
    println!(
        "{}",
        serde_json::to_string_pretty(response).context("encode response for output")?
    );
    Ok(())
}

pub fn ensure_ok(response: &Response) -> anyhow::Result<()> {
    if response.ok {
        return Ok(());
    }

    anyhow::bail!(
        "{}",
        response
            .error
            .as_ref()
            .map(|error| format!("{}: {}", error.code, error.message))
            .unwrap_or_else(|| "unknown server error".to_string())
    )
}
