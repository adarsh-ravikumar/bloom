use std::{net::TcpStream, time::Duration};

use bloom_api::BloomError;

pub fn wait_until_server_starts(
    port: u16,
    total_attempts: u32,
    attempt_cooldown_ms: u64,
) -> Result<(), BloomError> {
    let address = format!("127.0.0.1:{port}").parse().unwrap();

    for _ in 0..total_attempts {
        if TcpStream::connect_timeout(&address, Duration::from_millis(200))
            .is_ok()
        {
            return Ok(());
        }

        std::thread::sleep(Duration::from_millis(attempt_cooldown_ms));
    }

    Err(BloomError::ConnectToFrontendFailed)
}
