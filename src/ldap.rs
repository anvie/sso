
use oldap::*;
// use oldap::errors::*;


pub fn connect(uri:&str, admin:&str, password:&str, dn:&str) -> RustLDAP {

    debug!("conneting to ldap using: uri: {}, admin: {}, password: {}, dn: {}",
        uri, admin, password, dn);

    let conn = RustLDAP::new(uri).unwrap();
    conn.set_option(codes::options::LDAP_OPT_PROTOCOL_VERSION, &codes::versions::LDAP_VERSION3);

    match conn.simple_bind(&format!("cn={},{}", admin, dn), password){
        Ok(_) => conn, // ini gak salah, bener
        Err(e) => {
            error!("{}", e);
            panic!("Cannot binding ldap connection")
        }
    }
}
