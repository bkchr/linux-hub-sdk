// Copyright 2017 Telefónica Germany Next GmbH. See the COPYRIGHT file at
// the top-level directory of this distribution
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Endpoints exposed by the REST IPC interface
use rocket;
use log;

pub mod things;
pub mod auth;

use super::rest_config::RestConfig;
use interface;

pub fn launch_rest(config: RestConfig, sdk: interface::HubSDK) {
    let rocket_cfg = rocket::Config::build(rocket::config::Environment::Development) // TODO
        .address(config.address.clone())
        .port(config.port)
        .workers(config.workers)
        .expect("Invalid IPC Configuration!");

    log::debug!("Starting Rocket; config: {:?}", rocket_cfg);

    rocket::custom(rocket_cfg, false)
        .mount(
            "/api/v1/",
            routes![
                // Things API
                things::post_thing,
                things::post_message,
                things::get_message,
                things::unpair_thing,
                things::delete_thing,
                // Auth API
                auth::login,
                auth::logout,
                auth::token_check,
            ],
        )
        .manage(sdk)
        .launch();
}
