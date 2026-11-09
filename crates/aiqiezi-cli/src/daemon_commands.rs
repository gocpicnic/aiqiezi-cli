/*
  crates/aiqiezi-cli/src/daemon_commands.rs
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
*/

use aiqiezi_client::{backend, EndpointId};
use aiqiezi_proto::{error_codes, Request, Response, ALPN_AUTH, ALPN_JSON};
use serde_json::{json, Value};

use crate::daemon::AppState;

pub async fn execute(state: &AppState, request: Request) -> Response {
    let request_id = request.request_id;
    match request.command.as_str() {
        "status" => status(state, request_id).await,
        "set-server" => set_server(state, request_id, &request.params).await,
        "shutdown" => shutdown(state, request_id),
        "login" => login(state, request_id, &request.params).await,
        "change-password" => change_password(state, request_id, &request.params).await,
        "server-info" => server_info(state, request_id).await,
        "reconnect" => reconnect(state, request_id).await,
        _ => Response::error(
            request_id,
            error_codes::UNKNOWN_COMMAND,
            format!("unknown local command: {}", request.command),
        ),
    }
}

async fn status(state: &AppState, request_id: Option<String>) -> Response {
    let server_id = *state.server_id.read().await;
    Response::success(
        request_id,
        json!({
            "client_id": state.client_id.to_string(),
            "server_id": server_id.map(|endpoint_id| endpoint_id.to_string()),
            "uptime_secs": state.started.elapsed().as_secs(),
            "version": crate::version::VERSION,
            "socket": state.config.socket_path.display().to_string(),
        }),
    )
}

async fn set_server(state: &AppState, request_id: Option<String>, params: &Value) -> Response {
    let Some(value) = params.get("endpoint_id").and_then(Value::as_str) else {
        return Response::error(
            request_id,
            error_codes::INVALID_REQUEST,
            "set-server requires endpoint_id",
        );
    };
    let endpoint_id = match value.parse::<EndpointId>() {
        Ok(endpoint_id) => endpoint_id,
        Err(error) => {
            return Response::error(
                request_id,
                error_codes::INVALID_REQUEST,
                format!("invalid EndpointId: {error}"),
            );
        }
    };

    if let Err(error) = state.config.save_server_id(endpoint_id) {
        return Response::error(
            request_id,
            error_codes::INTERNAL,
            format!("failed to save server_id: {error:#}"),
        );
    }
    *state.server_id.write().await = Some(endpoint_id);

    Response::success(request_id, json!({"server_id": endpoint_id.to_string()}))
}

fn shutdown(state: &AppState, request_id: Option<String>) -> Response {
    state.shutdown.trigger();
    Response::success(request_id, json!({"message": "shutting down"}))
}

async fn login(state: &AppState, request_id: Option<String>, params: &Value) -> Response {
    let Some(password) = params.get("password").and_then(Value::as_str) else {
        return Response::error(
            request_id,
            error_codes::INVALID_REQUEST,
            "login requires password",
        );
    };

    backend_request(
        state,
        request_id,
        ALPN_AUTH,
        "login",
        json!({"password": password}),
        true,
    )
    .await
}

async fn change_password(state: &AppState, request_id: Option<String>, params: &Value) -> Response {
    let old_password = params
        .get("old_password")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let new_password = params
        .get("new_password")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if old_password.is_empty() || new_password.is_empty() {
        return Response::error(
            request_id,
            error_codes::INVALID_REQUEST,
            "change-password requires old_password and new_password",
        );
    }

    backend_request(
        state,
        request_id,
        ALPN_AUTH,
        "change-password",
        json!({
            "old_password": old_password,
            "new_password": new_password,
        }),
        false,
    )
    .await
}

async fn server_info(state: &AppState, request_id: Option<String>) -> Response {
    backend_request(state, request_id, ALPN_JSON, "server-info", json!({}), true).await
}

async fn reconnect(state: &AppState, request_id: Option<String>) -> Response {
    state.endpoint.network_change().await;
    Response::success(request_id, json!({"message": "network state refreshed"}))
}

async fn backend_request(
    state: &AppState,
    request_id: Option<String>,
    alpn: &[u8],
    command: &str,
    params: Value,
    retry_on_timeout: bool,
) -> Response {
    let server_id = *state.server_id.read().await;
    let Some(server_id) = server_id else {
        return Response::error(
            request_id,
            error_codes::AUTH_UNAVAILABLE,
            "server_id is not configured; restart with `aiqiezi start <endpoint-id>` or run `aiqiezi set-server <endpoint-id>`",
        );
    };

    let result = if retry_on_timeout {
        backend::request_with_network_retry(&state.endpoint, server_id, alpn, command, params).await
    } else {
        backend::request(&state.endpoint, server_id, alpn, command, params).await
    };

    match result {
        Ok(mut response) => {
            response.request_id = request_id;
            response
        }
        Err(error) => Response::error(
            request_id,
            error_codes::INTERNAL,
            format!("backend request failed: {error:#}"),
        ),
    }
}
