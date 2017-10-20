// Copyright 2017 Telefónica Germany Next GmbH. See the COPYRIGHT file at
// the top-level directory of this distribution
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Use the code generation features from `Rocket`. This requires a nightly compiler
#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]
// This feature is necessary for `error` macro aliasing between rocket and log
#![feature(use_extern_macros)]
// These features allow us to choose the allocator
#![allow(unused_features)]
#![feature(global_allocator)]
#![feature(allocator_api)]


// This section is necessary after rustc 1.20.x due to the new way
// allocator selection is handled. Either `jemalloc` may be used, or the system
// allocator (`malloc`) provided by glibc
#[cfg(feature = "system-alloc")]
mod allocator {
    use std::heap::System;

    #[global_allocator]
    pub static mut THE_ALLOC: System = System;
}

#[cfg(not(feature = "system-alloc"))]
mod allocator {
    #[allow(dead_code)]
    pub static THE_ALLOC: () = ();
}

#[allow(unused_imports)]
use allocator::THE_ALLOC;

extern crate hub_sdk;
extern crate mvdb;

#[macro_use(log)]
extern crate log;
extern crate env_logger;

use std::path::PathBuf;
use std::thread;

use hub_sdk::services::rest_ipc::{prep_rocket, ServiceConfig};
use hub_sdk::HubSDK;

fn main() {
    env_logger::init().expect("Failed to initalize logging");

    log::debug!("Starting Hub SDK Service");

    // TODO: Consider using defaults, but it should only pick defaults
    // on a per-field level, not the whole structure
    let config: ServiceConfig = mvdb::helpers::just_load(
        &PathBuf::from("geeny_hub_service.mvdb.json"),
    ).expect("Failed to load config file!");

    let (ipc_cfg, sdk_cfg) = (config.ipc, config.sdk);

    let hub_sdk = HubSDK::new(sdk_cfg);

    let x = prep_rocket(ipc_cfg, hub_sdk.clone());

    let ipc_api = thread::spawn(|| { x.launch(); });

    ipc_api
        .join()
        .expect("Failed to gracefully join the IPC thread");
}
