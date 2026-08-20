use std::net::TcpListener;

use bloom_api::BloomError;

pub fn find_available_port(port: u16) -> Result<u16, BloomError> {
    let mut current = port;
    let total_ports_to_try = 50;

    while (current - port) < total_ports_to_try {
        let listener = TcpListener::bind(format!("127.0.0.1:{current}"));
        if listener.is_ok() {
            drop(listener);
            return Ok(current);
        }

        current += 1;
    }

    Err(BloomError::FindPortFailed)
}
