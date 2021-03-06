extern crate rustc_serialize;

// use url;

use nickel::{Nickel, HttpRouter, QueryString};
use std::str;
// use std::sync::{Arc, Mutex};
// use std::io::Read;
// use std::error::Error;
use nickel::mimes::MediaType;
// use nickel::status::*;
use serialize::json;
// use time;
use Context;

// module

use api_result;
use utils;
use build;
use errno;

pub fn setup(ctx:&Context, server: &mut Nickel){

    let store = ctx.store.clone();

    server.get("/api/system/info", middleware! { |_req, mut _resp|

        api_result_success_json!(api_result::SystemInfo {
            server_time: utils::current_time_millis(),
            git_rev: build::GIT_REV.to_string(),
            version: build::VERSION.to_string()
        }, _resp)

    });

    // for access token lookup
    server.get("/api/lookup", middleware! { |_req, mut _resp|
        let query = _req.query();
        let access_token = query.get("access_token").unwrap_or("");

        let store = store.lock().unwrap();

        debug!("checking access token: {}", access_token);

        match (store.get(&access_token), store.get(&format!("dn_{}", &access_token))){
            (Some(uid), Some(dn)) => {
                let info = format!("Authentic for `{}`", uid);
                debug!("{}", info);

                api_result_success_json!(api_result::Cred::new(uid, dn), _resp)
            },
            _ => {
                warn!("Invalid access token or already expired: {}", access_token);
                api_result_error_json!(errno::INVALID_TOKEN, errno::INVALID_TOKEN_STR, _resp)
            }
        }

    });
}
