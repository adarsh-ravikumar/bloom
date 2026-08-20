use bloom_api::BloomError;

use crate::{
    pipeline::{Pipeline, Process},
    utils::{find_available_port, run_tauri},
};

pub fn dev() -> Result<(), BloomError> {
    let mut pipeline = Pipeline::new();

    let port = find_available_port(3141)?;

    let project = bloom_api::ProjectConfig::load()?;

    let mut args = project.dev.frontend.args.clone();
    args.push("--port".into());
    args.push(format!("{port}"));

    pipeline.push_process(Process::new(
        project.dev.frontend.command,
        args,
        bloom_logger::FRONTEND_STREAM,
        true,
        project.dev.frontend.watch,
    ));

    pipeline.push_process(Process::new(
        project.dev.client.command,
        project.dev.client.args,
        bloom_logger::CLIENT_STREAM,
        false,
        project.dev.client.watch,
    ));

    bloom_logger::log(
        bloom_logger::DEV_STREAM,
        "Starting Bloom in development mode...",
    );

    run_tauri(pipeline, port);

    Ok(())
}
