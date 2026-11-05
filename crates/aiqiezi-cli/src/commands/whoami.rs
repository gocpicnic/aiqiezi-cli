/*
  crates/aiqiezi-cli/src/commands/whoami.rs
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
*/

use aiqiezi_client::{key, ClientConfig};
use serde_json::json;

pub fn run(config: &ClientConfig, json: bool) -> anyhow::Result<()> {
    let key = key::load_key(config)?;
    let endpoint_id = key.public();

    if json {
        println!(
            "{}",
            serde_json::to_string(&json!({"endpoint_id": endpoint_id.to_string()}))?
        );
    } else {
        println!("{endpoint_id}");
    }
    Ok(())
}
