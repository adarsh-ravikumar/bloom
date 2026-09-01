use serde::{Deserialize, Serialize};
use serde_json::{self};

use crate::{BloomError, Packet};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
