use url;

use serialize::base64::{FromBase64};
// use serialize::hex::FromHex;
use serialize::json;
use nickel::MediaType;
// use std::collections::HashMap;
use nickel::{Nickel, HttpRouter, QueryString, Response};
// use nickel::status::StatusCode;
use nickel::extensions::Redirect;
use std::str;
// use std::sync::{Arc, Mutex};
// use crypto::bcrypt;
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use std::io::Read;
use std::error::Error;
use oldap::codes;
// use oldap::errors::*;
use regex::Regex;
use url::{Url, ParseError};
use mustache::{MapBuilder};
use nickel_mustache::Render;

// module
use ldap;
// use store::Store;
use token;
use Context;
use api_result;
// use errno;
use build;
use utils;


macro_rules! show_error{
    ($error:expr, $cont:expr, $conf:ident, $target_dn:expr, $_resp:ident) => {{
        // let mut data = HashMap::new();
        let cont:String = utils::encode_url($cont);

        let data = MapBuilder::new()
            .insert_str("continue", cont)
            .insert_str("login_caption", $conf.login_caption.clone())
            .insert_str("version", build::VERSION.to_string())
            .insert_bool("error", true)
            .insert_str("error_desc", $error.to_string())
            .insert_str("target_dn", $target_dn.to_string())
            .build();

        // data.insert("continue", cont.to_string());
        // data.insert("login_caption", $conf.login_caption.clone());
        // data.insert("version", build::VERSION.to_string());
        // data.insert("error", true);
        // data.insert("error_desc", $error.to_string());

        // return $_resp.render("tmpl/index.html", &data);


        return Render::render_data($_resp, "tmpl/index.html", &data);
    }}
}


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
    let conf = ctx.conf.clone();

    // for security reason we only accept for specific domain/sub-domain provided in config.
    let re_str = format!(r"^https?://[a-zA-Z0-9\.\\-_]*({}).*$", ctx.conf.allowed_continue_domain.replace(".", "\\."));
    debug!("re_str: {}", re_str);


    // @FIXME
    let url_re = Regex::new("^https?://.+$").unwrap();


    let cont_re = match Regex::new(&re_str){
        Ok(_r) => _r,
        Err(e) => {
            error!("{:?}", e);
            panic!("Invalid `allowed_continue_domain` format, please check your configuration file.")
        }
    };

    server.post("/login", middleware! { |_req, mut _resp|

        let mut user_name = String::new();
        let mut given_password = String::new();

        {
            let mut body = String::new();

            _req.origin.read_to_string(&mut body).unwrap();

            for (key, value) in url::form_urlencoded::parse(body.as_bytes()){
                if key == "user_name" { user_name = value.clone().into_owned(); }
                if key == "password" { given_password = value.clone().into_owned(); }
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
        let cont = query.get("continue").unwrap_or("?");


        debug!("user_name: {:?}", user_name);

        let dn = query.get("dn").unwrap_or("dc=ansvia,dc=org").to_string();
        let conn = ldap::connect(&conf.ldap.uri, &conf.ldap.admin_user,
            &conf.ldap.admin_password, &dn);


        let dn_query = &format!("uid={},ou=People,{}", user_name, dn);

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

                // let mut result_str = "".to_owned();

                let password_is_ok = check_password(&user_password, &given_password);

                // debug!("userPassword: {:?}, check_password(): {}", user_password, password_is_ok);

                if !password_is_ok {
                    // return _resp.send("Access Denied");

                    // let result = api_result_error_json!(errno::UNAUTHORIZED, errno::UNAUTHORIZED_STR, _resp);
                    // return _resp.send(result);


                    show_error!("Identitas atau kata kunci tidak benar, mohon pastikan identitas atau kata kunci yang Anda masukkan benar.",
                            cont, conf, dn, _resp);
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


                let generated_token = token::generate();

                match store.get(&user_name){
                    Some(old_token) => {
                        store.batch()
                            .del(&old_token) // remove old token records
                            .del(&format!("dn_{}", &old_token)) // and it's dn
                            .commit()
                    },
                    _ => ()
                }

                store.batch()
                    .put(&generated_token, &user_name)
                    .put(&user_name, &generated_token) // for reverse loookup
                    .put(&format!("dn_{}", &generated_token), &dn)
                    .commit();

                debug!("continue: {}", cont);

                if cont_re.is_match(cont){

                    let mut url = Url::parse(cont).unwrap();
                    url.query_pairs_mut().append_pair("token", &generated_token);

                    return _resp.redirect(url.into_string());

                }else if url_re.is_match(cont){
                    // api_result_error_json!(errno::UNAUTHORIZED, errno::UNAUTHORIZED_STR, _resp)

                    // return _resp.send(show_error(ctx, "Identitas atau kata kunci salah", cont, &_resp));

                    show_error!("Identitas atau kata kunci tidak benar, mohon pastikan identitas atau kata kunci yang Anda masukkan benar.",
                            cont, conf, dn, _resp)
                }else{
                    api_result_success_json!(generated_token, _resp)
                }


            },
            Err(err) => {
                match err.description().as_ref() {
                    "No such object" => {
                        // format!("Credential for `{}` didn't exists.", user_name)
                        // api_result_error_json!(errno::UNAUTHORIZED, errno::UNAUTHORIZED_STR, _resp)

                        show_error!("Kredensial tidak ditemukan, mohon periksa identitas masuk Anda.",
                                cont, conf, dn, _resp)
                    },
                    another_error => {
                        // api_result_error_json!(errno::INTERNAL_SERVER_ERROR,
                        //     &format!("Cannot binding to LDAP service. {}.", another_error), _resp)

                        error!("Cannot binding to LDAP service. {}.", another_error);

                        show_error!("Internal server error. Gagal terhubung dengan server LDAP.",
                                cont, conf, dn, _resp)

                        // format!("Error: {}", another_error)
                    }
                }

            }
        }

    });
}
