// Copyright 2017 Telefónica Germany Next GmbH. See the COPYRIGHT file at
// the top-level directory of this distribution
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use rocket::State;
use rocket_contrib::Json;

use errors as echain;
use geeny_api::models::AuthLoginRequest;

use interface::HubSDK;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Hash)]
pub struct TokenCheckResponse {
    email: String,
    valid_token: bool,
}

#[get("/token/check", format = "application/json")]
pub fn token_check(sdk: State<HubSDK>) -> Result<Json<TokenCheckResponse>, echain::Error> {

    let (email, valid) = sdk.check_token()?;


    Ok(Json(TokenCheckResponse {
        email: email,
        valid_token: valid,
    }))
}

#[post("/login", format = "application/json", data = "<message>")]
pub fn login(message: Json<AuthLoginRequest>, sdk: State<HubSDK>) -> Result<String, echain::Error> {
    sdk.login(&message.email, &message.password)?;
    Ok("success".into())
}

#[post("/logout")]
pub fn logout(sdk: State<HubSDK>) -> Result<String, echain::Error> {
    sdk.logout()?;
    Ok("success".into())
}
