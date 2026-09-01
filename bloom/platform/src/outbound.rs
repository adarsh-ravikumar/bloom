use bloom_api::{BloomEvent, EventChannel};
use tauri::State;

#[tauri::command]
pub fn emit_event(event: BloomEvent, event_channel: State<'_, EventChannel>) {
    println!("[OUTBOUND ORIGIN] Recieved event {}", event.event);
    event_channel.send(event);
}
