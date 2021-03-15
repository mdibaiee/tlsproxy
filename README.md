tls-forward-proxy
=================

A most basic forward proxy using [Tokio](https://tokio.rs/) [TcpStreams](https://docs.rs/tokio/1.3.0/tokio/net/struct.TcpStream.html).

Sample usage:
```
$ cargo build && cargo run
Listening on 127.0.0.1:8080
```

Send requests to your destination through this proxy:
```
curl --cacert /path/to/cert.pem https://test.dev:5000 -x http://proxy.dev:8080 --proxytunnel
```
