#[macro_use] extern crate nickel;
extern crate openldap as ldap;
extern crate crypto;
extern crate rustc_serialize as serialize;
extern crate url;

use serialize::base64::{self, ToBase64};
use serialize::hex::FromHex;
use std::collections::HashMap;
use nickel::{Nickel, HttpRouter, QueryString, StaticFilesHandler};
use ldap::*;
use ldap::errors::*;
use std::str;
use std::sync::{Arc, Mutex};
use crypto::bcrypt;
use std::io::Read;


fn main() {
    let mut server = Nickel::new();

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

    let conn = RustLDAP::new("ldap://127.0.0.1").unwrap();
    conn.set_option(codes::options::LDAP_OPT_PROTOCOL_VERSION, &codes::versions::LDAP_VERSION3);
    conn.simple_bind("cn=admin,dc=ansvia,dc=org", "123123").unwrap();
    let conn = Arc::new(Mutex::new(conn));

    server.post("/login", middleware! { |_req, mut _resp|

        let mut user_name = String::new();
        let mut password = String::new();

        {
            let mut body = String::new();

            _req.origin.read_to_string(&mut body).unwrap();

            println!("{:?}", body);

            for (key, value) in url::form_urlencoded::parse(body.as_bytes()){
                if (key == "user_name") { user_name = value.clone().into_owned(); }
                if (key == "password") { password = value.clone().into_owned(); }
            }

        }

        let query = _req.query();
        let cont = query.get("continue").unwrap_or("/");


        println!("{:?}", user_name);

        // let userName = query.get("user_name").unwrap();
        // let password = query.get("password").unwrap();

        let conn = conn.clone();
        let conn = conn.lock().unwrap();

        let org = query.get("org").unwrap_or("ansvia").to_string();

        let result = conn.simple_search(&format!("dc={},dc=org", org),
            codes::scopes::LDAP_SCOPE_BASE).unwrap();

        let mut result_str = "".to_owned();

        for res in result {
            println!("simple search result: {:?}", res);
            for (key, value) in res {
                println!("- key: {:?}", key);
                result_str.push_str("    ");
                result_str.push_str(&key);
                result_str.push_str(" :  ");
                for res_val in value {
                    println!("    + {:?}", res_val);
                    result_str.push_str(&res_val);
                    result_str.push_str("\n");
                }
            }
        }

        _resp.headers_mut().set_raw("Content-type", vec![b"text/plain".to_vec()]);

        //format!("continue to: {}\n  {:?}", cont, result)
        format!("Oke, continue to: {}, result:\n{}", cont, result_str)
    });

    server.listen("127.0.0.1:8080");
}
