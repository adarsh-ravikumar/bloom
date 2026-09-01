use crossbeam_channel::{Receiver, Sender, unbounded};

use crate::{BloomCommand, BloomConnection, BloomEvent, state::global_state};

#[derive(Debug, Clone)]
pub struct Channel<T> {
    tx: Sender<T>,
    rx: Receiver<T>,
}

impl<T> Channel<T> {
    pub fn new() -> Self {
        let (tx, rx) = unbounded();
        Self { tx, rx }
    }

    // Channel disconnection is fatal to the application.
    pub fn send(&self, value: T) {
        if self.tx.send(value).is_err() {
            global_state().terminate_app();
        }
    }

    pub fn recv(&self) -> T {
        match self.rx.recv() {
            Ok(val) => val,
            Err(_) => {
                global_state().terminate_app();
                unreachable!();
            }
        }
    }

    pub fn rx(&self) -> &Receiver<T> {
        &self.rx
    }
}

#[derive(Debug, Clone)]
pub enum ClientEvent {
    Connected(BloomConnection),
    Command(BloomCommand),
    Disconnected,
}

#[derive(Debug, Clone)]
pub enum FrontendEvent {
    Connected,
    Disconnected,
}

#[derive(Debug, Clone)]
pub enum AppEvent {
    Terminate,
    SetClient(Option<BloomConnection>),
}

pub type CommandChannel = Channel<BloomCommand>;
pub type EventChannel = Channel<BloomEvent>;
pub type ClientChannel = Channel<ClientEvent>;
pub type FrontendChannel = Channel<FrontendEvent>;
pub type AppChannel = Channel<AppEvent>;
