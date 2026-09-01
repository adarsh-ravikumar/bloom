use tauri::AppHandle;
use tauri_plugin_log::fern;

use bloom_api::{
    AppEvent, ClientChannel, CommandChannel, EventChannel, FrontendChannel,
    state::global_state,
};

use bloom_host;
use bloom_logger::TAURI_STREAM;

use crate::inbound::handle_inbound_terminal;

pub type Shutdown = Box<dyn Fn(i32) + Send>;

pub fn setup_tauri_logging(app_handle: &AppHandle) {
    if cfg!(debug_assertions) {
        let tauri_dispatch =
            fern::Dispatch::new().chain(fern::Output::call(move |record| {
                bloom_logger::log(TAURI_STREAM, record.args());
            }));

        app_handle
            .plugin(
                tauri_plugin_log::Builder::default()
                    .level(log::LevelFilter::Info)
                    .clear_targets()
                    .target(tauri_plugin_log::Target::new(
                        tauri_plugin_log::TargetKind::Dispatch(tauri_dispatch),
                    ))
                    .build(),
            )
            .unwrap();
    }
}

pub fn spawn_host(
    client_channel: &ClientChannel,
    command_channel: &CommandChannel,
    event_channel: &EventChannel,
) {
    let client_handle = client_channel.clone();
    let command_handle = command_channel.clone();
    let event_handle = event_channel.clone();

    std::thread::spawn(move || {
        bloom_host::handle_client_lifecycle(
            client_handle,
            command_handle,
            event_handle,
        )
        .unwrap();
    });
}

pub fn spawn_inbound_terminal(
    app_handle: &AppHandle,
    command_channel: &CommandChannel,
    frontend_channel: &FrontendChannel,
) {
    let app_handle = app_handle.clone();
    let command_channel = command_channel.clone();
    let frontend_channel = frontend_channel.clone();

    std::thread::spawn(move || {
        let _ = handle_inbound_terminal(
            app_handle,
            command_channel,
            frontend_channel,
        );
    });
}

pub fn handle_app_events(app_handle: &AppHandle) {
    let app_handle = app_handle.clone();

    std::thread::spawn(move || {
        loop {
            let msg = global_state().app_channel.recv();

            match msg {
                AppEvent::Terminate => app_handle.exit(1),

                AppEvent::SetClient(client) => {
                    *global_state().client.lock().unwrap() = client;
                }
            }
        }
    });
}

pub fn run_pipeline(
    app_handle: &AppHandle,
    start_pipeline: impl FnOnce(Shutdown) + Send + 'static,
) {
    let app_handle = app_handle.clone();

    start_pipeline(Box::new(move |code: i32| {
        app_handle.exit(code);
    }));
}
