mod server;
use tauri;

use crate::server::start_server;

pub fn run(_handle: tauri::AppHandle) {
    let _ = start_server();
}
