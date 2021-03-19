use std::sync::Arc;
use tokio::net::{TcpStream};
use std::error::Error;
use httparse::Request;
use webpki;
use std::str;
use std::io;
use std::io::{Write, Read};
use rustls::{ServerConfig, ClientConfig};

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

            let mut incoming_buf = vec![0; 4086];
            let mut outgoing_buf = vec![0; 4086];
            loop {
                match incoming_tls.read(&mut incoming_buf) {
                    Ok(n) => {
                        incoming_buf.truncate(n);
                        if args.verbose && n > 0 {
                            println!("received data of size {:#?} from client: {:#?}", n, str::from_utf8(&incoming_buf)?);
                        }

                        args.replaces.iter().for_each(|(from, to)| {
                            let from_buf = from.as_bytes().to_vec();
                            let to_buf = to.as_bytes().to_vec();
                            let from_len = from.len();

                            let mut windowed = incoming_buf.windows(from_len).collect::<Vec<&[u8]>>();
                            let mut indices: Vec<usize> = vec![];

                            while let Some(p) = windowed.iter().position(|x| x.to_owned() == from_buf) {
                                windowed.push(&to_buf);
                                windowed.swap_remove(p);
                                indices.push(p);
                            }

                            indices.iter().for_each(|p| {
                                let range = p..&(p+from_len);
                                incoming_buf.splice(range, to_buf.clone());
                            });
                            if args.verbose {
                                println!("replaced {} instances of {} with {}", indices.len(), from, to);
                            }
                        });
                        outgoing_tls.write_all(&incoming_buf)?;
                    },

                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => {},

                    Err(e) => return Err(e.into())
                }

                match outgoing_tls.read(&mut outgoing_buf) {
                    Ok(n) => {
                        outgoing_buf.truncate(n);
                        if args.verbose && n > 0 {
                            println!("received data of size {:#?} from server: {:#?}", n, str::from_utf8(&outgoing_buf)?);
                        }

                        incoming_tls.write_all(&outgoing_buf)?;
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        continue;
                    }
                    Err(e) => {
                        println!("error reading from stream: {:#?}", e);
                        return Err(e.into());
                    }
                }
            }
        }

        _ => {
            println!("Not What I Expected!!");
        }
    }

    return Ok(())
}
