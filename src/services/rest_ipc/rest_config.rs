// Copyright 2017 Telefónica Germany Next GmbH. See the COPYRIGHT file at
// the top-level directory of this distribution
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Configuration structure for the rest api

#[derive(Debug, Serialize, Deserialize, Hash, PartialEq, Clone)]
pub struct RestConfig {
    /// Address to serve the IPC interface on. It is recommended to use `localhost`
    /// to only expose this service on the local machine.
    pub address: String,

    /// Port to serve the IPC interface on. Defaults to 8000
    pub port: u16,

    /// Number of multithreaded workers to serve the IPC interface
    pub workers: u16,
}

impl Default for RestConfig {
    fn default() -> Self {
        Self {
            address: "127.0.0.1".into(),
            port: 8000,
            workers: 8,
        }
    }
}
