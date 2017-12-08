// Copyright 2017 Telefónica Germany Next GmbH. See the COPYRIGHT file at
// the top-level directory of this distribution
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use errors::*;
use geeny_api::models::AuthLoginRequest;

use interface::HubSDK;

use serde_json::{from_str, to_value, Value};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Hash)]
pub struct TokenCheckResponse {
    email: String,
    valid_token: bool,
}

pub fn token_check(sdk: &HubSDK) -> Result<Value> {
    let (email, valid) = sdk.check_token()?;

    to_value(
        (TokenCheckResponse {
            email: email,
            valid_token: valid,
        }),
    ).chain_err(|| "error converting TokenCheckResponse to json")
}

pub fn login(message: String, sdk: &HubSDK) -> Result<Value> {
    let message: AuthLoginRequest = from_str(&message)?;
    sdk.login(&message.email, &message.password)?;

    Ok(json!({"status": "success"}))
}

pub fn logout(sdk: &HubSDK) -> Result<Value> {
    sdk.logout()?;

    Ok(json!({"status": "success"}))
}
