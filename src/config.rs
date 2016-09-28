
use std::fs::File;
use std::io::Read;

use toml;
use toml::Value;

const DEFAULT_DB_STORE:&'static str = "/tmp/sso-store";

// inline simple read parsed toml object macro
macro_rules! simple_toml_read {
    ($toml:ident, $a:tt) => {
        match $toml.get("data_store"){
            Some(&Value::String(ref s)) => s.to_string(),
            _ => DEFAULT_DB_STORE.to_string()
        }
    }
}

pub struct Conf {
    pub data_store:String
}

impl Default for Conf {
    fn default() -> Conf {
        Conf {
            data_store: String::new()
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

                Conf {
                    data_store: data_store
                }
            },
            None => Default::default()
        }
    }
}
