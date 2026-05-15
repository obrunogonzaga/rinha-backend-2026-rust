use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::process::ExitCode;
use std::time::Duration;

const REQUEST: &[u8] = b"GET /ready HTTP/1.0\r\nHost: localhost\r\nConnection: close\r\n\r\n";
const TIMEOUT: Duration = Duration::from_millis(500);
const ADDR: &str = "127.0.0.1:9999";

pub fn run() -> ExitCode {
    let addr: SocketAddr = ADDR.parse().expect("valid addr");
    if probe(addr) {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn probe(addr: SocketAddr) -> bool {
    let Ok(mut stream) = TcpStream::connect_timeout(&addr, TIMEOUT) else {
        return false;
    };
    if stream.set_read_timeout(Some(TIMEOUT)).is_err() {
        return false;
    }
    if stream.set_write_timeout(Some(TIMEOUT)).is_err() {
        return false;
    }
    if stream.write_all(REQUEST).is_err() {
        return false;
    }
    let mut buf = [0u8; 64];
    let Ok(n) = stream.read(&mut buf) else {
        return false;
    };
    is_status_200(&buf[..n])
}

fn is_status_200(response: &[u8]) -> bool {
    response.len() >= 12 && &response[9..12] == b"200"
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    use std::thread;

    #[test]
    fn is_status_200_accepts_http_1_0_200() {
        assert!(is_status_200(b"HTTP/1.0 200 OK\r\n"));
    }

    #[test]
    fn is_status_200_accepts_http_1_1_200() {
        assert!(is_status_200(b"HTTP/1.1 200 OK\r\n"));
    }

    #[test]
    fn is_status_200_rejects_404() {
        assert!(!is_status_200(b"HTTP/1.1 404 Not Found\r\n"));
    }

    #[test]
    fn is_status_200_rejects_500() {
        assert!(!is_status_200(b"HTTP/1.1 500 Internal\r\n"));
    }

    #[test]
    fn is_status_200_rejects_short_response() {
        assert!(!is_status_200(b"HTTP/1.1"));
        assert!(!is_status_200(b""));
    }

    #[test]
    fn probe_returns_false_when_nothing_listening() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        drop(listener);
        assert!(!probe(addr));
    }

    fn serve_once(response: &'static [u8]) -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        thread::spawn(move || {
            if let Ok((mut sock, _)) = listener.accept() {
                let mut sink = [0u8; 256];
                let _ = sock.set_read_timeout(Some(Duration::from_millis(100)));
                let _ = sock.read(&mut sink);
                let _ = sock.write_all(response);
            }
        });
        addr
    }

    #[test]
    fn probe_returns_true_on_http_200() {
        let addr = serve_once(b"HTTP/1.0 200 OK\r\nContent-Length: 0\r\n\r\n");
        assert!(probe(addr));
    }

    #[test]
    fn probe_returns_false_on_http_500() {
        let addr = serve_once(b"HTTP/1.0 500 Internal Server Error\r\n\r\n");
        assert!(!probe(addr));
    }

    #[test]
    fn probe_returns_false_on_http_404() {
        let addr = serve_once(b"HTTP/1.0 404 Not Found\r\n\r\n");
        assert!(!probe(addr));
    }
}
