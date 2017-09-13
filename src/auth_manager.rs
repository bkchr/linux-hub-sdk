// Copyright 2017 Telefónica Germany Next GmbH. See the COPYRIGHT file at
// the top-level directory of this distribution
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use mvdb::Mvdb;
use geeny_api::ConnectApi;
use geeny_api::models::AuthLoginResponse;
use log;
use std::thread;
use interface;

#[derive(Debug, Serialize, Deserialize, Clone, Hash, Default)]
pub struct ServiceCredentials {
    pub username: String,
    pub token: Option<String>,
}

// TODO DI-246:
//   * This needs some love. We should probably immediately retry a few times if the token is invalid
//   * Inverse Exponential Backoff (try more often the closer we are to expiration)
//   * Provide a way to instantly invalidate the token, e.g., if requests fail to 40x errors (bad auth)
//   * Purge unwraps
//   * cleanup dead paths here, remove password from storage structure
//   * granularity between bad request, bad token, etc (also DI-234, enumerated error types)
pub fn auth_manager(config: interface::HubSDKConfig, auth: &Mvdb<ServiceCredentials>) {
    let server = config.connect_api;
    loop {
        let creds = auth.access(|auth| auth.clone()).unwrap();

        let new_tkn = match creds.token {
            None => None,
            Some(ref tkn) => check_and_refresh(&server, tkn),
        };

        let backoff_ms: u32 = match new_tkn {
            Some(_) => (24 * 60 * 60 * 1000), // One day
            None => (5 * 60 * 1000), // 5 minutes
        };

        if new_tkn != creds.token {
            auth.access_mut(|auth| auth.token = new_tkn).unwrap();
        }

        log::info!("auth manager sleeping for {} seconds...", backoff_ms);
        #[allow(deprecated)] thread::sleep_ms(backoff_ms);
    }
}

pub fn check_and_refresh(server: &ConnectApi, auth: &str) -> Option<String> {
    log::info!("Refreshing current token");
    match server.refresh_token(&AuthLoginResponse { token: auth.into() }) {
        Ok(tkn) => Some(tkn.token),
        Err(e) => {
            log::error!("Token refresh failed: {:?}", e);
            None
        }
    }
}
