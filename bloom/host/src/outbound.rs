use std::{io::Write, net::TcpStream};

use bloom_api::{
    BloomError, BloomEvent, ClientEvent, ClientRx, EventRx, Packet,
};
use crossbeam_channel::select;

// Outbound cannot reliable detect client disconnection
// It blocks the thread until event_rx is written to.
// Hence only when an event is generated, we will
// know if the client has disconnected

pub fn handle_outbound_terminal(
    client_rx: ClientRx,
    event_rx: EventRx,
) -> Result<(), BloomError> {
    println!("[I AM OUTBOUND]");
    loop {
        let ClientEvent::Connected(client) = client_rx
            .recv()
            .map_err(|_| BloomError::ChannelRecieveError)?
        else {
            // return Err(BloomError::ClientDisconnected);
            println!("client disconnected!");
            continue;
        };

        // this will obtain the currently connected client
        let Some(mut stream) = client.stream else {
            // return Err(BloomError::FailedToObtainClientStream);
            println!("client stream doesn't exist!");
            continue;
        };

        println!("[OUTBOUND TERMINAL] Client connected! Waiting on events");

        loop {
            select! {
                recv(client_rx) -> msg => {
                    if let Ok(ClientEvent::Disconnected) = msg {
                       break;
                    }
                }

                recv(event_rx) -> msg => {
                    let event = msg.map_err(|_| BloomError::ChannelRecieveError)?;
                    println!("[OUTBOUND TERMINAL] Recieved an event! sending to client... {}", event.event);
                    write_event(&mut stream, event)?;
                }
            }
        }
    }
}

fn write_event(
    stream: &mut TcpStream,
    event: BloomEvent,
) -> Result<(), BloomError> {
    let bytes = event.to_bytes()?;
    let len = bytes.len() as u32;

    println!("[OUTBOUND TERMINAL] Sending {} bytes", len);

    stream
        .write_all(&len.to_be_bytes())
        .map_err(|_| BloomError::TcpWriteFailed)?;

    stream
        .write_all(&bytes)
        .map_err(|_| BloomError::TcpWriteFailed)?;

    Ok(())
}
