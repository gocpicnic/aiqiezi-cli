/*
  crates/aiqiezi-cli/src/daemon.rs
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
*/

use std::sync::Arc;
use std::time::Instant;

use aiqiezi_client::config::create_private_dir;
use aiqiezi_client::firewall::ClientHooks;
use aiqiezi_client::{key, local, ClientConfig, EndpointId};
use aiqiezi_proto::{error_codes, Request, Response, PROTOCOL_VERSION};
use anyhow::Context;
use iroh::endpoint::presets;
use iroh::Endpoint;
use iroh_mdns_address_lookup::MdnsAddressLookup;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::daemon_commands;
use crate::lock::ClientLock;
use crate::shutdown::Shutdown;

pub struct AppState {
    pub config: ClientConfig,
    pub endpoint: Endpoint,
    pub client_id: EndpointId,
    pub server_id: Arc<RwLock<Option<EndpointId>>>,
    pub started: Instant,
    pub shutdown: Shutdown,
}

pub async fn run(config: ClientConfig) -> anyhow::Result<()> {
    create_private_dir(&config.data_dir)?;
    create_private_dir(&config.runtime_dir)?;
    let lock = ClientLock::acquire(&config.runtime_dir)?;
    info!(lock = %lock.path().display(), "client lock acquired");

    let key = key::load_or_create_key(&config)?;
    let client_id = key.public();
    let server_id = Arc::new(RwLock::new(config.server_id));
    let hooks = ClientHooks::new(server_id.clone());

    let endpoint = Endpoint::builder(presets::N0)
        .secret_key(key)
        .hooks(hooks)
        .address_lookup(MdnsAddressLookup::builder())
        .bind()
        .await
        .context("bind client endpoint")?;
    info!("client endpoint id: {client_id}");

    let listener = local::bind(&config.socket_path)?;
    info!(socket = %config.socket_path.display(), "local socket listening");

    let shutdown = Shutdown::new();
    let state = Arc::new(AppState {
        config,
        endpoint: endpoint.clone(),
        client_id,
        server_id,
        started: Instant::now(),
        shutdown: shutdown.clone(),
    });

    let signal_task = tokio::spawn(wait_for_signal(shutdown.clone()));
    let result = serve(state, listener).await;

    shutdown.trigger();
    signal_task.abort();
    endpoint.close().await;
    info!("client daemon stopped");
    drop(lock);
    result
}

async fn serve(state: Arc<AppState>, listener: local::LocalListener) -> anyhow::Result<()> {
    loop {
        tokio::select! {
            _ = state.shutdown.wait() => {
                info!("local socket stopping");
                return Ok(());
            }
            accepted = local::accept(&listener) => {
                match accepted {
                    Ok(stream) => {
                        let state = state.clone();
                        tokio::spawn(async move {
                            if let Err(error) = handle_local(stream, state).await {
                                warn!("local command error: {error}");
                            }
                        });
                    }
                    Err(error) => {
                        return Err(error).context("accept local command");
                    }
                }
            }
        }
    }
}

async fn handle_local(stream: local::LocalStream, state: Arc<AppState>) -> anyhow::Result<()> {
    let (read_half, mut write_half) = tokio::io::split(stream);
    let mut reader = BufReader::new(read_half);
    let mut line = String::new();
    let bytes = reader
        .read_line(&mut line)
        .await
        .context("read local command")?;
    if bytes == 0 {
        return Ok(());
    }

    let response = match serde_json::from_str::<Request>(line.trim()) {
        Ok(request) if request.version == PROTOCOL_VERSION => {
            daemon_commands::execute(&state, request).await
        }
        Ok(request) => Response::error(
            request.request_id,
            error_codes::UNSUPPORTED_PROTOCOL,
            format!(
                "unsupported protocol version {}, expected {}",
                request.version, PROTOCOL_VERSION
            ),
        ),
        Err(error) => Response::error(
            None,
            error_codes::INVALID_JSON,
            format!("invalid local JSON request: {error}"),
        ),
    };

    let mut encoded = serde_json::to_vec(&response).context("encode local response")?;
    encoded.push(b'\n');
    write_half
        .write_all(&encoded)
        .await
        .context("write local response")?;
    write_half.flush().await.context("flush local response")?;
    Ok(())
}

async fn wait_for_signal(shutdown: Shutdown) {
    #[cfg(unix)]
    let signal = {
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("install SIGTERM handler");

        tokio::select! {
            _ = tokio::signal::ctrl_c() => "Ctrl+C",
            _ = terminate.recv() => "SIGTERM",
        }
    };

    #[cfg(not(unix))]
    let signal = {
        tokio::signal::ctrl_c().await.ok();
        "Ctrl+C"
    };

    info!("received {signal}, shutting down");
    shutdown.trigger();
}
