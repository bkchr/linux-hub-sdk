// Copyright 2017 Telefónica Germany Next GmbH. See the COPYRIGHT file at
// the top-level directory of this distribution
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Error chain triggers an annoying lint
#![allow(unused_doc_comment)]

// Use the code generation features from `Rocket`. This requires a nightly compiler
#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]
#![feature(use_extern_macros)]

//! # Geeny Hub SDK
//!
//! ## Introduction
//!
//! The Geeny Hub SDK provides an abstraction over APIs and communication
//! interfaces necessary to connect physical or virtual devices to the Geeny
//! cloud. The Hub SDK can be used in one of two ways: as a Rust library crate, or as a
//! a standalone binary that can be used by applications written in other
//! languages by providing an interprocess communication interface.
//!
//! For developers with an existing Hub Device, such as a Smart Home Gateway,
//! the standalone Geeny Hub Service may be used to provide a single local interface
//! to communicate with Geeny Cloud Services to enable device provisioning, sending
//! messages to the cloud, and receiving messages from the cloud. The Geeny Hub Service
//! may be installed as a package, or included as part of a firmware generation process
//! such as Buildroot or Yocto. When used as a service, no knowledge of Rust developement
//! is necessary.
//!
//! For developers building a new Hub Device, the Geeny Hub SDK may be used as a library
//! (or crate), and can be tightly integrated into a Rust-based application. This allows
//! developers to interact with the Geeny Cloud through an idiomatic Rust library interface,
//! rather than having to implement REST, MQTT, and other communication protocols.
//!
//! ## Components
//!
//! ### Usage - As a Library Crate
//!
//! ```rust,no_run
//! extern crate hub_sdk;
//!
//! use hub_sdk::{HubSDK, HubSDKConfig};
//!
//! fn main() {
//!     let sdk_cfg = HubSDKConfig::default();
//!
//!     // Begin running the SDK. The hub_sdk handle may be used to interact with
//!     // the sdk. This handle may be cloned and given to multiple consumers
//!     let hub_sdk = HubSDK::new(sdk_cfg);
//!
//!     let msgs = hub_sdk.receive_messages("ABC123")
//!         .expect("No known device with that serial number");
//!
//!     println!("Messages: {:?}", msgs);
//! }
//! ```
//!
//! #### Documentation
//!
//! Full library documentation may be found on [docs.rs](https://docs.rs/hub-sdk), or may be generated
//! from this repository using `cargo doc --open`.
//!
//! ### Usage - As a standalone service
//!
//! ```bash
//! # Create a valid config file for this service
//! cp ./geeny_hub_service.mvdb.json.example ./geeny_hub_service.mvdb.json
//!
//! # Run the service, serving a REST IPC on localhost:9000
//! cargo run --release --bin hub-service
//! ```
//!
//! #### Documentation
//!
//! For more information regarding the REST IPC interface, please see
//! [this Swagger API specification](./docs/rest-ipc/swagger.json)
//! for more information.
//!
//! ## Requirements
//!
//! Currently, the Geeny Hub SDK requires a nightly build of Rust.
//!
//! ## Installation & Configuration
//!
//! ### As a library
//!
//! In your `Cargo.toml`, add the following lines:
//!
//! ```toml
//! [dependencies]
//! hub-sdk = "0.3"
//! ```
//!
//! In your main project file (likely `lib.rs` or `main.rs`), add the following line:
//!
//! ```rust,ignore
//! extern crate hub_sdk;
//! ```
//!
//! ### As a service
//!
//! ```bash
//! # Create a valid config file for this service
//! cp ./geeny_hub_service.mvdb.json.example ./geeny_hub_service.mvdb.json
//!
//! # Run the service, serving a REST IPC on localhost:9000
//! cargo run --release --bin hub-service
//! ```
//!
//! ## Testing
//!
//! Unit tests may be run with `cargo test`.
//!
//! ## License
//!
//! Copyright (C) 2017 Telefónica Germany Next GmbH, Charlottenstrasse 4, 10969 Berlin.
//!
//! This project is licensed under the terms of the [Mozilla Public License Version 2.0](LICENSE.md).
//!
//! Contact: devsupport@geeny.io

#[macro_use(log)]
extern crate log;

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;
extern crate rocket;
extern crate rumqtt;
extern crate uuid;

// TODO DI-245
//   * Move to sqlite3 or diesel
//   * OR if still with mvdb, how do we handle poisoned mutexes (Antidote)
extern crate mvdb;

// Re-export
pub extern crate geeny_api;

mod interface;

pub use self::interface::{HubSDK, HubSDKConfig};
pub mod errors;

// Used by bin crates, or by external services that consume the
// bin crates
pub mod services;

mod auth_manager;
mod things_db;
