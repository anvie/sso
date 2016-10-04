#![allow(dead_code)]

#[macro_use] extern crate nickel;
extern crate openldap as oldap;
extern crate crypto;
extern crate rustc_serialize as serialize;
#[macro_use] extern crate url;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate rocksdb;
extern crate toml;
extern crate rand;
extern crate time;
extern crate regex;


use serialize::base64::{self, ToBase64};
use serialize::hex::FromHex;
use std::collections::HashMap;
use nickel::{Nickel, HttpRouter, QueryString, StaticFilesHandler};
use url::percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};

use std::{str, env};
use std::sync::{Arc, Mutex};
use crypto::bcrypt;
use std::io::Read;

mod config;
mod ldap;
mod store;
mod token;
#[macro_use] mod api_result;
mod utils;
mod build;
mod errno;

// handlers
mod login_handler;
mod api_handler;

pub struct Context {
    conf:config::Conf,
    store:Arc<Mutex<store::Store>>
}

fn main() {

    env_logger::init().unwrap();


    let args:Vec<String> = env::args().collect();

    debug!("args: {:?}", args);

    if args.len() < 2 {
        println!("No configuration file specified");
        println!("Usage: {} [CONFIG-FILE]", args[0]);
        std::process::exit(22); // EINVAL (Linux system error code for invalid argument)
    }

    println!("\nSSO service v{}\n", build::VERSION);

    let conf = config::Conf::read_file(&args[1]);
    let store = store::Store::new(&conf.data_store);

    let ctx = Context {
        conf: conf,
        store: Arc::new(Mutex::new(store))
    };

    debug!("data_store: {:?}", ctx.conf.data_store);
    debug!("ldap.uri: {}", ctx.conf.ldap.uri);
    debug!("ldap.admin_user: {}", ctx.conf.ldap.admin_user);
    debug!("ldap.admin_password: {}", ctx.conf.ldap.admin_password);

    println!("Starting...");

    let mut server:Nickel = Nickel::new();

    server.utilize(StaticFilesHandler::new("static/"));

    let conf = ctx.conf.clone();

    server.get("/", middleware! { |_req, _resp|
        let mut data = HashMap::new();
        let query = _req.query();
        let cont:String = utils::encode_url(query.get("continue").unwrap_or("/"));
        debug!("cont: {}", cont);
        data.insert("continue", cont);
        data.insert("login_caption", conf.login_caption.clone());
        data.insert("version", build::VERSION.to_string());
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

    api_handler::setup(&ctx, &mut server);
    login_handler::setup(&ctx, &mut server);

    server.listen("127.0.0.1:8080");
}
