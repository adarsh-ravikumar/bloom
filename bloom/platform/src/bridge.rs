use std::sync::mpsc::{self, Receiver, Sender};

use bloom_api::{BloomCommand, BloomError, BloomEvent, OutboundTX};
use tauri::{AppHandle, Emitter, State};

// inbound thread
// host -> tauri -> frontend
// app_handle.emit
// needs app handle, tauri.rx
//
// outbound thread (doesn't need app handle)
// frontend -> tauri -> host
// tauri::command
// needs host.tx

pub fn handle_inbound(rx: Receiver<BloomCommand>, app_handle: AppHandle) -> Result<(), BloomError> {
    let message = rx.recv().map_err(|_| BloomError::ChannelRecieveError)?;

    app_handle.emit("command", &message);

    Ok(())
}

pub fn handle_outbound(tx: Sender<BloomEvent>, packet: BloomEvent) -> Result<(), BloomError> {
    tx.send(packet);
    Ok(())
}

#[tauri::command]
pub fn emit_event(event: BloomEvent, tx: State<'_, OutboundTX>) -> Result<(), String> {
    tx.send(event).map_err(|e| e.to_string())
}
