use std::{io::Write, net::TcpStream};

use bloom_api::{
    ClientChannel, ClientEvent, EventChannel, Packet, state::global_state,
};
use crossbeam_channel::select;

pub fn handle_outbound_terminal(
    stream: &mut TcpStream,
    client_channel: &ClientChannel,
    event_channel: &EventChannel,
) {
    loop {
        select! {
            recv(client_channel.rx()) -> msg => {
                if let Ok(ClientEvent::Disconnected) = msg {
                    break;
                }
            }

            recv(event_channel.rx()) -> msg => {
                match msg {
                    Ok(event) => {
                        let bytes = event.to_bytes().unwrap();
                        let len = bytes.len() as u32;

                        // even though it is possible that the writes error out due to client
                        // disconnection, we let the inbound handle the disconnection. oubound will
                        // simply ignore the error, if any occor.
                        // Multiple sources of reporting disconnection might lead to race conditions
                        let _ = stream.write_all(&len.to_be_bytes());
                        let _ = stream.write_all(&bytes);
                    }

                    Err(_) => {
                        global_state().terminate_app();
                    }
                }
            }
        }
    }
}
