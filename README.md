## quic-tranport

This is a basic example using the [`quinn`](https://github.com/quinn-rs/quinn) library and associated deps (i.e. `rustls`) to create a simple bi-directional QUIC connection using self-signed certificates

Usage is as follows:
```sh
$ cargo run --bin certs
# Open up one terminal
$ cargo run --bin server
# Open a second terminal
$ cargo run --bin client
```

By default, the `.pem` certificates and keys are written into the `target/` directory. 