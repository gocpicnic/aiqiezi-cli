/*
  crates/aiqiezi-cli/src/logging.rs
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
*/

use std::io::IsTerminal;

use tracing_subscriber::EnvFilter;

pub fn init() -> anyhow::Result<()> {
    let filter = std::env::var("AIQIEZI_LOG_LEVEL")
        .unwrap_or_else(|_| "aiqiezi_cli=info,aiqiezi_client=info,iroh=warn".to_string());
    let filter = EnvFilter::try_new(filter).map_err(|error| anyhow::anyhow!("{error}"))?;

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_ansi(std::io::stderr().is_terminal())
        .try_init()
        .map_err(|error| anyhow::anyhow!("failed to install tracing subscriber: {error}"))
}
