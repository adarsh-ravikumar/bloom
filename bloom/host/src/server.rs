use std::{
    io::Read,
    net::{TcpListener, TcpStream},
};

use bloom_logger;

fn handle_client(mut stream: TcpStream) {
    bloom_logger::log(bloom_logger::HOST_STREAM, "Incoming connection!");

    let mut buf = [0u8; 4096];

    if let Ok(n) = stream.read(&mut buf) {
        bloom_logger::log(
            bloom_logger::HOST_STREAM,
            format!("Recv {n} bytes from client: {:?}", &buf[..n]),
        );

        // Connection Request Packet -> 44 bytes
        // : Client Version -> [major, minor, patch] 3 bytes
        // : ID ->             8 bytes (8 random characters)
        // : App name       -> 32 bytes [UTF-8 encoded string, NUL-padded]
        // : NUL-Padding    -> 1 byte
        let major_version = buf[0];
        let minor_version = buf[1];
        let patch_version = buf[2];
        let id = str::from_utf8(&buf[3..11]).unwrap();
        let app_name = str::from_utf8(&buf[11..43]).unwrap();
        if buf[44] != '\0' as u8 {
            println!("Invalid packet terminator!");
        }

        println!(
            "client_ver: v{major_version}.{minor_version}.{patch_version} | client_id: {id} | app_name: {app_name}",
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
