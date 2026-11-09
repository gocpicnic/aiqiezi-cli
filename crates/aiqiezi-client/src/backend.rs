/*
  crates/aiqiezi-client/src/backend.rs
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
*/

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use aiqiezi_proto::{Request, Response, PROTOCOL_VERSION};
use anyhow::{bail, Context};
use iroh::{Endpoint, EndpointId};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::warn;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const RETRY_ATTEMPT_TIMEOUT: Duration = Duration::from_secs(15);
static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);

pub async fn request(
    endpoint: &Endpoint,
    server_id: EndpointId,
    alpn: &[u8],
    command: &str,
    params: Value,
) -> anyhow::Result<Response> {
    tokio::time::timeout(
        REQUEST_TIMEOUT,
        request_inner(endpoint, server_id, alpn, command, params),
    )
    .await
    .context("backend request timed out")?
}

/// Request a read-only resource, retrying once after notifying iroh about a
/// possible network change. Use only for idempotent commands.
pub async fn request_with_network_retry(
    endpoint: &Endpoint,
    server_id: EndpointId,
    alpn: &[u8],
    command: &str,
    params: Value,
) -> anyhow::Result<Response> {
    match tokio::time::timeout(
        RETRY_ATTEMPT_TIMEOUT,
        request_inner(endpoint, server_id, alpn, command, params.clone()),
    )
    .await
    {
        Ok(result) => result,
        Err(_) => {
            warn!(
                command,
                "backend request timed out; notifying iroh about a network change"
            );
            endpoint.network_change().await;
            tokio::time::timeout(
                RETRY_ATTEMPT_TIMEOUT,
                request_inner(endpoint, server_id, alpn, command, params),
            )
            .await
            .context("backend request timed out after network refresh")?
        }
    }
}

async fn request_inner(
    endpoint: &Endpoint,
    server_id: EndpointId,
    alpn: &[u8],
    command: &str,
    params: Value,
) -> anyhow::Result<Response> {
    let connection = endpoint
        .connect(server_id, alpn)
        .await
        .with_context(|| format!("connect to backend {server_id}"))?;

    if connection.remote_id() != server_id {
        connection.close(403u32.into(), b"wrong server");
        bail!(
            "connected peer is {}, expected server {}",
            connection.remote_id(),
            server_id
        );
    }

    let (mut send, recv) = connection.open_bi().await.context("open backend stream")?;
    let request = Request {
        version: PROTOCOL_VERSION,
        request_id: Some(format!(
            "client-{}-{}",
            std::process::id(),
            REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed)
        )),
        command: command.to_string(),
        params,
    };
    let mut encoded = serde_json::to_vec(&request).context("encode backend request")?;
    encoded.push(b'\n');
    send.write_all(&encoded)
        .await
        .context("write backend request")?;
    send.flush().await.context("flush backend request")?;

    let mut reader = BufReader::new(recv);
    let mut line = String::new();
    let bytes = reader
        .read_line(&mut line)
        .await
        .context("read backend response")?;
    if bytes == 0 {
        bail!("backend closed the stream without a response");
    }

    let response = serde_json::from_str(&line).context("parse backend response")?;
    connection.close(0u32.into(), b"request complete");
    Ok(response)
}
