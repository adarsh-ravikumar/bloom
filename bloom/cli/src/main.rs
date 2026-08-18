// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::pipeline::dev;

mod pipeline;
mod utils;

fn main() {
    let command = std::env::args().nth(1);

    match command.as_deref() {
        Some("dev") => {
            println!("Starting Bloom in development mode...");
            let _ = dev();
        }

        _ => {
            println!("Usage: bloom dev");
        }
    }
}
