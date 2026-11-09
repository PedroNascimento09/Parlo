use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

pub(crate) const TTL_SECONDS: u64 = 30;
const TTL: Duration = Duration::from_secs(TTL_SECONDS);

fn store() -> &'static Mutex<HashMap<String, Instant>> {
    static STORE: OnceLock<Mutex<HashMap<String, Instant>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(crate) fn issue() -> Result<(String, u64), String> {
    let ticket = super::auth::new_token()?;
    let now = Instant::now();
    let mut live = store()
        .lock()
        .map_err(|_| "the ticket store is unavailable")?;
    live.retain(|_, at| now.saturating_duration_since(*at) <= TTL);
    live.insert(ticket.clone(), now);
    Ok((ticket, TTL_SECONDS))
}

pub(crate) fn redeem(ticket: &str) -> bool {
    if ticket.is_empty() {
        return false;
    }
    let Ok(mut live) = store().lock() else {
        return false;
    };
    match live.remove(ticket) {
        Some(at) => at.elapsed() <= TTL,
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_ticket_is_spent_once() {
        let (ticket, lifetime) = issue().unwrap();
        assert_eq!(lifetime, TTL_SECONDS);
        assert!(redeem(&ticket));
        assert!(!redeem(&ticket));
    }

    #[test]
    fn two_tickets_are_not_the_same() {
        let (first, _) = issue().unwrap();
        let (second, _) = issue().unwrap();
        assert_ne!(first, second);
    }

    #[test]
    fn a_ticket_older_than_its_lease_is_refused() {
        let ticket = "an-old-one".to_string();
        let issued = Instant::now()
            .checked_sub(TTL + Duration::from_secs(1))
            .unwrap();
        store().lock().unwrap().insert(ticket.clone(), issued);
        assert!(!redeem(&ticket));
        assert!(!redeem(&ticket));
    }

    #[test]
    fn an_unknown_ticket_is_refused() {
        assert!(!redeem(""));
        assert!(!redeem("never-issued"));
    }
}
