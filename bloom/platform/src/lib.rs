mod inbound;
mod outbound;
mod setup;
mod utils;

use bloom_api::{
    Channel,
    state::{self, GlobalState},
};
use bloom_logger::TAURI_STREAM;

use setup::*;

use crate::{
    inbound::{frontend_connected, frontend_disconnected},
    outbound::emit_event,
    utils::wait_until_server_starts,
};

pub fn run(port: u16, start_pipeline: impl FnOnce(Shutdown) + Send + 'static) {
    let command_channel = Channel::new();
    let event_channel = Channel::new();
    let frontend_channel = Channel::new();
    let app_channel = Channel::new();
    let client_channel = Channel::new();

    state::init(GlobalState::new(app_channel.clone()));

    let event_channel_handle = event_channel.clone();
    let frontend_channel_handle = frontend_channel.clone();
    tauri::Builder::default()
        .setup(move |app| {
            // setup tauri app
            setup_tauri_logging(app.handle());
            spawn_host(&client_channel, &command_channel, &event_channel);
            spawn_inbound_terminal(app.handle(), &command_channel, &frontend_channel);
            handle_app_events(app.handle());
            run_pipeline(app.handle(), start_pipeline);

            if wait_until_server_starts(port, 20, 200).is_err() {
                bloom_logger::log(
                    TAURI_STREAM,
                    "Failed to connect to the frontend. Recheck the host and port configuration on your bundler.",
                );
                app.handle().exit(1);
            }

            // build the window
            let url = format!("http://127.0.0.1:{port}").parse().unwrap();
            tauri::WebviewWindowBuilder::new(
                app,
                "main",
                tauri::WebviewUrl::External(url),
            )
            .build()?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            emit_event,
            frontend_connected,
            frontend_disconnected
        ])
        .manage(event_channel_handle)
        .manage(frontend_channel_handle)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
