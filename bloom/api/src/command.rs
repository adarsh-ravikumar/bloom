use serde::{Deserialize, Serialize};
use serde_json::{self};

use crate::{BloomError, Packet};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
