use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

pub const MAX_AGE_SECONDS: u64 = 300;

#[derive(Debug, Clone, PartialEq)]
pub struct Last {
    pub window: usize,
    pub characters: usize,
    pub at: u64,
}

static LAST: Mutex<Option<Last>> = Mutex::new(None);

pub fn remember(window: usize, text: &str) -> Last {
    let last = Last {
        window,
        characters: text.chars().count(),
        at: now(),
    };
    if let Ok(mut stored) = LAST.lock() {
        *stored = Some(last.clone());
    }
    last
}

pub fn take() -> Result<Last, String> {
    let mut stored = LAST
        .lock()
        .map_err(|_| "the last dictation could not be read".to_string())?;
    let last = stored
        .take()
        .ok_or_else(|| "there is no dictation left to undo".to_string())?;
    if now().saturating_sub(last.at) > MAX_AGE_SECONDS {
        return Err("that dictation is too old to take back".to_string());
    }
    if last.characters == 0 {
        return Err("that dictation wrote nothing".to_string());
    }
    Ok(last)
}

pub fn restore(last: &Last) {
    if let Ok(mut stored) = LAST.lock() {
        *stored = Some(last.clone());
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh() {
        *LAST.lock().unwrap() = None;
    }

    #[test]
    fn a_dictation_is_counted_in_characters() {
        fresh();
        let last = remember(7, "olá, mundo");
        assert_eq!(last.characters, 10);
        assert_eq!(last.window, 7);
        fresh();
    }

    #[test]
    fn taking_it_once_leaves_nothing_behind() {
        fresh();
        remember(1, "once");
        assert!(take().is_ok());
        assert!(take().is_err());
        fresh();
    }

    #[test]
    fn an_old_dictation_is_refused() {
        fresh();
        *LAST.lock().unwrap() = Some(Last {
            window: 1,
            characters: 4,
            at: now() - MAX_AGE_SECONDS - 1,
        });
        let error = take().unwrap_err();
        assert!(error.contains("too old"));
        fresh();
    }

    #[test]
    fn a_refused_undo_keeps_nothing_to_try_again() {
        fresh();
        remember(1, "");
        assert!(take().is_err());
        fresh();
    }

    #[test]
    fn restoring_puts_the_dictation_back() {
        fresh();
        let last = remember(3, "abc");
        let taken = take().unwrap();
        restore(&taken);
        assert_eq!(take().unwrap(), last);
        fresh();
    }
}
