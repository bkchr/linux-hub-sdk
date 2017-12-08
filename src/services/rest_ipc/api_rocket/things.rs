// Copyright 2017 Telefónica Germany Next GmbH. See the COPYRIGHT file at
// the top-level directory of this distribution
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use rocket::State;
use rocket_contrib::{Json, Value};

use geeny_api::models::ThingRequest;

use errors as echain;
use things_db::PartialThingMessage;

use interface::HubSDK;

// Convenience type
type IpcApiResult<T> = Result<Json<T>, echain::Error>;

#[derive(Debug, Serialize, Deserialize)]
pub struct IncomingMessages {
    pub msgs: Vec<PartialThingMessage>,
}

#[post("/things", format = "application/json", data = "<payload>")]
pub fn post_thing(payload: Json<ThingRequest>, sdk: State<HubSDK>) -> IpcApiResult<Value> {
    sdk.create_thing(payload.into_inner())?;

    Ok(Json(json!({"status": "success"})))
}

#[delete("/things/<serial>", format = "application/json")]
pub fn delete_thing(serial: String, sdk: State<HubSDK>) -> IpcApiResult<Value> {
    sdk.delete_thing_by_serial(&serial)?;

    Ok(Json(json!({
        "status": "success",
    })))
}

#[delete("/things/unpair/<serial>", format = "application/json")]
pub fn unpair_thing(serial: String, sdk: State<HubSDK>) -> IpcApiResult<Value> {
    sdk.unpair_thing_by_serial(&serial)?;

    Ok(Json(json!({
        "status": "success",
    })))
}


#[post("/messages/<serial>", format = "application/json", data = "<payload>")]
pub fn post_message(
    serial: String,
    payload: Json<IncomingMessages>,
    sdk: State<HubSDK>,
) -> IpcApiResult<Value> {
    sdk.send_messages(&serial, &payload.msgs)?;

    Ok(Json(json!({
        "status": "success",
    })))
}

#[get("/messages/<serial>", format = "application/json")]
pub fn get_message(serial: String, sdk: State<HubSDK>) -> IpcApiResult<IncomingMessages> {
    let msgs = sdk.receive_messages(&serial)?;

    Ok(Json(IncomingMessages { msgs: msgs }))
}
