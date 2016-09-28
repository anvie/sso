use url;

// use serialize::base64::{self, ToBase64, FromBase64};
// use serialize::hex::FromHex;
// use std::collections::HashMap;
use nickel::{Nickel, HttpRouter, QueryString, StaticFilesHandler};
use std::str;
// use std::sync::{Arc, Mutex};
// use std::io::Read;
// use std::error::Error;

use Context;

// module


pub fn setup(ctx:&Context, server: &mut Nickel){

    let store = ctx.store.clone();

    server.get("/lookup", middleware! { |_req, mut _resp|
        let query = _req.query();
        let access_token = query.get("access_token").unwrap_or("");

        let store = store.lock().unwrap();

        debug!("checking access token: {}", access_token);

        match store.get(&access_token){
            Some(token) => {
                let res = format!("Authentic for `{}`", token);
                debug!("{}", res);
                res
            },
            _ => {
                warn!("Invalid access token or already expired: {}", access_token);
                format!("None")
            }
        }

    });
}
