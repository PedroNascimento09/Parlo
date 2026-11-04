const MAX_PARTS: usize = 16;
const MAX_HEADER: usize = 8 * 1024;
const MAX_FIELD: usize = 1024 * 1024;
const MAX_FILE: usize = 25 * 1024 * 1024;

pub(crate) struct Part {
    pub(crate) name: String,
    pub(crate) data: Vec<u8>,
}

pub(crate) fn parse(content_type: &str, body: &[u8]) -> Result<Vec<Part>, String> {
    let boundary =
        boundary(content_type).ok_or_else(|| "expected a multipart/form-data body".to_string())?;
    if boundary.is_empty() || boundary.len() > 70 {
        return Err("the multipart boundary is not usable".to_string());
    }
    let delimiter = format!("--{boundary}");
    let delimiter = delimiter.as_bytes();
    if !body.starts_with(delimiter) {
        return Err("the multipart body does not start with its boundary".to_string());
    }

    let mut parts = Vec::new();
    let mut position = 0;
    loop {
        if !body[position..].starts_with(delimiter) {
            return Err("a multipart boundary is missing".to_string());
        }
        position += delimiter.len();
        if body[position..].starts_with(b"--") {
            break;
        }
        if !body[position..].starts_with(b"\r\n") {
            return Err("a multipart boundary is not followed by a line break".to_string());
        }
        position += 2;

        let head_end = find(&body[position..], b"\r\n\r\n")
            .map(|offset| position + offset)
            .ok_or_else(|| "a multipart part has no headers".to_string())?;
        if head_end - position > MAX_HEADER {
            return Err("a multipart part has headers that are too large".to_string());
        }
        let head = std::str::from_utf8(&body[position..head_end])
            .map_err(|_| "a multipart part has non-text headers".to_string())?;
        position = head_end + 4;

        let next = find(&body[position..], b"\r\n--")
            .map(|offset| position + offset)
            .ok_or_else(|| "a multipart part has no closing boundary".to_string())?;
        let data = body[position..next].to_vec();
        position = next + 2;

        let (name, filename) = disposition(head)?;
        let limit = if filename.is_some() {
            MAX_FILE
        } else {
            MAX_FIELD
        };
        if data.len() > limit {
            return Err("a multipart part is too large".to_string());
        }
        if parts.len() == MAX_PARTS {
            return Err("the form has too many parts".to_string());
        }
        parts.push(Part { name, data });
    }
    Ok(parts)
}

fn boundary(content_type: &str) -> Option<String> {
    if !content_type
        .trim_start()
        .to_ascii_lowercase()
        .starts_with("multipart/form-data")
    {
        return None;
    }
    content_type.split(';').skip(1).find_map(|piece| {
        let (key, value) = piece.trim().split_once('=')?;
        if key.trim().eq_ignore_ascii_case("boundary") {
            Some(value.trim().trim_matches('"').to_string())
        } else {
            None
        }
    })
}

fn disposition(head: &str) -> Result<(String, Option<String>), String> {
    let mut name = None;
    let mut filename = None;
    for line in head.split("\r\n") {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        if !key.trim().eq_ignore_ascii_case("content-disposition") {
            continue;
        }
        for piece in value.split(';').skip(1) {
            let piece = piece.trim();
            if let Some(value) = piece.strip_prefix("name=") {
                name = Some(unquote(value));
            } else if let Some(value) = piece.strip_prefix("filename=") {
                filename = Some(unquote(value));
            }
        }
    }
    let name = name
        .filter(|name| !name.is_empty())
        .ok_or_else(|| "a multipart part has no name".to_string())?;
    Ok((name, filename))
}

fn unquote(value: &str) -> String {
    value.trim().trim_matches('"').to_string()
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}
