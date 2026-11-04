use std::io::Write;
use std::net::TcpStream;
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Duration;

pub fn stream<T>(
    stream: &mut TcpStream,
    events: Receiver<T>,
    render: impl Fn(&T) -> (String, bool),
) -> Result<(), String> {
    stream
        .write_all(
            b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        )
        .map_err(|error| error.to_string())?;
    loop {
        match events.recv_timeout(Duration::from_secs(15)) {
            Ok(event) => {
                let (payload, terminal) = render(&event);
                if stream
                    .write_all(format!("data: {payload}\n\n").as_bytes())
                    .is_err()
                {
                    break;
                }
                let _ = stream.flush();
                if terminal {
                    break;
                }
            }
            Err(RecvTimeoutError::Timeout) => {
                if stream.write_all(b": keep-alive\n\n").is_err() {
                    break;
                }
                let _ = stream.flush();
            }
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
    Ok(())
}
