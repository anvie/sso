
use oldap::*;
use oldap::errors::*;


pub fn connect(uri:&str, admin:&str, password:&str) -> RustLDAP {
    let conn = RustLDAP::new(uri).unwrap();
    conn.set_option(codes::options::LDAP_OPT_PROTOCOL_VERSION, &codes::versions::LDAP_VERSION3);
    conn.simple_bind(&format!("cn={},dc=ansvia,dc=org", admin), password).unwrap();
    conn
}
