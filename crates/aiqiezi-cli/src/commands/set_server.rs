/*
  crates/aiqiezi-cli/src/commands/set_server.rs
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
*/

use aiqiezi_client::{local, ClientConfig, EndpointId};
use aiqiezi_proto::{Request, PROTOCOL_VERSION};
use anyhow::Context;
use serde_json::json;

use super::{ensure_ok, print_response};

pub async fn run(config: &ClientConfig, endpoint_id: &str) -> anyhow::Result<()> {
    let endpoint_id = endpoint_id
        .parse::<EndpointId>()
        .context("invalid server EndpointId")?;
    let request = Request {
        version: PROTOCOL_VERSION,
        request_id: Some(format!("cli-{}", std::process::id())),
        command: "set-server".to_string(),
        params: json!({"endpoint_id": endpoint_id.to_string()}),
    };

    match local::request(&config.socket_path, &request).await {
        Ok(response) => {
            print_response(&response)?;
            ensure_ok(&response)?;
        }
        Err(_) => {
            config.save_server_id(endpoint_id)?;
            println!("server id saved to {}", config.config_path.display());
        }
    }

    Ok(())
}
