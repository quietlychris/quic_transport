use futures_util::StreamExt;
use quinn::NewConnection;
use quinn::{Endpoint, ServerConfig};
use std::error::Error;
use std::net::SocketAddr;
use std::{fs::File, io::BufReader};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (certs, key) = read_certs_from_file()?;
    let server_config = ServerConfig::with_single_cert(certs, key)?;
    dbg!(&server_config);

    let (_endpoint, mut incoming) = Endpoint::server(
        server_config,
        "127.0.0.1:25000".parse::<SocketAddr>().unwrap(),
    )?;

    // Start iterating over incoming connections.
    while let Some(conn) = incoming.next().await {
        let mut connection: NewConnection = conn.await?;
        dbg!(&connection);

        while let Some(Ok((mut send, recv))) = connection.bi_streams.next().await {
            // Because it is a bidirectional stream, we can both send and receive.
            let request = recv.read_to_end(100).await?;
            let msg = std::str::from_utf8(&request[..])?;
            println!("request: {:?}", msg);

            send.write_all(b"response").await?;
            send.finish().await?;
        }
        // Save connection somewhere, start transferring, receiving data, see DataTransfer tutorial.
    }

    Ok(())
}

pub fn read_certs_from_file(
) -> Result<(Vec<rustls::Certificate>, rustls::PrivateKey), Box<dyn Error>> {
    let mut cert_chain_reader = BufReader::new(File::open("target/cert.pem")?);
    let certs = rustls_pemfile::certs(&mut cert_chain_reader)?
        .into_iter()
        .map(rustls::Certificate)
        .collect();

    let mut key_reader = BufReader::new(File::open("target/priv_key.pem")?);
    // Since our private key file starts with "BEGIN PRIVATE KEY"
    let mut keys = rustls_pemfile::pkcs8_private_keys(&mut key_reader)?;

    assert_eq!(keys.len(), 1);
    let key = rustls::PrivateKey(keys.remove(0));

    Ok((certs, key))
}
