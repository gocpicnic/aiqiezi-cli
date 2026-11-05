/*
  crates/aiqiezi-cli/src/commands/whoami.rs
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
*/

use aiqiezi_client::{key, ClientConfig};

pub fn run(config: &ClientConfig) -> anyhow::Result<()> {
    let key = key::load_key(config)?;
    println!("{}", key.public());
    Ok(())
}
