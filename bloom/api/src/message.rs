use serde::{Deserialize, Serialize};
use serde_json;

use crate::{BloomError, Packet};

#[derive(Debug, Serialize, Deserialize)]
pub struct BloomCommand {
    pub command: String,
    pub panel_id: u32,
    pub data: serde_json::Value,
}

impl Packet for BloomCommand {
    fn from_bytes(bytes: Vec<u8>) -> Result<Self, crate::BloomError> {
        let panel_id = u32::from_be_bytes(bytes[0..4].try_into().unwrap()) as u32;
        let specifier = u32::from_be_bytes(bytes[4..8].try_into().unwrap()) as u32;

        let payload = &bytes[8..];

        let data = serde_json::from_slice(payload).map_err(|_| BloomError::InvalidPacket)?;

        Ok(BloomCommand {
            panel_id,
            command: "testing".into(),
            data,
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
    fn from_bytes(bytes: Vec<u8>) -> Result<Self, crate::BloomError> {
        Err(BloomError::InvalidMessageUse)
    }

    fn to_bytes(&self) -> Result<Vec<u8>, crate::BloomError> {
        serde_json::to_vec(self).map_err(|_| BloomError::InvalidPacket)
    }
}
