use quinn::{ClientConfig, Endpoint};
use std::fs::File;
use std::io::BufReader;
use std::{error::Error, net::SocketAddr};

use rustls::Certificate;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut certs = rustls::RootCertStore::empty();
    let mut cert_chain_reader = BufReader::new(File::open("target/cert.pem")?);
    let server_certs: Vec<Certificate> = rustls_pemfile::certs(&mut cert_chain_reader)?
        .into_iter()
        .map(rustls::Certificate)
        .collect();
    for cert in server_certs {
        certs.add(&cert)?;
    }

    let client_cfg = ClientConfig::with_root_certificates(certs);

    let client_addr = "0.0.0.0:0".parse::<SocketAddr>()?;
    let mut endpoint = Endpoint::client(client_addr)?;
    endpoint.set_default_client_config(client_cfg);

    let server_addr = "127.0.0.1:25000".parse::<SocketAddr>()?;
    // Connect to the server passing in the server name which is supposed to be in the server certificate.
    let connection = endpoint.connect(server_addr, "localhost")?.await?;

    let mut buf = vec![0; 1_000];

    for i in 0..100 {
        let (mut send, mut recv) = connection.open_bi().await?;

        let msg = format!("test #{}", i);
        send.write_all(msg.as_bytes()).await?;
        send.finish().await?;

        match recv.read(&mut buf).await.unwrap() {
            Some(n) => {
                let received = std::str::from_utf8(&buf[..n])?;
                dbg!(&received);
            }
            None => continue,
        };

        use tokio::time::{sleep, Duration};
        sleep(Duration::from_millis(500)).await;
    }

    Ok(())
}

/// Constructs a QUIC endpoint configured for use a client only.
/// - server_certs: list of trusted certificates.
#[allow(unused)]
pub fn make_client_endpoint(
    bind_addr: SocketAddr,
    server_certs: &[&[u8]],
) -> Result<Endpoint, Box<dyn Error>> {
    let client_cfg = configure_client(server_certs)?;
    let mut endpoint = Endpoint::client(bind_addr)?;
    endpoint.set_default_client_config(client_cfg);
    Ok(endpoint)
}

/// Builds default quinn client config and trusts given certificates.
/// - server_certs: a list of trusted certificates in DER format.
fn configure_client(server_certs: &[&[u8]]) -> Result<ClientConfig, Box<dyn Error>> {
    let mut certs = rustls::RootCertStore::empty();
    for cert in server_certs {
        certs.add(&rustls::Certificate(cert.to_vec()))?;
    }

    Ok(ClientConfig::with_root_certificates(certs))
}
