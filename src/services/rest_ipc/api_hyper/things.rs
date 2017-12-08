// Copyright 2017 Telefónica Germany Next GmbH. See the COPYRIGHT file at
// the top-level directory of this distribution
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use errors::*;
use things_db::PartialThingMessage;

use interface::HubSDK;

use serde_json::{from_str, to_value, Value};

// Convenience type
type IpcApiResult = Result<Value>;

#[derive(Debug, Serialize, Deserialize)]
pub struct IncomingMessages {
    pub msgs: Vec<PartialThingMessage>,
}

pub fn post_thing(payload: String, sdk: &HubSDK) -> IpcApiResult {
    let thing = from_str(&payload)?;
    sdk.create_thing(thing)?;

    Ok(json!({"status": "success"}))
}

pub fn delete_thing(serial: String, sdk: &HubSDK) -> IpcApiResult {
    sdk.delete_thing_by_serial(&serial)?;

    Ok(json!({"status": "success"}))
}

pub fn unpair_thing(serial: String, sdk: &HubSDK) -> IpcApiResult {
    sdk.unpair_thing_by_serial(&serial)?;

    Ok(json!({"status": "success"}))
}

pub fn post_message(serial: String, payload: String, sdk: &HubSDK) -> IpcApiResult {
    let payload: IncomingMessages = from_str(&payload)?;
    sdk.send_messages(&serial, &payload.msgs)?;

    Ok(json!({"status": "success"}))
}

pub fn get_message(serial: String, sdk: &HubSDK) -> IpcApiResult {
    let msgs = sdk.receive_messages(&serial)?;

    to_value(IncomingMessages { msgs: msgs })
        .chain_err(|| "error converting IncomingMessages to json")
}
