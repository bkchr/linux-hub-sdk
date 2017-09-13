// Copyright 2017 Telefónica Germany Next GmbH. See the COPYRIGHT file at
// the top-level directory of this distribution
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// The things database is broken into the following parts
// with the following responsibilities:

// The `core` module contains the `ThingDb` struct. It is a container for
// `HubThing`s , which can be indexed by either Serial Number (primary key),
// or Geeny Thing UUID (secondary key)
mod core;

// The `hub_thing` module contains the `HubThing` struct. This container holds
// the current state of the device (e.g., 'needs to gather metadata', 'active device'),
// as well as the channels used to send data hub -> cloud and cloud -> hub
mod hub_thing;

// The `runner` module is a manually pumped event loop. It periodically prompts
// the `ThingDb` to trigger each of its `HubThing`s to perform any necessary actions,
// such as processing queued messages, or to update state if new information is available
mod runner;

/// The `state` module contains the inner state types used by `HubThing`s.
/// State transition logic occurs here
mod state;

pub use self::runner::ThingDbRunner;
pub use self::core::ThingDb;

/// Structure to contain a message to be sent to or received from the Geeny Cloud
///
/// Does not contain the Serial Number or `ThingId` of the device in question.
///
/// Please see `HubSDK::send_messages` and `HubSDK::receive_messages` for further
/// examples of usage
///
/// ```rust,no_run
/// use hub_sdk::{HubSDK, HubSDKConfig};
/// use hub_sdk::services::PartialThingMessage;
///
/// let sdk_cfg = HubSDKConfig::default();
/// let hub_sdk = HubSDK::new(sdk_cfg);
///
/// let messages = vec!(
///     PartialThingMessage {
///         topic: "demo/send/path".into(),
///         msg: "demonstration message".into(),
///     },
///     PartialThingMessage {
///         topic: "demo/other/path".into(),
///         msg: "second demonstration message".into(),
///     },
/// );
///
/// hub_sdk.send_messages("ABC123456", &messages)
///     .expect("Failed to send messages!");
/// ```
#[derive(Debug, Serialize, Deserialize, Clone, Hash)]
pub struct PartialThingMessage {
    pub topic: String,
    pub msg: String,
}
