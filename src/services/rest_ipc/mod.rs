// Copyright 2017 Telefónica Germany Next GmbH. See the COPYRIGHT file at
// the top-level directory of this distribution
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Provide a REST API for interfacing with the Hub SDK
//!
//! This module uses (Rocket)[https://rocket.rs] to present a REST API.
//! This module is meant to be used as a standalone binary, usable when
//! the consumer of the Hub SDK is not a rust application

#[cfg(feature = "rest-rocket-service")]
pub mod api_rocket;
#[cfg(feature = "rest-hyper-service")]
pub mod api_hyper;
pub mod rest_config;

use interface::HubSDKConfig;
use self::rest_config::RestConfig;
#[cfg(feature = "rest-rocket-service")]
use self::api_rocket as api;
#[cfg(feature = "rest-hyper-service")]
use self::api_hyper as api;
pub use self::api::launch_rest;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceConfig {
    pub sdk: HubSDKConfig,

    pub ipc: RestConfig,
}
