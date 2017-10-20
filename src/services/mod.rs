// Copyright 2017 Telefónica Germany Next GmbH. See the COPYRIGHT file at
// the top-level directory of this distribution
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Hub SDK Services - Service Applications using the Geeny Hub SDK

#[cfg(feature = "rest-service")]
pub mod rest_ipc;

pub use things_db::PartialThingMessage;
