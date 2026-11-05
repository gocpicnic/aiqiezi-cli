/*
  crates/aiqiezi-cli/src/commands/login.rs
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
*/

use aiqiezi_client::ClientConfig;
use aiqiezi_proto::error_codes;
use serde_json::json;

use super::{ensure_ok, local_request, print_response};

pub async fn run(config: &ClientConfig) -> anyhow::Result<()> {
    let password = rpassword::prompt_password("password: ")?;
    if password.is_empty() {
        anyhow::bail!("password must not be empty");
    }

    let response = local_request(config, "login", json!({"password": password})).await?;
    print_response(&response)?;

    if let Some(error) = &response.error {
        if error.code == error_codes::PASSWORD_CHANGE_REQUIRED {
            eprintln!("bootstrap password must be changed: run `aiqiezi change-password`");
            return Ok(());
        }
    }

    ensure_ok(&response)
}
