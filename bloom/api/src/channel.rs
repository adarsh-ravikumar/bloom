use crossbeam_channel::{Receiver, Sender};

use crate::{BloomCommand, BloomConnection, BloomEvent};

pub type CommandTx = Sender<BloomCommand>;
pub type CommandRx = Receiver<BloomCommand>;

pub type EventTx = Sender<BloomEvent>;
pub type EventRx = Receiver<BloomEvent>;

#[derive(Clone)]
pub enum ClientEvent {
    Connected(BloomConnection),
    Disconnected,
}

pub enum FrontendEvent {
    Connected,
    Disconnected,
}

pub type ClientTx = Sender<ClientEvent>;
pub type ClientRx = Receiver<ClientEvent>;

pub type FrontendTx = Sender<FrontendEvent>;
pub type FrontendRx = Receiver<FrontendEvent>;
