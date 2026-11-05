/*
  crates/aiqiezi-cli/src/shutdown.rs
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
*/

use tokio::sync::watch;

#[derive(Clone)]
pub struct Shutdown {
    sender: watch::Sender<bool>,
}

impl Shutdown {
    pub fn new() -> Self {
        let (sender, _receiver) = watch::channel(false);
        Self { sender }
    }

    pub fn trigger(&self) {
        self.sender.send_replace(true);
    }

    pub async fn wait(&self) {
        let mut receiver = self.sender.subscribe();
        loop {
            if *receiver.borrow() {
                return;
            }
            if receiver.changed().await.is_err() {
                return;
            }
        }
    }
}
