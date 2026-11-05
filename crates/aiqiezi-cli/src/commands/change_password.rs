/*
  crates/aiqiezi-cli/src/commands/change_password.rs
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
*/

use aiqiezi_client::ClientConfig;
use serde_json::json;

use super::{ensure_ok, local_request, print_response};

pub async fn run(config: &ClientConfig) -> anyhow::Result<()> {
    let old_password = rpassword::prompt_password("current password: ")?;
    let new_password = rpassword::prompt_password("new password: ")?;
    let confirmation = rpassword::prompt_password("confirm new password: ")?;

    if new_password != confirmation {
        anyhow::bail!("passwords do not match");
    }
    if old_password.is_empty() || new_password.is_empty() {
        anyhow::bail!("passwords must not be empty");
    }

    let response = local_request(
        config,
        "change-password",
        json!({
            "old_password": old_password,
            "new_password": new_password,
        }),
    )
    .await?;
    print_response(&response)?;
    ensure_ok(&response)
}
