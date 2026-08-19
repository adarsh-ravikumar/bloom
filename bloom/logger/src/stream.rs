use std::process::{ChildStderr, ChildStdout};

use crate::style::Style;

pub enum ProcessStream {
    Stdout(ChildStdout),
    Stderr(ChildStderr),
}

#[derive(Clone, Copy)]
pub struct StreamConfig {
    pub label: &'static str,
    pub color: &'static str,
}

pub const BLOOM_STREAM: StreamConfig = StreamConfig {
    label: "bloom(host)",
    color: Style::MAGENTA,
};

pub const BLOOM_DEV_STREAM: StreamConfig = StreamConfig {
    label: "bloom(dev)",
    color: Style::CYAN,
};

pub const VITE_STREAM: StreamConfig = StreamConfig {
    label: "vite",
    color: Style::YELLOW,
};

pub const TAURI_STREAM: StreamConfig = StreamConfig {
    label: "tauri",
    color: Style::GREEN,
};

pub const CLIENT_STREAM: StreamConfig = StreamConfig {
    label: "bloom(client)",
    color: Style::BLUE,
};
