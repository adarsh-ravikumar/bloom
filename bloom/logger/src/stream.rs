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

pub const HOST_STREAM: StreamConfig = StreamConfig {
    label: "host",
    color: Style::MAGENTA,
};

pub const DEV_STREAM: StreamConfig = StreamConfig {
    label: "dev",
    color: Style::CYAN,
};

pub const FRONTEND_STREAM: StreamConfig = StreamConfig {
    label: "frontend",
    color: Style::YELLOW,
};

pub const TAURI_STREAM: StreamConfig = StreamConfig {
    label: "tauri",
    color: Style::GREEN,
};

pub const CLIENT_STREAM: StreamConfig = StreamConfig {
    label: "client",
    color: Style::BLUE,
};
