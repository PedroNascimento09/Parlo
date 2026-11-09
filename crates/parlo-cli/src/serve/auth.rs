pub fn new_token() -> Result<String, String> {
    let mut bytes = [0u8; 24];
    rustls::crypto::aws_lc_rs::default_provider()
        .secure_random
        .fill(&mut bytes)
        .map_err(|_| "cannot generate a secure token".to_string())?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
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
