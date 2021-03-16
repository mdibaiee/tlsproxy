use tokio::net::{TcpStream};
use std::error::Error;
use std::{io};
use std::io::{Read, Write};
use rustls::{Session};

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

pub fn sync_read<T: Read>(stream: &mut T, buf: &mut Vec<u8>) -> Result<usize, Box<dyn Error>> {
    loop {
        match stream.read(buf) {
            Ok(n) => {
                buf.truncate(n);
                return Ok(n);
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

pub fn sync_write<T: Write>(stream: &mut T, buf: &[u8]) -> Result<usize, Box<dyn Error>> {
    loop {
        match stream.write(buf) {
            Ok(n) => {
                return Ok(n);
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
}

pub fn tls_read<T: Session, R: Read>(stream: &mut T, buf: &mut R) -> Result<usize, Box<dyn Error>> {
    loop {
        match stream.read_tls(buf) {
            Ok(n) => {
                return Ok(n);
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

pub fn tls_write<T: Session, W: Write>(stream: &mut T, buf: &mut W) -> Result<usize, Box<dyn Error>> {
    loop {
        match stream.write_tls(buf) {
            Ok(n) => {
                return Ok(n);
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
}
