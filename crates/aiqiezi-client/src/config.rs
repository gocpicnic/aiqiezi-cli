/*
  crates/aiqiezi-client/src/config.rs
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
*/

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context;
use iroh::EndpointId;
use serde::{Deserialize, Serialize};

const CONFIG_FILE: &str = "config/client.json";
const KEY_FILE: &str = "key_data/secret_key.bin";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ClientConfigFile {
    #[serde(default)]
    server_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub data_dir: PathBuf,
    pub runtime_dir: PathBuf,
    pub config_path: PathBuf,
    pub key_path: PathBuf,
    pub socket_path: PathBuf,
    pub server_id: Option<EndpointId>,
}

impl ClientConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let data_dir = std::env::var_os("AIQIEZI_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("./data"));
        let runtime_dir = std::env::var_os("AIQIEZI_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| data_dir.join("run"));

        Self::load(data_dir, runtime_dir)
    }

    pub fn load(data_dir: PathBuf, runtime_dir: PathBuf) -> anyhow::Result<Self> {
        let config_path = data_dir.join(CONFIG_FILE);
        let key_path = data_dir.join(KEY_FILE);
        let socket_path = runtime_dir.join("client.sock");

        let server_id =
            if config_path.exists() {
                let contents = fs::read_to_string(&config_path)
                    .with_context(|| format!("read client config {}", config_path.display()))?;
                let file: ClientConfigFile = serde_json::from_str(&contents)
                    .with_context(|| format!("parse client config {}", config_path.display()))?;
                match file.server_id {
                    Some(value) => Some(value.parse::<EndpointId>().with_context(|| {
                        format!("invalid server_id in {}", config_path.display())
                    })?),
                    None => None,
                }
            } else {
                None
            };

        Ok(Self {
            data_dir,
            runtime_dir,
            config_path,
            key_path,
            socket_path,
            server_id,
        })
    }

    pub fn save_server_id(&self, server_id: EndpointId) -> anyhow::Result<()> {
        let config_dir = self
            .config_path
            .parent()
            .context("client config path has no parent")?;
        create_private_dir(config_dir)?;

        let file = ClientConfigFile {
            server_id: Some(server_id.to_string()),
        };
        let encoded = serde_json::to_vec_pretty(&file).context("encode client config")?;
        write_private_file(&self.config_path, &encoded)
    }

    pub fn require_server_id(&self) -> anyhow::Result<EndpointId> {
        self.server_id.ok_or_else(|| {
            anyhow::anyhow!(
                "server_id is not configured; run `aiqiezi start <endpoint-id>` or `aiqiezi set-server <endpoint-id>`"
            )
        })
    }
}

pub fn create_private_dir(path: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(path).with_context(|| format!("create directory {}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))
            .with_context(|| format!("set permissions on {}", path.display()))?;
    }
    Ok(())
}

pub fn write_private_file(path: &Path, contents: &[u8]) -> anyhow::Result<()> {
    let temporary = path.with_extension("tmp");
    fs::write(&temporary, contents)
        .with_context(|| format!("write temporary file {}", temporary.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&temporary, fs::Permissions::from_mode(0o600))
            .with_context(|| format!("set permissions on {}", temporary.display()))?;
    }
    fs::rename(&temporary, path).with_context(|| format!("replace {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ClientConfig, CONFIG_FILE};
    use iroh::SecretKey;

    fn endpoint_id() -> iroh::EndpointId {
        SecretKey::generate().public()
    }

    #[test]
    fn missing_config_has_no_server_id() {
        let dir = tempfile::tempdir().unwrap();
        let config = ClientConfig::load(dir.path().to_path_buf(), dir.path().join("run")).unwrap();
        assert!(config.server_id.is_none());
        assert!(config.require_server_id().is_err());
    }

    #[test]
    fn server_id_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let config = ClientConfig::load(dir.path().to_path_buf(), dir.path().join("run")).unwrap();
        let server_id = endpoint_id();

        config.save_server_id(server_id).unwrap();
        let loaded = ClientConfig::load(dir.path().to_path_buf(), dir.path().join("run")).unwrap();

        assert_eq!(loaded.server_id, Some(server_id));
        assert!(dir.path().join(CONFIG_FILE).exists());
    }

    #[test]
    fn invalid_server_id_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(CONFIG_FILE);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, r#"{"server_id":"not-an-endpoint-id"}"#).unwrap();

        assert!(ClientConfig::load(dir.path().to_path_buf(), dir.path().join("run")).is_err());
    }
}
