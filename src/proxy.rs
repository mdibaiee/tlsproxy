use std::sync::Arc;
use tokio::net::{TcpStream};
use std::error::Error;
use httparse::Request;
use webpki;
use std::str;
use std::io::{Write, Read};
use rustls::{ServerConfig, ClientConfig, Session};

mod ops;

pub async fn proxy(mut incoming: TcpStream, client_config: ClientConfig, server_config: ServerConfig, args: super::command::Args) -> Result<(), Box<dyn Error>> {
    let mut buf = vec![0; 1024];
    ops::read(&incoming, &mut buf).await.unwrap();

    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut req = Request::new(&mut headers);
    req.parse(&buf).unwrap();

    if args.verbose {
        println!("{}\r\n------", str::from_utf8(&buf)?);
    }

    match (req.method, req.path) {
        (Some("CONNECT"), Some(ref path)) => {
            ops::write(&mut incoming, b"HTTP/1.1 200 OK\r\n\r\n").await?;
            let host = path.split(":").next().expect("Hostname must be valid");
            let dns_name = webpki::DNSNameRef::try_from_ascii_str(host).unwrap();
            println!("{:#?}", dns_name);

            let mut server_session = rustls::ServerSession::new(&Arc::new(server_config));
            let mut incoming_std = incoming.into_std()?;
            let mut incoming_tls = rustls::Stream::new(&mut server_session, &mut incoming_std);

            let mut client_session = rustls::ClientSession::new(&Arc::new(client_config), dns_name);
            let outgoing = TcpStream::connect(path).await?;
            let mut outgoing_std = outgoing.into_std()?;
            let mut outgoing_tls = rustls::Stream::new(&mut client_session, &mut outgoing_std);

            let mut tmp = vec![0; 1024^2];
            let mut tmp2 = vec![0; 1024^2];
            //let ciphersuite = incoming_tls.sess.get_negotiated_ciphersuite().unwrap();
            //println!("{:#?}", ciphersuite);
            loop {
                println!("incoming");
                let incoming_read = ops::sync_read(&mut incoming_tls, &mut tmp);
                println!("incoming {:#?}", str::from_utf8(&tmp)?);
                match incoming_read {
                    Ok(n) if n > 0 => ops::sync_write(&mut outgoing_tls, &tmp),
                    Ok(0) => return Ok(()),
                    e => e
                }.unwrap();

                let outgoing_read = ops::sync_read(&mut outgoing_tls, &mut tmp2);

                println!("outgoing {:#?}", str::from_utf8(&tmp2)?);

                match outgoing_read {
                    Ok(n) if n > 0 => ops::sync_write(&mut incoming_tls, &tmp2),
                    Ok(0) => return Ok(()),

                    e => e
                }.unwrap();
            }
        }

        _ => {
            println!("Not What I Expected!!");
        }
    }

    return Ok(())
}
