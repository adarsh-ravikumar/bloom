use crate::{
    pipeline::{Pipeline, Process},
    utils::{find_available_port, run_tauri},
};

pub fn dev() -> Result<(), ()> {
    let mut pipeline = Pipeline::new();

    let port = find_available_port(3141).ok_or(())?;

    pipeline.push_process(Process::new(
        "bun",
        [
            "--cwd",
            "./app",
            "dev",
            "--host",
            "127.0.0.1",
            "--port",
            &port.to_string(),
            "--strictPort",
        ],
        bloom_logger::VITE_STREAM,
        true,
        None,
    ));

    pipeline.push_process(Process::new(
        "./client/run.sh",
        vec![""],
        bloom_logger::CLIENT_STREAM,
        false,
        Some("./client/py/src".into()),
    ));

    bloom_logger::log(
        bloom_logger::BLOOM_DEV_STREAM,
        "Starting Bloom in development mode...",
    );

    run_tauri(pipeline, port);

    Ok(())
}
