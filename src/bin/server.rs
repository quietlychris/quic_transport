use quinn::{Endpoint, RecvStream, SendStream, ServerConfig};
use std::error::Error;
use std::net::SocketAddr;
use std::{fs::File, io::BufReader};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (certs, key) = read_certs_from_file()?;
    let server_config = ServerConfig::with_single_cert(certs, key)?;
    dbg!(&server_config);

    let endpoint = Endpoint::server(
        server_config,
        "127.0.0.1:25000".parse::<SocketAddr>().unwrap(),
    )?;

    let mut connections = Vec::new();

    while let Some(conn) = endpoint.accept().await {
        /* let quinn::NewConnection {
            connection,
            mut bi_streams,
            ..
        } = conn.await?;
        */
        let connection = conn.await?;
        // let bi_streams = connection.open

        let handle = tokio::spawn(async move {
            dbg!(connection.remote_address());
            let mut buf = vec![0; 1000];
            while let Ok(stream) = connection.accept_bi().await {
                process_quic(&connection, stream, &mut buf).await;
            }
        });
        connections.push(handle);
    }

    Ok(())
}

async fn process_quic(
    connection: &quinn::Connection,
    stream: (SendStream, RecvStream),
    buf: &mut Vec<u8>,
) {
    let (mut tx, mut rx) = stream;

    if let Some(n) = rx.read(buf).await.unwrap() {
        let msg = std::str::from_utf8(&buf[..n]).unwrap();
        println!("msg: {:?}", msg);
        let reply = format!("got: {} from {}", msg, connection.remote_address());
        if let Err(e) = tx.write_all(reply.as_bytes()).await {
            println!("Error: {}", e);
        };
    }
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
