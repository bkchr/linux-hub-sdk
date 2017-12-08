// Copyright 2017 Telefónica Germany Next GmbH. See the COPYRIGHT file at
// the top-level directory of this distribution
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Endpoints exposed by the REST IPC interface

pub mod things;
pub mod auth;

use interface;
use errors::*;
use super::rest_config::RestConfig;

use std::net::IpAddr;

use hyper::server::{Http, Request, Response, Service};
use hyper::header::{ContentLength, ContentType};
use hyper::{self, Body, Delete, Get, Post, StatusCode};

use serde_json::Value;

use futures::stream::Stream;
use futures::future::FutureResult;
use futures::IntoFuture;

use log;

struct Api {
    sdk: interface::HubSDK,
}

impl Api {
    fn handle_get_request(&self, req: Request) -> Result<Value> {
        let path = req.path();

        if path == "/token/check" {
            auth::token_check(&self.sdk)
        } else if path.starts_with("/messages/") {
            things::get_message(path[10..].to_string(), &self.sdk)
        } else {
            bail!("unknown get request")
        }
    }

    fn handle_post_request(&self, req: Request) -> Result<Value> {
        let path = req.path().to_owned();

        if path == "/login" {
            let body = self.body_to_string(req.body())?;
            auth::login(body, &self.sdk)
        } else if path == "/logout" {
            auth::logout(&self.sdk)
        } else if path == "/things" {
            let body = self.body_to_string(req.body())?;
            things::post_thing(body, &self.sdk)
        } else if path.starts_with("/messages/") {
            let body = self.body_to_string(req.body())?;
            things::post_message(path[10..].to_string(), body, &self.sdk)
        } else {
            bail!("unknown post request")
        }
    }

    fn handle_delete_request(&self, req: Request) -> Result<Value> {
        let path = req.path();

        let unpair = "/things/unpair/";
        let things = "/things/";
        if path.starts_with(unpair) {
            things::unpair_thing(path[unpair.len()..].to_string(), &self.sdk)
        } else if path.starts_with(things) {
            things::delete_thing(path[things.len()..].to_string(), &self.sdk)
        } else {
            bail!("unknown delete request")
        }
    }

    fn body_to_string(&self, body: Body) -> Result<String> {
        // hacky, wait is not nice here
        String::from_utf8(body.wait().fold(vec![], |mut v, c| {
            v.extend(c.unwrap().to_vec());
            v
        })).chain_err(|| "error reading body")
    }
}

impl Service for Api {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = FutureResult<Response, hyper::Error>;

    fn call(&self, req: Request) -> Self::Future {
        let resp = match req.method() {
            &Get => self.handle_get_request(req),
            &Post => self.handle_post_request(req),
            &Delete => self.handle_delete_request(req),
            _ => Err("unknown method".into()),
        };

        match resp {
            Ok(v) => {
                let res = v.to_string();
                Ok(
                    Response::new()
                        .with_header(ContentLength(res.len() as u64))
                        .with_header(ContentType::json())
                        .with_body(res),
                )
            }
            Err(e) => {
                log::error!("{:?}", e);

                let err = format!("{:?}", e);
                Ok(
                    Response::new()
                        .with_header(ContentLength(err.len() as u64))
                        .with_header(ContentType::json())
                        .with_status(StatusCode::BadRequest)
                        .with_body(err),
                )
            }
        }.into_future()
    }
}

pub fn launch_rest(config: RestConfig, sdk: interface::HubSDK) {
    let addr: IpAddr = config.address.parse().expect("Invalid address");
    let port = config.port;

    log::debug!("Starting Hyper; config: {:?}", config);

    let server = Http::new()
        .bind(&(addr, port).into(), move || Ok(Api { sdk: sdk.clone() }))
        .unwrap();
    server.run().expect("error running hyper");
}
