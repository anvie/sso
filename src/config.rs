
use std::fs::File;
use std::io::Read;

use toml;
use toml::Value;

const DEFAULT_DB_STORE:&'static str = "/tmp/sso-store";

// inline simple read parsed toml object macro
macro_rules! simple_toml_read {
    ($toml:ident, $a:expr, $dflt:expr) => {
        match $toml.get($a){
            Some(&Value::String(ref s)) => s.to_string(),
            _ => $dflt
        }
    };
    ($toml:ident, $tbl:expr, $a:tt, $dflt:expr) => {
        match $toml.get($tbl){
            Some(&Value::Table(ref _tbl)) => simple_toml_read!(_tbl, $a, $dflt),
            _ => $dflt
        }
    }
}

#[derive(Clone)]
pub struct LdapConf {
    pub uri:String,
    pub admin_user:String,
    pub admin_password:String
}

impl Default for LdapConf {
    fn default() -> LdapConf {
        LdapConf {
            ..Default::default()
        }
    }
}

#[derive(Clone)]
pub struct Conf {
    pub data_store:String,
    pub allowed_continue_domain:String,
    pub ldap: LdapConf,
    pub login_caption:String
}

impl Default for Conf {
    fn default() -> Conf {
        Conf {
            data_store: String::new(),
            allowed_continue_domain: String::new(),
            ..Default::default()
        }
    }
}

impl Conf {

    pub fn read_file(path:&str) -> Conf {
        let mut input = String::new();
        File::open(path).and_then(|mut f| {
            f.read_to_string(&mut input)
        }).unwrap();
        Conf::read_str(&input)
    }

    pub fn read_str(data:&str) -> Conf {
        let mut parser = toml::Parser::new(data);
        match parser.parse() {
            Some(toml) => {
                debug!("toml: {:?}", toml);

                let data_store = simple_toml_read!(toml, "data_store", DEFAULT_DB_STORE.to_string());
                let allowed_continue_domain = simple_toml_read!(toml, "allowed_continue_domain", "".to_string());
                let ldap_conf = LdapConf {
                    uri : simple_toml_read!(toml, "ldap", "uri", "".to_string()),
                    admin_user : simple_toml_read!(toml, "ldap", "admin_user", "".to_string()),
                    admin_password : simple_toml_read!(toml, "ldap", "admin_password", "".to_string())
                };

                Conf {
                    data_store: data_store,
                    allowed_continue_domain: allowed_continue_domain,
                    ldap: ldap_conf,
                    login_caption: simple_toml_read!(toml, "login_caption", "".to_string())
                }
            },
            None => Default::default()
        }
    }
}
