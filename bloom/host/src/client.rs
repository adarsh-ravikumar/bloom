use crossbeam_channel::unbounded;
use std::{io::Read, net::TcpListener};

use bloom_api::{BloomConnection, BloomError, ClientTx};

pub fn handle_client_lifecycle(client_tx: ClientTx) -> Result<(), BloomError> {
    let listener = TcpListener::bind("127.0.0.1:31415")
        .map_err(|_| BloomError::TcpBindFailed)?;

    let mut connection_packet = [0u8; 44];

    let (disconnect_tx, disconnect_rx) = unbounded();

    loop {
        let (mut stream, _) = listener
            .accept()
            .map_err(|_| BloomError::ClientConnectionFailed)?;

        stream
            .read_exact(&mut connection_packet)
            .map_err(|_| BloomError::TcpReadFailed)?;

        let disconnect_tx = disconnect_tx.clone();

        let connection = BloomConnection::new(
            connection_packet.into(),
            stream,
            disconnect_tx,
        )?;

        client_tx
            .send(bloom_api::ClientEvent::Connected(connection))
            .map_err(|_| BloomError::ChannelSendError)?;

        // block  until client disconnects
        disconnect_rx
            .recv()
            .map_err(|_| BloomError::ChannelRecieveError)?;
    }
}
