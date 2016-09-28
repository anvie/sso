use url;

use serialize::base64::{self, ToBase64, FromBase64};
use serialize::hex::FromHex;
use std::collections::HashMap;
use nickel::{Nickel, HttpRouter, QueryString, StaticFilesHandler};
use std::str;
use std::sync::{Arc, Mutex};
use crypto::bcrypt;
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use std::io::Read;
use std::error::Error;
use oldap::codes;
use oldap::errors::*;

// module
use ldap;
use store::Store;
use config::Conf;
use token;
use Context;


fn check_password(challenge_password: &String, password: &String) -> bool {
    let algo = &challenge_password[..6];
    if algo != "{SSHA}" {
        error!("Unexpected pass hash algo: {}", algo);
        return false;
    }
    let data = &challenge_password[6..];

    let data_bytes = data.from_base64().unwrap();
    let digest = &data_bytes[..20];
    let salt = &data_bytes[20..];

    debug!("data: {:?}", data_bytes);
    debug!("digest: {:?}", digest);
    debug!("salt: {:?}", salt);

    let mut sha = Sha1::new();
    sha.input_str(password);
    sha.input(salt);

    let mut calculated_sha_output = [0u8; 20];
    sha.result(&mut calculated_sha_output);

    let matched = digest[..] == calculated_sha_output[..];

    debug!("sha: {:?}", calculated_sha_output);
    debug!("matched: {}", matched);

    matched
}


pub fn setup(ctx:&Context, server: &mut Nickel){

    let store = ctx.store.clone();

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

        let store = store.lock().unwrap();

        // // before
        // debug!("before (user_name): {:?}", store.get("user_name"));
        //
        // store.put("user_name", &user_name);
        // // after
        // debug!("after (user_name): {:?}", store.get("user_name"));

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

                let password_is_ok = check_password(&user_password, &given_password);

                println!("userPassword: {:?}, check_password(): {}", user_password, password_is_ok);

                if (!password_is_ok){
                    return _resp.send("Access Denied");
                }

                // for debugging purposes only.
                // for res in result {
                //     println!("simple search result: {:?}", res);
                //     for (key, value) in res {
                //         println!("- key: {:?}", key);
                //         result_str.push_str("    ");
                //         result_str.push_str(&key);
                //         result_str.push_str(" :  ");
                //         for res_val in value {
                //             println!("    + {:?}", res_val);
                //             result_str.push_str(&res_val);
                //             result_str.push_str("\n");
                //         }
                //     }
                // }

                // @TODO(robin): generate real token here
                let generated_token = token::generate();
                match store.get(&user_name){
                    Some(old_token) => {
                        store.del(&old_token)
                    },
                    _ => ()
                }
                store.put(&generated_token, &user_name);
                store.put(&user_name, &generated_token);

                _resp.headers_mut().set_raw("Content-type", vec![b"text/plain".to_vec()]);

                //format!("continue to: {}\n  {:?}", cont, result)
                // format!("Oke, continue to: {}, result:\n{}", cont, result_str)
                format!("Access Granted. Token: {}. Continue to: {}", generated_token, cont)
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
