/*
  crates/aiqiezi-client/src/firewall.rs
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
*/

use std::sync::Arc;

use iroh::endpoint::{AfterHandshakeOutcome, BeforeConnectOutcome, Connection, EndpointHooks};
use iroh::EndpointId;
use tokio::sync::RwLock;
use tracing::{debug, warn};

#[derive(Debug, Clone)]
pub struct ClientHooks {
    server_id: Arc<RwLock<Option<EndpointId>>>,
}

impl ClientHooks {
    pub fn new(server_id: Arc<RwLock<Option<EndpointId>>>) -> Self {
        Self { server_id }
    }
}

impl EndpointHooks for ClientHooks {
    async fn before_connect(
        &self,
        _remote_addr: &iroh::EndpointAddr,
        _alpn: &[u8],
    ) -> BeforeConnectOutcome {
        BeforeConnectOutcome::Accept
    }

    async fn after_handshake(&self, conn: &Connection) -> AfterHandshakeOutcome {
        let server_id = *self.server_id.read().await;
        if is_allowed(server_id, conn.remote_id()) {
            debug!("accepted iroh connection from configured server");
            AfterHandshakeOutcome::Accept
        } else {
            warn!(
                "rejected iroh connection from {}, expected server {}",
                conn.remote_id(),
                server_id
                    .map(|endpoint_id| endpoint_id.to_string())
                    .unwrap_or_else(|| "<not configured>".to_string())
            );
            AfterHandshakeOutcome::Reject {
                error_code: 403u32.into(),
                reason: b"unauthorized".to_vec(),
            }
        }
    }
}

fn is_allowed(server_id: Option<EndpointId>, remote_id: EndpointId) -> bool {
    server_id == Some(remote_id)
}

#[cfg(test)]
mod tests {
    use super::is_allowed;
    use iroh::SecretKey;

    fn endpoint_id() -> iroh::EndpointId {
        SecretKey::generate().public()
    }

    #[test]
    fn only_the_configured_server_is_allowed() {
        let server = endpoint_id();
        let other = endpoint_id();

        assert!(is_allowed(Some(server), server));
        assert!(!is_allowed(Some(server), other));
        assert!(!is_allowed(None, server));
    }
}
