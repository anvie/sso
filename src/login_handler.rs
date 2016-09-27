use url;

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
use std::error::Error;



pub fn setup(server: &mut Nickel){

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

        match conn.simple_search(&format!("uid={},ou=People,dc={},dc=org", user_name, org),
            codes::scopes::LDAP_SCOPE_BASE){
            Ok(result) => {
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
            },
            Err(err) => {
                match err.description().as_ref() {
                    "No such object" => {
                        format!("Credential for `{}` didn't exists.", user_name)
                    },
                    another_error =>
                        format!("Error: {}", another_error)
                }

            }
        }

    });
}
