use bloom_api::{BloomError, CommandTx, EventRx};
use crossbeam_channel::unbounded;

use crate::{
    client::handle_client_lifecycle, inbound::handle_inbound_origin,
    outbound::handle_outbound_terminal,
};

mod client;
mod inbound;
mod outbound;

pub fn init(
    command_tx: CommandTx,
    event_rx: EventRx,
) -> Result<(), BloomError> {
    let (client_tx, client_rx) = unbounded();

    let (inbound_client_tx, inbound_client_rx) = unbounded();
    let (outbound_client_tx, outbound_client_rx) = unbounded();

    let lifecylce_tx_handle = client_tx.clone();
    std::thread::spawn(move || {
        let _ = handle_client_lifecycle(lifecylce_tx_handle);
    });

    // broadcaster
    let broadcaster_handle = client_rx.clone();

    std::thread::spawn(move || {
        loop {
            let Ok(msg) = broadcaster_handle.recv() else {
                continue;
            };

            let inbound_msg = msg.clone();
            let outbound_msg = msg.clone();

            let _ = inbound_client_tx.send(inbound_msg);
            let _ = outbound_client_tx.send(outbound_msg);
        }
    });

    let inbound_tx_handle = client_tx.clone();
    std::thread::spawn(move || {
        let _ = handle_inbound_origin(
            inbound_tx_handle,
            inbound_client_rx,
            command_tx,
        );
    });

    std::thread::spawn(move || {
        println!("spawning outbound thread");
        let _ = handle_outbound_terminal(outbound_client_rx, event_rx);
    });

    Ok(())
}
