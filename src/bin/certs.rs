use std::error::Error;
use std::fs;

fn main() -> Result<(), Box<dyn Error>> {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let cert_pem = cert.serialize_pem().unwrap();
    fs::write("target/cert.pem", cert_pem).expect("Error writing certificate to file");

    let priv_key_pem = cert.serialize_private_key_pem();
    fs::write("target/priv_key.pem", priv_key_pem).expect("Error writing private key to file");

    Ok(())
}
