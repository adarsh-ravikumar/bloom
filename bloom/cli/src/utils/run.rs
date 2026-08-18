use crate::pipeline::Pipeline;

pub fn run_tauri(pipeline: Pipeline, port: u16) {
    bloom_platform::run(port, move |shutdown| {
        let _ = pipeline.start(shutdown);
    });
}
