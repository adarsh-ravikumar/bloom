use crate::BloomError;

pub trait Packet: Sized {
    fn from_bytes(bytes: Vec<u8>) -> Result<Self, BloomError>;
    fn to_bytes(&self) -> Result<Vec<u8>, BloomError>;
}
