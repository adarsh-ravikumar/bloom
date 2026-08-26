use bloom_api::{BloomEvent, EventTx};
use tauri::State;

#[tauri::command]
pub fn emit_event(event: BloomEvent, event_tx: State<'_, EventTx>) {
    println!("[OUTBOUND ORIGIN] Recieved event {}", event.event);
    let _ = event_tx.send(event);
}
