/*
  crates/aiqiezi-cli/src/commands/start.rs
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
*/

use aiqiezi_client::{ClientConfig, EndpointId};
use anyhow::Context;

pub async fn run(config: ClientConfig, endpoint_id: Option<String>) -> anyhow::Result<()> {
    let endpoint_id = parse_server_endpoint_id(endpoint_id)?;
    crate::daemon::run(config, endpoint_id).await
}

fn parse_server_endpoint_id(value: Option<String>) -> anyhow::Result<Option<EndpointId>> {
    value
        .as_deref()
        .map(str::parse::<EndpointId>)
        .transpose()
        .context("invalid aiqiezi-server EndpointId")
}

#[cfg(test)]
mod tests {
    use super::parse_server_endpoint_id;
    use iroh::SecretKey;

    #[test]
    fn absent_endpoint_id_is_allowed() {
        assert_eq!(parse_server_endpoint_id(None).unwrap(), None);
    }

    #[test]
    fn valid_endpoint_id_is_parsed() {
        let endpoint_id = SecretKey::generate().public();
        let parsed = parse_server_endpoint_id(Some(endpoint_id.to_string()))
            .unwrap()
            .unwrap();
        assert_eq!(parsed, endpoint_id);
    }

    #[test]
    fn invalid_endpoint_id_is_rejected() {
        assert!(parse_server_endpoint_id(Some("not-an-endpoint-id".to_string())).is_err());
    }
}
