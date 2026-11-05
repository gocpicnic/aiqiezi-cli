/*
  crates/aiqiezi-cli/src/commands/start.rs
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
*/

use aiqiezi_client::ClientConfig;

pub async fn run(config: ClientConfig) -> anyhow::Result<()> {
    crate::daemon::run(config).await
}
