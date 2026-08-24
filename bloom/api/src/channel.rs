use std::sync::mpsc::{Receiver, Sender, channel};

use crate::{BloomCommand, BloomEvent};

pub struct Channel<T> {
    pub tx: Sender<T>,
    pub rx: Receiver<T>,
}

impl<T> Channel<T> {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Channel { tx, rx }
    }
}

pub type InboundChannel = Channel<BloomCommand>;
pub type OutboundChannel = Channel<BloomEvent>;

pub type InboundTX = Sender<BloomCommand>;
pub type InboundRX = Receiver<BloomCommand>;

pub type OutboundTX = Sender<BloomEvent>;
pub type OutboundRX = Receiver<BloomEvent>;
