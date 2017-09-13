// Copyright 2017 Telefónica Germany Next GmbH. See the COPYRIGHT file at
// the top-level directory of this distribution
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Configuration structure for [Rocket](https://rocket.rs)
//!
//! Necessary because Rocket's current configuration structures are not
//! serializable/deserializable, which is inconvenient for making a single
//! configuration file for this service

use rocket;

#[derive(Debug, Serialize, Deserialize, Hash, PartialEq, Clone)]
pub struct RocketConfig {
    /// Address to serve the IPC interface on. It is recommended to use `localhost`
    /// to only expose this service on the local machine.
    pub address: String,

    /// Port to serve the IPC interface on. Defaults to 8000
    pub port: u16,

    /// Number of multithreaded workers to serve the IPC interface
    pub workers: u16,
}

impl RocketConfig {
    pub fn render(&self) -> rocket::Config {
        rocket::Config::build(rocket::config::Environment::Development) // TODO
            .address(self.address.clone())
            .port(self.port)
            .workers(self.workers)
            .expect("Invalid IPC Configuration!")
    }
}

impl Default for RocketConfig {
    fn default() -> Self {
        Self {
            address: "127.0.0.1".into(),
            port: 8000,
            workers: 8,
        }
    }
}
