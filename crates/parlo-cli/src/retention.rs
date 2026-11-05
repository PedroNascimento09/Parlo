use crate::meeting::Meeting;

pub const DAY_MS: u64 = 86_400_000;

pub fn cutoff_ms(days: u64, now_ms: u64) -> Result<u64, String> {
    if days == 0 {
        return Err(
            "a retention of 0 days would delete every meeting; ask for at least 1 day".to_string(),
        );
    }
    Ok(now_ms.saturating_sub(days.saturating_mul(DAY_MS)))
}

pub fn stale(meetings: &[Meeting], cutoff_ms: u64) -> Vec<String> {
    meetings
        .iter()
        .filter(|meeting| meeting.started_at < cutoff_ms)
        .map(|meeting| meeting.id.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meeting(id: &str, started_at: u64) -> Meeting {
        Meeting {
            id: id.to_string(),
            title: "Sync".to_string(),
            started_at,
            speaker_hint: None,
            segments: Vec::new(),
        }
    }

    #[test]
    fn the_cutoff_sits_the_window_behind_now() {
        let now = 100 * DAY_MS;
        assert_eq!(cutoff_ms(30, now), Ok(70 * DAY_MS));
    }

    #[test]
    fn a_window_of_no_days_is_refused() {
        assert!(cutoff_ms(0, 100 * DAY_MS).is_err());
    }

    #[test]
    fn only_meetings_older_than_the_window_are_stale() {
        let meetings = vec![
            meeting("old", 10 * DAY_MS),
            meeting("edge", 70 * DAY_MS),
            meeting("recent", 99 * DAY_MS),
        ];
        assert_eq!(stale(&meetings, 70 * DAY_MS), vec!["old".to_string()]);
    }

    #[test]
    fn an_empty_folder_has_nothing_to_delete() {
        assert!(stale(&[], 70 * DAY_MS).is_empty());
    }
}
