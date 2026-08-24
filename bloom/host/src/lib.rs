use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::mpsc::{Sender, channel},
};

use bloom_api::{
    BloomCommand, BloomConnection, BloomError, BloomEvent, InboundChannel, InboundTX,
    OutboundChannel, OutboundRX, OutboundTX, Packet,
};

// the init host will create it's own channel and return it
fn handle_connection(stream: &mut TcpStream) -> Result<BloomCommand, BloomError> {
    let mut length_buf = [0u8; 44];

    stream
        .read_exact(&mut length_buf)
        .map_err(|_| BloomError::TcpReadFailed)?;

    let connection = BloomConnection::from_bytes(length_buf.into())?;

    connection.into_command()
}

fn handle_inbound(mut stream: TcpStream) -> Result<(), BloomError> {
    let mut length_buf = [0u8; 4];

    loop {
        stream
            .read_exact(&mut length_buf)
            .map_err(|_| BloomError::TcpReadFailed)?;

        // from be, as TCP uses big endian
        let payload_len = u32::from_be_bytes(length_buf) as usize;
        if payload_len > 10_244_102 {
            // 10 MB
            return Err(BloomError::PacketTooLarge);
        }

        let mut packet_buf = vec![0u8; payload_len + 8];

        let _ = stream.read_exact(&mut packet_buf);

        // Inbound packet can only be a command
        // let command = BloomCommand::from_bytes(packet_buf);
    }
}

fn handle_outbound(rx: OutboundRX, mut stream: TcpStream) -> Result<(), BloomError> {
    loop {
        // let data = rx.recv().map_err(|_| BloomError::ChannelRecieveError)?;
        let data = rx.recv().unwrap();
        println!("outbound recieved!{:?}", data);
        let bytes = data.to_bytes()?;
        let len = bytes.len() as u32;

        println!("{len} {:?}", len.to_be_bytes());
        stream.write_all(&len.to_be_bytes());
        stream.write_all(&bytes);
    }
}

fn listen(tx: InboundTX, client_tx: InboundTX, outbound_rx: OutboundRX) -> Result<(), BloomError> {
    let listener = TcpListener::bind("127.0.0.1:31415").map_err(|_| BloomError::TcpBindFailed)?;

    let (disconnect_tx, disconnect_rx) = channel();

    // block the current thread and obtain exactly one client
    let (mut stream, _) = listener.accept().map_err(|_| BloomError::TcpAcceptFailed)?;

    let client_info = handle_connection(&mut stream)?;
    println!("Client connected. Writing to stream.");
    client_tx.send(client_info);

    let inbound_stream = stream
        .try_clone()
        .map_err(|_| BloomError::TcpStreamCloneFailed)?;

    let mut outbound_stream = stream
        .try_clone()
        .map_err(|_| BloomError::TcpStreamCloneFailed)?;

    // spawn the streams
    let disconnect_tx = disconnect_tx.clone();
    std::thread::spawn(move || {
        // this will block the thread as long as the client is connected
        let _ = handle_inbound(inbound_stream);

        // client disconnected
        let _ = disconnect_tx.send(true);
    });

    std::thread::spawn(move || {
        handle_outbound(outbound_rx, outbound_stream);
    });

    // block until client disconnects
    let _ = disconnect_rx.recv();

    Ok(())
}

pub fn init(tx: InboundTX, rx: OutboundRX) -> Result<(), BloomError> {
    // start the server
    let client_info_channel = InboundChannel::new();

    let tauri_tx = tx.clone();
    std::thread::spawn(move || {
        let _ = listen(tauri_tx, client_info_channel.tx.clone(), rx);
    });

    // loop {
    //     let event = rx.recv().unwrap(); // TODO: err handling
    //     //
    //     if event.event == "connected" {
    //         println!("Host recieved frontend connection. Relaying client information");
    //         let client: BloomCommand = client_info_channel.rx.recv().unwrap();
    //         println!("data: {:?}", client.data);
    //         tx.send(client).unwrap();
    //     }
    // }
    //
    Ok(())
}
