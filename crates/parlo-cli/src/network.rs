use crate::update::LATEST;

const CALLS: [&str; 5] = [
    "fetch(",
    "new WebSocket(",
    "new EventSource(",
    "sendBeacon(",
    "src=",
];

const SCHEMES: [&str; 4] = ["https://", "http://", "wss://", "ws://"];

pub fn targets(source: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for call in CALLS {
        for (at, _) in source.match_indices(call) {
            let rest = &source[at + call.len()..];
            let quoted = rest.trim_start_matches(['"', '\'', '`', ' ', '\n', '\t']);
            let Some(url) = absolute(quoted) else {
                continue;
            };
            if !found.iter().any(|seen| seen == url) {
                found.push(url.to_string());
            }
        }
    }
    found
}

pub fn leaving(source: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for url in targets(source) {
        let Some(host) = host_of(&url) else {
            continue;
        };
        if local(host) || found.iter().any(|seen| seen == host) {
            continue;
        }
        found.push(host.to_string());
    }
    found
}

pub fn hosts(source: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for scheme in SCHEMES {
        for (at, _) in source.match_indices(scheme) {
            let Some(host) = host_of(&source[at..]) else {
                continue;
            };
            if !found.iter().any(|seen| seen == host) {
                found.push(host.to_string());
            }
        }
    }
    found.sort();
    found
}

fn absolute(text: &str) -> Option<&str> {
    if !SCHEMES.iter().any(|scheme| text.starts_with(scheme)) {
        return None;
    }
    let end = text
        .find(|character: char| {
            character.is_whitespace()
                || matches!(character, '"' | '\'' | '`' | ')' | '<' | '>' | ',')
        })
        .unwrap_or(text.len());
    Some(&text[..end])
}

fn host_of(text: &str) -> Option<&str> {
    let rest = SCHEMES
        .iter()
        .find_map(|scheme| text.strip_prefix(scheme))?;
    let end = rest
        .find(|character: char| {
            !(character.is_ascii_alphanumeric()
                || character == '.'
                || character == '-'
                || character == ':'
                || character == '['
                || character == ']')
        })
        .unwrap_or(rest.len());
    let authority = rest[..end].trim_start_matches('[');
    let host = authority.split([']', ':']).next().unwrap_or("");
    if host.is_empty() {
        None
    } else {
        Some(host)
    }
}

fn local(host: &str) -> bool {
    if host == "localhost" || host.ends_with(".localhost") {
        return true;
    }
    match host.parse::<std::net::IpAddr>() {
        Ok(std::net::IpAddr::V4(address)) => {
            address.is_loopback()
                || address.is_private()
                || address.is_link_local()
                || address.is_unspecified()
        }
        Ok(std::net::IpAddr::V6(address)) => {
            address.is_loopback() || address.is_unique_local() || address.is_unspecified()
        }
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ASSETS: [(&str, &str); 5] = [
        ("app.js", include_str!("../web/app.js")),
        ("index.html", include_str!("../web/index.html")),
        ("captions.html", include_str!("../web/captions.html")),
        ("island.html", include_str!("../web/island.html")),
        ("live.html", include_str!("../web/live.html")),
    ];

    #[test]
    fn the_app_asks_this_machine_for_everything_but_the_release() {
        assert_eq!(
            targets(include_str!("../web/app.js")),
            vec![LATEST.to_string()]
        );
        for (name, source) in ASSETS {
            let outside = leaving(source);
            let expected = if name == "app.js" {
                vec!["api.github.com".to_string()]
            } else {
                Vec::new()
            };
            assert_eq!(outside, expected, "{name}");
        }
    }

    #[test]
    fn a_request_to_another_machine_is_caught() {
        assert_eq!(
            leaving("fetch(\"https://collect.example.com/ping\")"),
            vec!["collect.example.com"]
        );
        assert_eq!(
            leaving("<script src='https://cdn.example.com/a.js'></script>"),
            vec!["cdn.example.com"]
        );
        assert_eq!(
            leaving("new WebSocket(\"wss://relay.example.com\")"),
            vec!["relay.example.com"]
        );
        assert_eq!(
            targets("fetch(\"https://collect.example.com/ping\")"),
            vec!["https://collect.example.com/ping"]
        );
    }

    #[test]
    fn a_request_to_this_machine_is_not_caught() {
        for source in [
            "fetch(\"/models\")",
            "fetch(\"http://127.0.0.1:8787/models\")",
            "new WebSocket(\"ws://localhost:8787/stream\")",
            "<script src=\"/app.js\"></script>",
            "<img src=\"data:image/png;base64,AA\">",
        ] {
            assert!(leaving(source).is_empty(), "{source}");
        }
    }

    #[test]
    fn the_update_check_is_the_only_place_the_cli_reaches_out() {
        assert_eq!(hosts(include_str!("update.rs")), vec!["api.github.com"]);
    }

    #[test]
    fn the_catalogue_downloads_from_the_registry_hosts_only() {
        assert_eq!(
            hosts(include_str!("../../../catalog/models.json")),
            vec!["github.com", "huggingface.co"]
        );
    }
}
