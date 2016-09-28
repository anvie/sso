#[macro_use] extern crate nickel;
extern crate openldap as oldap;
extern crate crypto;
extern crate rustc_serialize as serialize;
extern crate url;
#[macro_use] extern crate log;
extern crate env_logger;

use serialize::base64::{self, ToBase64};
use serialize::hex::FromHex;
use std::collections::HashMap;
use nickel::{Nickel, HttpRouter, QueryString, StaticFilesHandler};
// use ldap::*;
// use ldap::errors::*;
use std::str;
use std::sync::{Arc, Mutex};
use crypto::bcrypt;
use std::io::Read;

mod ldap;
mod login_handler;

fn main() {

    env_logger::init().unwrap();

    let mut server:Nickel = Nickel::new();

    server.utilize(StaticFilesHandler::new("static/"));

    server.get("/", middleware! { |_req, _resp|
        let mut data = HashMap::new();
        let query = _req.query();
        let cont = query.get("continue").unwrap_or("/");
        data.insert("continue", cont);
        return _resp.render("tmpl/index.html", &data);
    });

    server.get("/genPass", middleware! { |_req, _resp|
        let query = _req.query();
        let pass = query.get("pass").unwrap();

        let mut out = [0u8; 24];

        bcrypt::bcrypt(5, b"1234567891234567", pass.as_bytes(), &mut out);

        let s = out.to_base64(base64::STANDARD);

        format!("crypted pass: {}", s)
    });

    login_handler::setup(&mut server);

    server.listen("127.0.0.1:8080");
}
