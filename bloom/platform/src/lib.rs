use bloom_api::BloomError;
use bloom_host;
use bloom_logger::TAURI_STREAM;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;
use tauri_plugin_log::fern;

type Shutdown = Box<dyn Fn(i32) + Send>;

fn wait_until_server_starts(
    port: u16,
    total_attempts: u32,
    attempt_cooldown_ms: u64,
) -> Result<(), BloomError> {
    let address = format!("127.0.0.1:{port}").parse().unwrap();

    for _ in 0..total_attempts {
        if TcpStream::connect_timeout(&address, Duration::from_millis(200)).is_ok() {
            return Ok(());
        }

        thread::sleep(Duration::from_millis(attempt_cooldown_ms));
    }

    Err(BloomError::ConnectToFrontendFailed)
}

pub fn run(port: u16, start_pipeline: impl FnOnce(Shutdown) + Send + 'static) {
    tauri::Builder::default()
        .setup(move |app| {
            let host_handle = app.handle().clone();

            if cfg!(debug_assertions) {
                let tauri_dispatch =
                    fern::Dispatch::new().chain(fern::Output::call(move |record| {
                        bloom_logger::log(TAURI_STREAM, record.args());
                    }));

                host_handle.plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .clear_targets()
                        .target(tauri_plugin_log::Target::new(
                            tauri_plugin_log::TargetKind::Dispatch(tauri_dispatch),
                        ))
                        .build(),
                )?;
            }

            // start the host
            std::thread::spawn(move || {
                bloom_host::run(host_handle);
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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
