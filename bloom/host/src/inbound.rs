use std::{io::Read, net::TcpStream};

use bloom_api::{
    BloomCommand, ClientChannel, ClientEvent, CommandChannel, Packet,
};

pub fn handle_inbound_origin(
    stream: &mut TcpStream,
    client_channel: &ClientChannel,
    command_channel: &CommandChannel,
) {
    loop {
        let mut length_buf = [0u8; 4];

        if stream.read_exact(&mut length_buf).is_err() {
            client_channel.send(ClientEvent::Disconnected);
            break;
        }

        // from be, as TCP uses big endian
        let payload_len = u32::from_be_bytes(length_buf) as usize;

        println!("Command length: {payload_len}");

        let mut packet_buf = vec![0u8; payload_len];
        let _ = stream.read_exact(&mut packet_buf);

        println!("Recieved packet");

        // deserialize into command
        // TODO: Send an "invalid command" packet back to the client
        let command = BloomCommand::from_bytes(packet_buf).unwrap();
        println!("Command name: {}", command.command);

        command_channel.send(command);
    }
}
