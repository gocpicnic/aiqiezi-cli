/*
  crates/aiqiezi-cli/src/commands/status.rs
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
*/

use aiqiezi_client::ClientConfig;
use serde_json::json;

use super::{ensure_ok, local_request, print_response};

pub async fn run(config: &ClientConfig) -> anyhow::Result<()> {
    let response = local_request(config, "status", json!({})).await?;
    print_response(&response)?;
    ensure_ok(&response)
}
