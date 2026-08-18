use std::{
    fmt::Display,
    io::{BufRead, BufReader},
};

use crate::{
    stream::{ProcessStream, StreamConfig},
    style::Style,
};

pub fn spawn_log_stream(config: StreamConfig, stream: ProcessStream) {
    std::thread::spawn(move || match stream {
        ProcessStream::Stdout(stdout) => {
            let reader = BufReader::new(stdout);
            for line in reader.lines().flatten() {
                log(config, line);
            }
        }

        ProcessStream::Stderr(stderr) => {
            let reader = BufReader::new(stderr);
            for line in reader.lines().flatten() {
                log(config, line);
            }
        }
    });
}

pub fn log(config: StreamConfig, message: impl Display) {
    println!(
        "{}[{}]{} {message}",
        config.color,
        config.label,
        Style::RESET
    );
}
