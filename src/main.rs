use std::sync::Arc;
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
    let certs = command::load_certs(&args.certs);
    let privkey = command::load_private_key(&args.key);
    server_config
        .set_single_cert(certs.clone(), privkey.clone())
        .expect("bad certificates/private key");

    let mut client_config = ClientConfig::new();
    client_config.key_log = Arc::new(rustls::KeyLogFile::new());

    client_config
        .dangerous()
        .set_certificate_verifier(Arc::new(cert::NoCertificateVerification {}));

    client_config
        .root_store
        .add_pem_file(&mut command::read_file(&args.certs)).unwrap();
            
    //client_config
        //.root_store
        //.add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);

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
