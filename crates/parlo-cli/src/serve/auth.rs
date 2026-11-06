use std::time::{SystemTime, UNIX_EPOCH};

pub fn new_token() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or_default();
    format!("{nanos:016x}{:08x}", std::process::id())
}

pub(crate) fn authorised(headers: &[(String, String)], token: &str) -> bool {
    headers.iter().any(|(name, value)| {
        if name == "authorization" {
            value
                .strip_prefix("Bearer ")
                .is_some_and(|candidate| constant_time_eq(candidate.trim(), token))
        } else if name == "x-parlo-token" {
            constant_time_eq(value, token)
        } else {
            false
        }
    })
}

pub(crate) fn constant_time_eq(left: &str, right: &str) -> bool {
    let left = left.as_bytes();
    let right = right.as_bytes();
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right)
        .fold(0u8, |acc, (a, b)| acc | (a ^ b))
        == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_without_short_circuiting_on_length() {
        assert!(!constant_time_eq("abc", "ab"));
        assert!(constant_time_eq("abc", "abc"));
    }
}
