/*
  crates/aiqiezi-cli/src/commands/mod.rs
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
*/

pub mod change_password;
pub mod login;
pub mod reconnect;
pub mod server_info;
pub mod set_server;
pub mod start;
pub mod status;
pub mod stop;
pub mod whoami;

use std::io::BufRead;

use aiqiezi_client::{local, ClientConfig};
use aiqiezi_proto::{Request, Response, PROTOCOL_VERSION};
use anyhow::Context;
use serde_json::Value;

pub fn read_secret_line(reader: &mut impl BufRead) -> anyhow::Result<String> {
    let mut line = String::new();
    let bytes = reader
        .read_line(&mut line)
        .context("read password from stdin")?;
    if bytes == 0 {
        anyhow::bail!("stdin closed before a password was provided");
    }

    let value = line.trim_end_matches(['\r', '\n']).to_string();
    if value.is_empty() {
        anyhow::bail!("stdin password must not be empty");
    }
    Ok(value)
}

pub async fn local_request(
    config: &ClientConfig,
    command: &str,
    params: Value,
) -> anyhow::Result<Response> {
    let request = Request {
        version: PROTOCOL_VERSION,
        request_id: Some(format!("cli-{}", std::process::id())),
        command: command.to_string(),
        params,
    };

    local::request(&config.socket_path, &request)
        .await
        .with_context(|| {
            format!(
                "client daemon is not running; start it with `aiqiezi start` (socket: {})",
                config.socket_path.display()
            )
        })
}

pub fn print_response(response: &Response) -> anyhow::Result<()> {
    println!(
        "{}",
        serde_json::to_string_pretty(response).context("encode response for output")?
    );
    Ok(())
}

pub fn ensure_ok(response: &Response) -> anyhow::Result<()> {
    if response.ok {
        return Ok(());
    }

    anyhow::bail!(
        "{}",
        response
            .error
            .as_ref()
            .map(|error| format!("{}: {}", error.code, error.message))
            .unwrap_or_else(|| "unknown server error".to_string())
    )
}

#[cfg(test)]
mod tests {
    use super::read_secret_line;
    use std::io::Cursor;

    #[test]
    fn reads_passwords_without_trimming_spaces() {
        let mut reader = Cursor::new(b"  secret pass  \r\n");
        assert_eq!(read_secret_line(&mut reader).unwrap(), "  secret pass  ");
    }

    #[test]
    fn rejects_empty_password() {
        let mut reader = Cursor::new(b"\n");
        assert!(read_secret_line(&mut reader).is_err());
    }

    #[test]
    fn rejects_closed_stdin() {
        let mut reader = Cursor::new(b"");
        assert!(read_secret_line(&mut reader).is_err());
    }
}
