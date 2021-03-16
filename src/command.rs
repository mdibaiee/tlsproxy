use rustls_pemfile;
use std::io::BufReader;
use std::{fs, str};

use clap::{App, Arg};

#[derive(Debug, Clone)]
pub struct Args {
    pub port: u16,
    pub certs: String,
    pub key: String,
    pub replaces: Vec<String>,
    pub verbose: bool,
}

pub fn args() -> Args {
    let version = env!("CARGO_PKG_NAME").to_string() + ", version: " + env!("CARGO_PKG_VERSION");

    let matches = App::new("tlsproxy")
                        .version(&*version)
                        .author("Mahdi Dibaiee <mdibaiee@pm.me>")
                        .about("A simple TLS forward proxy capable of replacing parts of the outgoing traffic")
                        .arg(Arg::with_name("port")
                                .short("p")
                                .long("port")
                                .value_name("PORT")
                                .help("Listen on PORT [default: 8080]")
                                .takes_value(true))
                        .arg(Arg::with_name("certs")
                                .long("certs")
                                .value_name("FILE")
                                .help("Read server certificates from FILE. This should contain PEM-format certificates in the right order. The first certificate should certify KEYFILE, the last should be a root CA.")
                                .required(true)
                                .takes_value(true))
                        .arg(Arg::with_name("key")
                                .long("key")
                                .value_name("FILE")
                                .help("Read private key from FILE. This should be a RSA private key or PKCS8-encoded private key, in PEM format.")
                                .required(true)
                                .takes_value(true))
                        .arg(Arg::with_name("replace")
                                .long("replace")
                                .value_name("PATTERN")
                                .help("Replace data in outgoing requests according to patterns specified in the s/MATCH/REPLACEMENT format.")
                                .multiple(true)
                                .use_delimiter(true)
                                .takes_value(true))
                        .arg(Arg::with_name("verbose")
                                .short("v")
                                .long("verbose")
                                .help("Be noisy"))
                        .get_matches();

    return Args {
        port: matches
            .value_of("port")
            .unwrap_or("8080")
            .parse::<u16>()
            .unwrap(),
        certs: matches
            .value_of("certs")
            .map(|a| a.to_owned())
            .expect("--certs must be specified"),
        key: matches
            .value_of("key")
            .map(|a| a.to_owned())
            .expect("--key must be specified"),
        verbose: matches.is_present("verbose"),
        replaces: matches
            .values_of("replace")
            .map_or(vec![], |a| a.map(|b| b.to_owned()).collect()),
    };
}

pub fn load_certs(filename: &str) -> Vec<rustls::Certificate> {
    let certfile = fs::File::open(filename).expect("cannot open certificate file");
    let mut reader = BufReader::new(certfile);
    rustls_pemfile::certs(&mut reader)
        .unwrap()
        .iter()
        .map(|v| rustls::Certificate(v.clone()))
        .collect()
}

pub fn read_file(filename: &str) -> BufReader<fs::File> {
    let certfile = fs::File::open(filename).expect("cannot open certificate file");
    BufReader::new(certfile)
}

pub fn load_private_key(filename: &str) -> rustls::PrivateKey {
    let keyfile = fs::File::open(filename).expect("cannot open private key file");
    let mut reader = BufReader::new(keyfile);

    loop {
        match rustls_pemfile::read_one(&mut reader).expect("cannot parse private key .pem file") {
            Some(rustls_pemfile::Item::RSAKey(key)) => return rustls::PrivateKey(key),
            Some(rustls_pemfile::Item::PKCS8Key(key)) => return rustls::PrivateKey(key),
            None => break,
            _ => {}
        }
    }

    panic!(
        "no keys found in {:?} (encrypted keys not supported)",
        filename
    );
}
