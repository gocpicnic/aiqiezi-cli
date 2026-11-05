/*
  crates/aiqiezi-cli/src/lock.rs
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
*/

use std::fs::{File, OpenOptions, TryLockError};
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use aiqiezi_client::config::create_private_dir;
use anyhow::{bail, Context};

#[derive(Debug)]
pub struct ClientLock {
    file: File,
    path: PathBuf,
}

impl ClientLock {
    pub fn acquire(runtime_dir: &Path) -> anyhow::Result<Self> {
        create_private_dir(runtime_dir)?;

        let path = runtime_dir.join("client.lock");
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(&path)
            .with_context(|| format!("open client lock {}", path.display()))?;

        match file.try_lock() {
            Ok(()) => {}
            Err(TryLockError::WouldBlock) => {
                bail!(
                    "another aiqiezi client daemon holds the lock {}",
                    path.display()
                );
            }
            Err(TryLockError::Error(error)) => {
                return Err(error).with_context(|| format!("lock client file {}", path.display()));
            }
        }

        file.set_len(0)
            .with_context(|| format!("truncate client lock {}", path.display()))?;
        (&file)
            .seek(SeekFrom::Start(0))
            .with_context(|| format!("seek client lock {}", path.display()))?;
        writeln!(&file, "{}", std::process::id())
            .with_context(|| format!("write client lock {}", path.display()))?;
        (&file)
            .flush()
            .with_context(|| format!("flush client lock {}", path.display()))?;

        Ok(Self { file, path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for ClientLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}
