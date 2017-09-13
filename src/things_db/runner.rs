// Copyright 2017 Telefónica Germany Next GmbH. See the COPYRIGHT file at
// the top-level directory of this distribution
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::path::PathBuf;
use std::thread;

use geeny_api::ThingsApi;
use mvdb::Mvdb;
use auth_manager::ServiceCredentials;

use std::fs;
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;

use interface;
use things_db::core::ThingDb;

pub struct RunnerConfig {
    pub certificate_storage: PathBuf,
    pub mqtt_host: String,
    pub mqtt_port: u16,
    pub api: ThingsApi,
}

pub struct CarePackage<'a> {
    // This changes every time
    pub token_opt: Option<String>,

    pub config: &'a RunnerConfig,
}

pub struct ThingDbRunner {
    db: Mvdb<ThingDb>,
    config: RunnerConfig,
    auth: Mvdb<ServiceCredentials>,
}

impl ThingDbRunner {
    pub fn new(config: interface::HubSDKConfig, auth: Mvdb<ServiceCredentials>) -> Self {
        let run_cfg = RunnerConfig {
            certificate_storage: config.mqtt_cert_path,
            mqtt_host: config.mqtt_host,
            mqtt_port: config.mqtt_port,
            api: config.api,
        };


        // Create or load DB file, and ensure permissions are set correctly
        let db_file = Mvdb::from_file_or_default(&config.element_file)
            .expect("Failed to create element storage!");
        fs::set_permissions(&config.element_file, Permissions::from_mode(0o600))
            .expect("Failed to set credentials file permissions");

        ThingDbRunner {
            db: db_file,
            config: run_cfg,
            auth: auth,
        }
    }

    /// Get a thread safe handle to the inner data store
    pub fn thing_db_handle(&self) -> Mvdb<ThingDb> {
        self.db.clone()
    }

    /// Event loop
    pub fn run(&mut self) {
        // TODO, init steps? Load from a file?
        loop {
            self.step();
            #[allow(deprecated)] thread::sleep_ms(250); // TODO IR - Use the scheduler thing
        }
        // TODO, closeout steps?
    }

    /// Single step of the event loop
    fn step(&mut self) {
        let token_opt = self.auth
            .access(|auth| auth.token.clone())
            .expect("Unable to access Auth!");

        let x = CarePackage {
            // This changes every time
            token_opt: token_opt,

            // This is constant across lifetime of self
            config: &self.config,
        };

        self.db
            .access_mut(move |tdb| { tdb.manage(x); })
            .expect("Failed to access ThingDb!");
    }
}
