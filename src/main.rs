use std::net::SocketAddr;
use tokio::net::{TcpListener,TcpStream};
use tokio::io::{copy_bidirectional};
use std::error::Error;
use std::{str, io};
use httparse::Request;

async fn read(stream: &TcpStream, buf: &mut Vec<u8>) -> Result<(), Box<dyn Error>> {
    loop {
        // Wait for the socket to be readable
        stream.readable().await?;

        // Try to read data, this may still fail with `WouldBlock`
        // if the readiness event is a false positive.
        match stream.try_read(buf) {
            Ok(n) => {
                buf.truncate(n);
                break;
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
    
    Ok(())
}

async fn write(stream: &TcpStream, buf: &[u8]) -> Result<(), Box<dyn Error>> {
    loop {
        // Wait for the socket to be writable
        stream.writable().await?;

        // Try to write data, this may still fail with `WouldBlock`
        // if the readiness event is a false positive.
        match stream.try_write(buf) {
            Ok(_) => {
                break;
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                println!("error writing to stream: {:#?}", e);
                return Err(e.into());
            }
        }
    }
    
    Ok(())
}

async fn proxy(mut incoming: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut buf = vec![0; 1024];
    read(&incoming, &mut buf).await.unwrap();

    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut req = Request::new(&mut headers);
    req.parse(&buf).unwrap();

    println!("{}\r\n------", str::from_utf8(&buf)?);

    match (req.method, req.path) {
        (Some("CONNECT"), Some(ref path)) => {
            println!("CONNECT {:#?}", path);
            let mut outgoing = TcpStream::connect(path).await?;

            write(&incoming, b"HTTP/1.1 200 OK\r\n\r\n").await?;

            copy_bidirectional(&mut incoming, &mut outgoing).await?;

            /* A naive implementation of a bidirectional copy:
             * I am actually concerned about this implementation since both reads happen at the
             * same time, if in any case two streams are writing at the same time, some of the data
             * of one of those writes might get discarded.
             *
             *  let mut tmp = vec![0; 1024];
             *  let mut tmp2 = vec![0; 1024];
             *  loop {
             *      let r = select! {
             *          _ = read(&mut incoming, &mut tmp).fuse() =>
             *              write(&mut outgoing, &tmp).fuse(),
             *          _ = read(&mut outgoing, &mut tmp2).fuse() =>
             *              write(&mut incoming, &tmp2).fuse()
             *      };

             *      r.await?
             *  }
             */
        }

        _ => {
            println!("Not What I Expected!!");
        }
    }

    return Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // We'll bind to 127.0.0.1:3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Listening on {:#?}", addr);

    let tcp_listener = TcpListener::bind(addr).await?;
    loop {
        let (tcp_stream, _) = tcp_listener.accept().await?;
        tokio::task::spawn(async move {
            match proxy(tcp_stream).await {
                Ok(()) => {}
                Err(e) => {
                    println!("error: {:#?}", e);
                }
            }
        });
    }
}
