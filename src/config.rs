
use std::fs::File;
use std::io::Read;

use toml;
use toml::Value;

const DEFAULT_DB_STORE:&'static str = "/tmp/sso-store";

// inline simple read parsed toml object macro
macro_rules! simple_toml_read {
    ($toml:ident, $a:tt) => {
        match $toml.get($a){
            Some(&Value::String(ref s)) => s.to_string(),
            _ => DEFAULT_DB_STORE.to_string()
        }
    }
}

pub struct Conf {
    pub data_store:String,
    pub allowed_continue_domain:String
}

impl Default for Conf {
    fn default() -> Conf {
        Conf {
            data_store: String::new(),
            allowed_continue_domain: String::new()
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

                let data_store = simple_toml_read!(toml, "data_store");
                let allowed_continue_domain = simple_toml_read!(toml, "allowed_continue_domain");

                Conf {
                    data_store: data_store,
                    allowed_continue_domain: allowed_continue_domain
                }
            },
            None => Default::default()
        }
    }
}
