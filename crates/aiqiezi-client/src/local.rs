/*
  crates/aiqiezi-client/src/local.rs
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
*/

use std::path::Path;

use aiqiezi_proto::{Request, Response};
use anyhow::Context;
use interprocess::local_socket::{
    tokio::{Listener, Stream},
    traits::tokio::{Listener as _, Stream as _},
    ListenerOptions,
};
#[cfg(unix)]
use interprocess::local_socket::{GenericFilePath, ToFsName};
#[cfg(windows)]
use interprocess::local_socket::{GenericNamespaced, ToNsName};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::config::create_private_dir;

pub type LocalListener = Listener;
pub type LocalStream = Stream;

pub fn bind(path: &Path) -> anyhow::Result<LocalListener> {
    if let Some(parent) = path.parent() {
        create_private_dir(parent)?;
    }

    #[cfg(unix)]
    {
        let name = path
            .to_fs_name::<GenericFilePath>()
            .with_context(|| format!("create local socket name {}", path.display()))?;
        let listener = ListenerOptions::new()
            .name(name)
            .try_overwrite(true)
            .create_tokio()
            .with_context(|| format!("bind local socket {}", path.display()))?;

        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
            .with_context(|| format!("set permissions on {}", path.display()))?;

        Ok(listener)
    }

    #[cfg(windows)]
    {
        let name = pipe_name(path)
            .to_ns_name::<GenericNamespaced>()
            .context("create local pipe name")?;
        ListenerOptions::new()
            .name(name)
            .try_overwrite(true)
            .create_tokio()
            .context("bind local pipe")
    }
}

pub async fn accept(listener: &LocalListener) -> std::io::Result<LocalStream> {
    listener.accept().await
}

pub async fn connect(path: &Path) -> anyhow::Result<LocalStream> {
    #[cfg(unix)]
    let name = path
        .to_fs_name::<GenericFilePath>()
        .with_context(|| format!("create local socket name {}", path.display()))?;

    #[cfg(windows)]
    let name = pipe_name(path)
        .to_ns_name::<GenericNamespaced>()
        .context("create local pipe name")?;

    Stream::connect(name)
        .await
        .with_context(|| format!("connect to local socket {}", path.display()))
}

pub async fn request(path: &Path, request: &Request) -> anyhow::Result<Response> {
    let stream = connect(path).await?;
    let (read_half, mut write_half) = tokio::io::split(stream);

    let mut encoded = serde_json::to_vec(request).context("encode local request")?;
    encoded.push(b'\n');
    write_half
        .write_all(&encoded)
        .await
        .context("write local request")?;
    write_half.flush().await.context("flush local request")?;

    let mut reader = BufReader::new(read_half);
    let mut line = String::new();
    let bytes = reader
        .read_line(&mut line)
        .await
        .context("read local response")?;
    if bytes == 0 {
        anyhow::bail!("local daemon closed without a response");
    }

    serde_json::from_str(&line).context("parse local response")
}

#[cfg(windows)]
fn pipe_name(path: &Path) -> String {
    use std::hash::{Hash, Hasher};

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    path.hash(&mut hasher);
    format!("aiqiezi-client-{:016x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::{bind, request};
    use aiqiezi_proto::{Request, Response, PROTOCOL_VERSION};
    use serde_json::json;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    #[tokio::test]
    async fn local_socket_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let socket_path = dir.path().join("client.sock");
        let listener = bind(&socket_path).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&socket_path)
                .unwrap()
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(mode, 0o600);
        }

        let server = tokio::spawn(async move {
            let stream = super::accept(&listener).await.unwrap();
            let (read_half, mut write_half) = tokio::io::split(stream);
            let mut reader = BufReader::new(read_half);
            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            let request: Request = serde_json::from_str(&line).unwrap();
            let response = Response::success(request.request_id, json!({"ok": true}));
            let mut encoded = serde_json::to_vec(&response).unwrap();
            encoded.push(b'\n');
            write_half.write_all(&encoded).await.unwrap();
            write_half.flush().await.unwrap();
        });

        let response = request(
            &socket_path,
            &Request {
                version: PROTOCOL_VERSION,
                request_id: Some("test".to_string()),
                command: "status".to_string(),
                params: json!({}),
            },
        )
        .await
        .unwrap();

        assert!(response.ok);
        server.await.unwrap();
    }
}
