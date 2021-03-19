use std::sync::{Arc};
use std::net::SocketAddr;
use tokio::net::{TcpListener};
use rustls::{ServerConfig, ClientConfig, NoClientAuth};
use webpki_roots;

mod command;
mod proxy;
mod cert;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = command::args();

    // We'll bind to 127.0.0.1:3000
    let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
    println!("Listening on {:#?}", addr);

    let mut server_config = ServerConfig::new(NoClientAuth::new());
    let chaincert = command::load_certs(&args.chaincert);
    let privkey = command::load_private_key(&args.key);
    server_config
        .set_single_cert(chaincert, privkey)
        .expect("bad certificates/private key");

    let mut client_config = ClientConfig::new();

    let (added, unused) = client_config
        .root_store
        .add_pem_file(&mut command::read_file(&args.cacert)).unwrap();

    println!("{} certificates added, {} unused", added, unused);
    let (added, unused) = client_config
        .root_store
        .add_pem_file(&mut command::read_file(&args.chaincert)).unwrap();
    println!("{} certificates added, {} unused", added, unused);
            
    let tcp_listener = TcpListener::bind(addr).await?;
    loop {
        let (tcp_stream, _) = tcp_listener.accept().await?;
        let cc = client_config.clone();
        let sc = server_config.clone();
        let args_copy = args.clone();
        tokio::task::spawn(async move {
            match proxy::proxy(tcp_stream, cc, sc, args_copy).await {
                Ok(()) => {}
                Err(e) => {
                    println!("error: {:#?}", e);
                }
            }
        });
    }
}
