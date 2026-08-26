use bloom_host;
use bloom_logger::TAURI_STREAM;
use crossbeam_channel::unbounded;
use tauri_plugin_log::fern;

mod inbound;
mod outbound;
mod utils;

use crate::{
    inbound::{
        frontend_connected, frontend_disconnected, handle_inbound_terminal,
    },
    outbound::emit_event,
    utils::wait_until_server_starts,
};

type Shutdown = Box<dyn Fn(i32) + Send>;

pub fn run(port: u16, start_pipeline: impl FnOnce(Shutdown) + Send + 'static) {
    let (command_tx, command_rx) = unbounded();
    let (event_tx, event_rx) = unbounded();
    let (frontend_tx, frontend_rx) = unbounded();

    tauri::Builder::default()
        .setup(move |app| {
            // setup tauri app
            if cfg!(debug_assertions) {
                let tauri_dispatch =
                    fern::Dispatch::new().chain(fern::Output::call(move |record| {
                        bloom_logger::log(TAURI_STREAM, record.args());
                    }));

                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .clear_targets()
                        .target(tauri_plugin_log::Target::new(
                            tauri_plugin_log::TargetKind::Dispatch(tauri_dispatch),
                        ))
                        .build(),
                )?;
            }

            // spawn host and inbound terminal worker threads
            std::thread::spawn(move || {
                bloom_host::init(command_tx, event_rx).unwrap();
            });

            let command_handle = app.handle().clone();
            std::thread::spawn(move || {
                let _ = handle_inbound_terminal(command_handle, command_rx, frontend_rx);
            });

            // run the pipeline
            let shutdown_handle = app.handle().clone();
            start_pipeline(Box::new(move |code: i32| {
                shutdown_handle.exit(code);
            }));

            if wait_until_server_starts(port, 20, 200).is_err() {
                bloom_logger::log(TAURI_STREAM, "Failed to connect to the frontend. Recheck the host and port configuration on your bundler.");
                app.handle().exit(1);
            }

            // build the window
            let url = format!("http://127.0.0.1:{port}").parse().unwrap();
            tauri::WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::External(url))
                .build()?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            emit_event,
            frontend_connected,
            frontend_disconnected
        ])
        .manage(event_tx)
        .manage(frontend_tx)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
