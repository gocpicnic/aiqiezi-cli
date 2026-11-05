/*
  crates/aiqiezi-client/src/lib.rs
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
*/

//! Shared client library for the aiqiezi CLI and GUI.

pub mod backend;
pub mod config;
pub mod firewall;
pub mod key;
pub mod local;

pub use aiqiezi_proto;
pub use config::ClientConfig;
pub use iroh::EndpointId;
