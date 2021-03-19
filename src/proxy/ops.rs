use tokio::net::{TcpStream};
use std::error::Error;
use std::{io};

pub async fn read(stream: &TcpStream, buf: &mut Vec<u8>) -> Result<(), Box<dyn Error>> {
    loop {
        stream.readable().await?;

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

pub async fn write(stream: &TcpStream, buf: &[u8]) -> Result<(), Box<dyn Error>> {
    loop {
        stream.writable().await?;

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
