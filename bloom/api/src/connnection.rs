use crate::{BloomCommand, BloomError, Packet};

pub struct BloomConnection {
    // Connection Packet -> 44 bytes
    // : Client Version -> [major, minor, patch] 3 bytes
    // : ID ->             8 bytes (8 random characters)
    // : App name       -> 32 bytes [UTF-8 encoded string, NUL-padded]
    // : NUL-Padding    -> 1 byte
    client_ver: (u8, u8, u8),
    client_id: String,
    app_name: String,
}

impl Packet for BloomConnection {
    fn from_bytes(bytes: Vec<u8>) -> Result<Self, BloomError> {
        if bytes.len() != 44 {
            return Err(BloomError::InvalidPacketSize);
        }

        let major_ver = bytes[0];
        let minor_ver = bytes[1];
        let patch_ver = bytes[2];

        let id = str::from_utf8(&bytes[3..11]).map_err(|_| BloomError::InvalidClientId)?;

        let app_name_bytes = &bytes[11..43];
        let app_name_end = app_name_bytes
            .iter()
            .position(|&byte| byte == 0)
            .unwrap_or(app_name_bytes.len());

        let app_name = str::from_utf8(&app_name_bytes[..app_name_end])
            .map_err(|_| BloomError::InvalidAppName)?;

        if bytes[43] != '\0' as u8 {
            return Err(BloomError::PacketNotTerminated);
        }

        Ok(Self {
            client_ver: (major_ver, minor_ver, patch_ver),
            client_id: id.into(),
            app_name: app_name.into(),
        })
    }

    fn to_bytes(&self) -> Result<Vec<u8>, BloomError> {
        let mut packet = [0u8; 44];

        // client version
        packet[0] = self.client_ver.0;
        packet[1] = self.client_ver.1;
        packet[2] = self.client_ver.2;

        // client id
        let id = self.client_id.as_bytes();

        if id.len() != 8 {
            return Err(BloomError::InvalidClientId);
        }

        packet[3..11].copy_from_slice(id);

        // app name
        let app_name = self.app_name.as_bytes();

        if app_name.len() > 32 {
            return Err(BloomError::AppNameTooLong);
        }

        let end = app_name.len() + 11;
        packet[11..end].copy_from_slice(app_name);

        // NUL-terminator (as per spec)
        packet[43] = 0;

        Ok(packet.into())
    }
}

impl BloomConnection {
    pub fn into_command(&self) -> Result<BloomCommand, BloomError> {
        let data = serde_json::json!({
            "client_ver": self.client_ver,
            "client_id": self.client_id,
            "app_name": self.app_name
        });

        Ok(BloomCommand {
            command: "client-connected".into(),
            panel_id: 0,
            data,
        })
    }
}
