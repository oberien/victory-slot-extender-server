#[macro_use] extern crate log;

extern crate env_logger;
extern crate futures;
extern crate tokio_core;
extern crate sha2;

use std::fs::File;

use std::io::{Error, ErrorKind};
use futures::Future;
use futures::stream::Stream;
use tokio_core::io::{read, write_all};
use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;
use sha2::digest::Digest;
use sha2::Sha256;

fn main() {
    env_logger::init().unwrap();
    let addr = "0.0.0.0:13371".to_string().parse().unwrap();

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let socket = TcpListener::bind(&addr, &handle).unwrap();

    // Once we've got the TCP listener, inform that we have it
    info!("Listening on: {}", addr);

    let done = socket.incoming().for_each(move |(socket, addr)| {
        info!("{}: New Connection", addr);
        let mut buf = Vec::new();
        // the file should not be larger than 1MB
        buf.resize(1024*1024, 0);
        let amt = read(socket, buf)
            .and_then(|(_, mut buf, n)| {
                if n >= 1024*1024 {
                    return futures::failed(Error::new(ErrorKind::ConnectionAborted, "client sent too much")).boxed();
                }
                buf.resize(n, 0);
                let mut sha = Sha256::new();
                sha.input(&buf[..]);
                let hex = sha.result_str();
                let file = File::create(format!("out/{}", hex)).unwrap();
                write_all(file, buf).boxed()
            }).map(move |_| info!("{}: wrote file", addr))
            .map_err(move |e| info!("{}: error: {:?}", addr, e));
        handle.spawn(amt);
        Ok(())
    });
    core.run(done).unwrap();
}
