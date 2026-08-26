use std::{io::Read, net::TcpStream};

use bloom_api::{
    BloomCommand, BloomError, ClientEvent, ClientRx, ClientTx, CommandTx,
    Packet,
};
use crossbeam_channel::select;
const MAX_PACKET_SIZE: usize = 10_244_102; // 10 MB

pub fn handle_inbound_origin(
    client_tx: ClientTx,
    client_rx: ClientRx,
    command_tx: CommandTx,
) -> Result<(), BloomError> {
    loop {
        select! {
            recv(client_rx) -> msg =>
                match msg {
                    Ok(ClientEvent::Connected(client)) => {
                        let connect_command = client.into_command()?;
                        command_tx
                            .send(connect_command)
                            .map_err(|_| BloomError::ChannelSendError)?;

                        let Some(mut stream) = client.stream else {
                            return Err(BloomError::FailedToObtainClientStream);
                        };

                        loop {
                            println!("Waiting for command");

                            let command = read_command(&mut stream).map_err(|_| {
                                // if error, we assume that client has disconnected (because the stream has closed)
                                let _ = client_tx.send(ClientEvent::Disconnected);
                                BloomError::ClientDisconnected
                            })?;

                            println!("Recieved command! {}", command.command);

                            command_tx
                                .send(command)
                                .map_err(|_| BloomError::ChannelSendError)?;
                        }
                    }

                    Ok(_) => continue,

                    Err(_) => Err(BloomError::ChannelRecieveError)?

                }
        }
    }
}

fn read_command(stream: &mut TcpStream) -> Result<BloomCommand, BloomError> {
    let mut length_buf = [0u8; 4];

    stream
        .read_exact(&mut length_buf)
        .map_err(|_| BloomError::TcpReadFailed)?;

    // from be, as TCP uses big endian
    let payload_len = u32::from_be_bytes(length_buf) as usize;

    println!("Command length: {payload_len}");

    let mut packet_buf = vec![0u8; payload_len];

    let _ = stream.read_exact(&mut packet_buf);

    println!("Recieved payload packet:\n{:?}", packet_buf);

    // deserialize into command
    match BloomCommand::from_bytes(packet_buf) {
        Ok(command) => {
            println!("Command name: {}", command.command);
            return Ok(command);
        }

        Err(e) => {
            println!("Something went wrong: {:?}", e);
        }
    }

    Err(BloomError::ClientDisconnected)
}
