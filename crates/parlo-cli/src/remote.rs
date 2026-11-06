use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::time::Duration;

use rustls::pki_types::ServerName;
use rustls::RootCertStore;

pub fn check_url(value: &str) -> Result<String, String> {
    let trimmed = value.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Ok(String::new());
    }
    if !trimmed.starts_with("https://") || trimmed.len() <= "https://".len() {
        return Err("the remote backend must be an https address".to_string());
    }
    Ok(trimmed.to_string())
}

pub fn target(base: &str, path: &str) -> Result<String, String> {
    let base = check_url(base)?;
    if base.is_empty() {
        return Err(
            "no remote backend is configured, run parlo config --remote-url https://host:8788"
                .to_string(),
        );
    }
    let path = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    };
    Ok(format!("{base}{path}"))
}

fn roots(ca: &str) -> Result<RootCertStore, String> {
    let mut store = RootCertStore::empty();
    for certificate in rustls_native_certs::load_native_certs().certs {
        let _ = store.add(certificate);
    }
    if !ca.trim().is_empty() {
        let bytes = fs::read(ca.trim()).map_err(|error| format!("{}: {error}", ca.trim()))?;
        for certificate in rustls_pemfile::certs(&mut &bytes[..]) {
            store
                .add(certificate.map_err(|error| error.to_string())?)
                .map_err(|error| format!("{}: {error}", ca.trim()))?;
        }
    }
    if store.is_empty() {
        return Err("no trusted certificate is available, set --remote-ca".to_string());
    }
    Ok(store)
}

#[derive(Debug)]
pub struct Answer {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

pub fn post(
    config: &parlo_core::settings::Config,
    path: &str,
    content_type: &str,
    body: &[u8],
) -> Result<Answer, String> {
    let url = target(&config.remote_url, path)?;
    let rest = url.trim_start_matches("https://");
    let (authority, address) = rest.split_once('/').unwrap_or((rest, ""));
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) => (
            host.to_string(),
            port.parse::<u16>()
                .map_err(|_| "the remote port must be a number".to_string())?,
        ),
        None => (authority.to_string(), 443),
    };
    let name = ServerName::try_from(host.clone())
        .map_err(|_| format!("{host} is not a valid remote host"))?;
    let client = rustls::ClientConfig::builder()
        .with_root_certificates(roots(&config.remote_ca)?)
        .with_no_client_auth();
    let connection =
        rustls::ClientConnection::new(Arc::new(client), name).map_err(|error| error.to_string())?;
    let stream = TcpStream::connect((host.as_str(), port))
        .map_err(|error| format!("cannot reach {host}:{port}: {error}"))?;
    let _ = stream.set_read_timeout(Some(Duration::from_secs(120)));
    let mut tls = rustls::StreamOwned::new(connection, stream);
    let head = format!(
        "POST /{address} HTTP/1.1\r\nHost: {authority}\r\nAuthorization: Bearer {}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        config.remote_token,
        body.len()
    );
    tls.write_all(head.as_bytes())
        .and_then(|()| tls.write_all(body))
        .and_then(|()| tls.flush())
        .map_err(|error| {
            format!("the remote certificate was refused or the request failed: {error}")
        })?;
    let mut answer = Vec::new();
    let mut chunk = [0u8; 8192];
    loop {
        match tls.read(&mut chunk) {
            Ok(0) => break,
            Ok(read) => answer.extend_from_slice(&chunk[..read]),
            Err(error)
                if error.kind() == std::io::ErrorKind::UnexpectedEof && !answer.is_empty() =>
            {
                break
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    parse(&answer)
}

fn parse(raw: &[u8]) -> Result<Answer, String> {
    let split = raw
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or("the remote answer has no headers")?;
    let head = String::from_utf8_lossy(&raw[..split]).into_owned();
    let mut lines = head.lines();
    let status = lines
        .next()
        .unwrap_or_default()
        .split_whitespace()
        .nth(1)
        .and_then(|code| code.parse::<u16>().ok())
        .ok_or("the remote answer has no status")?;
    let headers = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(name, value)| (name.trim().to_ascii_lowercase(), value.trim().to_string()))
        .collect();
    Ok(Answer {
        status,
        headers,
        body: raw[split + 4..].to_vec(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_remote_address_must_use_https() {
        assert_eq!(
            target(" https://desk.example:8788/ ", "/translate").unwrap(),
            "https://desk.example:8788/translate"
        );
        assert!(target("http://desk.example:8788", "/translate").is_err());
        assert!(target("desk.example:8788", "/translate").is_err());
        assert!(target("", "/translate").is_err());
    }

    #[test]
    fn check_url_normalises_and_refuses_plain_http() {
        assert_eq!(
            check_url(" https://desk.example/ ").unwrap(),
            "https://desk.example"
        );
        assert_eq!(check_url("").unwrap(), "");
        assert!(check_url("http://desk.example").is_err());
        assert!(check_url("https://").is_err());
    }
}
