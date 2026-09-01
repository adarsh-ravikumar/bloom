use std::{io::Read, net::TcpListener};

use bloom_api::{
    BloomConnection, BloomError, ClientChannel, CommandChannel, EventChannel,
    Packet, state::global_state,
};

use crate::{
    inbound::handle_inbound_origin, outbound::handle_outbound_terminal,
};

pub fn handle_client_lifecycle(
    client_channel: ClientChannel,
    command_channel: CommandChannel,
    event_channel: EventChannel,
) -> Result<(), BloomError> {
    let listener = TcpListener::bind("127.0.0.1:31415")
        .map_err(|_| BloomError::TcpBindFailed)?;

    let mut connection_packet = [0u8; 44];

    loop {
        let (mut stream, _) = listener
            .accept()
            .map_err(|_| BloomError::ClientConnectionFailed)?;

        // block thread until client sends connection packet
        stream
            .read_exact(&mut connection_packet)
            .map_err(|_| BloomError::TcpReadFailed)?;

        // TODO: validate packet, send ACK packet

        // client has connected
        let connection = BloomConnection::from_bytes(connection_packet.into())?;

        global_state().set_client(Some(connection.clone()));

        client_channel.send(bloom_api::ClientEvent::Connected(connection));

        // spawn the outbound terminal handler
        let mut stream_handle = stream.try_clone().unwrap();
        let client_channel_handle = client_channel.clone();
        let event_channel_handle = event_channel.clone();

        std::thread::spawn(move || {
            handle_outbound_terminal(
                &mut stream_handle,
                &client_channel_handle,
                &event_channel_handle,
            );
        });

        // blocks thread until client disconnects
        handle_inbound_origin(&mut stream, &client_channel, &command_channel);

        // client has disconnected
        global_state().set_client(None);
        client_channel.send(bloom_api::ClientEvent::Disconnected);
    }
}
