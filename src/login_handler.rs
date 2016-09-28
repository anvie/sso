use url;

use serialize::base64::{self, ToBase64};
use serialize::hex::FromHex;
use std::collections::HashMap;
use nickel::{Nickel, HttpRouter, QueryString, StaticFilesHandler};
use std::str;
use std::sync::{Arc, Mutex};
use crypto::bcrypt;
use std::io::Read;
use std::error::Error;
use oldap::codes;
use oldap::errors::*;

// module
use ldap;

fn check_password(challenge_password: &String, password: &String) -> bool {
    let algo = &challenge_password[..6];
    if (algo != "{SSHA}"){
        error!("Unexpected pass hash algo: {}", algo);
        return false;
    }
    let data = &challenge_password[6..];
    println!("data: {}", data);
    true
}


pub fn setup(server: &mut Nickel){


    server.post("/login", middleware! { |_req, mut _resp|

        let mut user_name = String::new();
        let mut given_password = String::new();

        {
            let mut body = String::new();

            _req.origin.read_to_string(&mut body).unwrap();

            for (key, value) in url::form_urlencoded::parse(body.as_bytes()){
                if (key == "user_name") { user_name = value.clone().into_owned(); }
                if (key == "password") { given_password = value.clone().into_owned(); }
            }

        }

        let query = _req.query();
        let cont = query.get("continue").unwrap_or("/");


        debug!("user_name: {:?}", user_name);

        // let userName = query.get("user_name").unwrap();
        // let password = query.get("password").unwrap();

        // let conn = conn.clone();
        // let conn = conn.lock().unwrap();

        let conn = ldap::connect("ldap://127.0.0.1", "admin", "123123");

        let org = query.get("org").unwrap_or("ansvia").to_string();

        let dn_query = &format!("uid={},ou=People,dc={},dc=org", user_name, org);

        debug!("dn_query: {}", dn_query);

        match conn.simple_search(dn_query,
            codes::scopes::LDAP_SCOPE_BASE){
            Ok(result) => {

                let user_password = {
                    let owned = result.to_owned();
                    match owned.first(){
                        Some(ref o) => {
                            let v = o.get("userPassword").unwrap_or(&Vec::<String>::new()).to_owned();
                            match v.first() {
                                Some(v) => v.clone(),
                                None => "".to_string()
                            }
                        },
                        _ => "".to_string()
                    }
                };

                let mut result_str = "".to_owned();

                println!("userPassword: {:?}, check_password(): {}", user_password,
                    check_password(&user_password, &given_password));

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
