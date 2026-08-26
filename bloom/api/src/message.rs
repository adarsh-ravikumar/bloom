use serde::{Deserialize, Serialize};
use serde_json::{self};

use crate::{BloomError, Packet};

#[derive(Debug, Serialize, Deserialize)]
pub struct BloomCommand {
    pub command: String,
    pub data: serde_json::Value,
}

impl Packet for BloomCommand {
    fn from_bytes(bytes: Vec<u8>) -> Result<Self, crate::BloomError> {
        serde_json::from_slice(&bytes).map_err(|e| {
            println!("{:?}", e);
            BloomError::InvalidPacket
        })
    }

    fn to_bytes(&self) -> Result<Vec<u8>, crate::BloomError> {
        Err(BloomError::InvalidMessageUse)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BloomEvent {
    pub event: String,
    pub payload: serde_json::Value,
}

impl Packet for BloomEvent {
    fn from_bytes(_: Vec<u8>) -> Result<Self, crate::BloomError> {
        Err(BloomError::InvalidMessageUse)
    }

    fn to_bytes(&self) -> Result<Vec<u8>, crate::BloomError> {
        serde_json::to_vec(self).map_err(|_| BloomError::InvalidPacket)
    }
}
