use std::{
    io::Read,
    net::{TcpListener, TcpStream},
};

use bloom_logger;

fn handle_client(mut stream: TcpStream) {
    bloom_logger::log(bloom_logger::BLOOM_STREAM, "Incoming connection!");

    let mut buf = [0u8; 4096];

    if let Ok(n) = stream.read(&mut buf) {
        bloom_logger::log(
            bloom_logger::BLOOM_STREAM,
            format!("Recv {n} bytes from client: {:?}", &buf[..n]),
        );
    }
}

pub fn start_server() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:31415")?;

    for stream in listener.incoming() {
        handle_client(stream?);
    }

    Ok(())
}
