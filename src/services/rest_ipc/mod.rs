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

use rocket;
use log;

pub mod api;
pub mod rocket_config;

use interface::{self, HubSDKConfig};
use self::rocket_config::RocketConfig;

pub fn prep_rocket(config: RocketConfig, sdk: interface::HubSDK) -> rocket::Rocket {
    let rocket_cfg = config.render();

    log::debug!("Starting Rocket; config: {:?}", rocket_cfg);

    rocket::custom(rocket_cfg, false)
        .mount(
            "/api/v1/",
            routes![
                // Things API
                api::things::post_thing,
                api::things::post_message,
                api::things::get_message,
                api::things::unpair_thing,
                api::things::delete_thing,

                // Auth API
                api::auth::login,
                api::auth::logout,
                api::auth::token_check,
            ],
        )
        .manage(sdk)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceConfig {
    pub sdk: HubSDKConfig,

    pub ipc: RocketConfig,
}
