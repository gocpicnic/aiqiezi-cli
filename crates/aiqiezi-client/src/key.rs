/*
  crates/aiqiezi-client/src/key.rs
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
*/

use std::fs;
use std::path::Path;

use anyhow::Context;
use iroh::SecretKey;

use crate::config::{create_private_dir, ClientConfig};

pub fn load_key(config: &ClientConfig) -> anyhow::Result<SecretKey> {
    if !config.key_path.exists() {
        anyhow::bail!(
            "client key not found at {}; start the client daemon first",
            config.key_path.display()
        );
    }

    read_key(&config.key_path)
}

pub fn load_or_create_key(config: &ClientConfig) -> anyhow::Result<SecretKey> {
    let key_dir = config
        .key_path
        .parent()
        .context("client key path has no parent")?;
    create_private_dir(key_dir)?;

    if config.key_path.exists() {
        return read_key(&config.key_path);
    }

    let key = SecretKey::generate();
    fs::write(&config.key_path, key.to_bytes())
        .with_context(|| format!("write client key {}", config.key_path.display()))?;
    set_private_permissions(&config.key_path)?;
    Ok(key)
}

fn read_key(path: &Path) -> anyhow::Result<SecretKey> {
    let bytes = fs::read(path).with_context(|| format!("read client key {}", path.display()))?;
    let bytes: [u8; 32] = bytes.try_into().map_err(|bytes: Vec<u8>| {
        anyhow::anyhow!(
            "client key {} has invalid length: expected 32 bytes, got {}",
            path.display(),
            bytes.len()
        )
    })?;
    set_private_permissions(path)?;
    Ok(SecretKey::from_bytes(&bytes))
}

fn set_private_permissions(path: &Path) -> anyhow::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .with_context(|| format!("set permissions on {}", path.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{load_key, load_or_create_key};
    use crate::config::ClientConfig;

    fn config(dir: &std::path::Path) -> ClientConfig {
        ClientConfig::load(dir.to_path_buf(), dir.join("run")).unwrap()
    }

    #[test]
    fn key_is_stable_and_loadable() {
        let dir = tempfile::tempdir().unwrap();
        let config = config(dir.path());

        let created = load_or_create_key(&config).unwrap();
        let loaded = load_key(&config).unwrap();

        assert_eq!(created.public(), loaded.public());
    }

    #[test]
    fn missing_key_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let config = config(dir.path());
        assert!(load_key(&config).is_err());
    }

    #[test]
    fn invalid_key_length_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let config = config(dir.path());
        std::fs::create_dir_all(config.key_path.parent().unwrap()).unwrap();
        std::fs::write(&config.key_path, [0u8; 16]).unwrap();

        assert!(load_key(&config).is_err());
    }
}
