use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::Duration;

use arboard::{Clipboard, ImageData};

pub const LEASE_SECONDS: u64 = 10;
const LEASE: Duration = Duration::from_secs(LEASE_SECONDS);

#[derive(Clone)]
struct Snapshot {
    text: Option<String>,
    image: Option<Picture>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Picture {
    width: usize,
    height: usize,
    bytes: Vec<u8>,
}

struct Lease {
    generation: u64,
    snapshot: Snapshot,
}

static NEXT: AtomicU64 = AtomicU64::new(0);

fn state() -> &'static Mutex<Option<Lease>> {
    static STATE: OnceLock<Mutex<Option<Lease>>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(None))
}

trait Backend {
    fn text(&mut self) -> Option<String>;
    fn image(&mut self) -> Option<Picture>;
    fn set_text(&mut self, text: &str) -> Result<(), String>;
    fn set_image(&mut self, image: &Picture) -> Result<(), String>;
}

impl Backend for Clipboard {
    fn text(&mut self) -> Option<String> {
        Clipboard::get_text(self).ok()
    }

    fn image(&mut self) -> Option<Picture> {
        Clipboard::get_image(self).ok().map(|image| Picture {
            width: image.width,
            height: image.height,
            bytes: image.bytes.into_owned(),
        })
    }

    fn set_text(&mut self, text: &str) -> Result<(), String> {
        Clipboard::set_text(self, text).map_err(|error| error.to_string())
    }

    fn set_image(&mut self, image: &Picture) -> Result<(), String> {
        Clipboard::set_image(
            self,
            ImageData {
                width: image.width,
                height: image.height,
                bytes: image.bytes.clone().into(),
            },
        )
        .map_err(|error| error.to_string())
    }
}

pub fn copy(text: &str) -> Result<u64, String> {
    copy_for(text, LEASE)
}

fn set_for_paste(backend: &mut dyn Backend, text: &str) -> Result<(), String> {
    if text.is_empty() {
        return Err("the clipboard cannot take an empty sentence".to_string());
    }
    backend
        .set_text(text)
        .map_err(|error| format!("the system clipboard refused the text: {error}"))
}

pub fn copy_for(text: &str, after: Duration) -> Result<u64, String> {
    let mut stored = state()
        .lock()
        .map_err(|_| "the clipboard lease is unavailable".to_string())?;
    let mut clipboard =
        Clipboard::new().map_err(|error| format!("the system clipboard did not open: {error}"))?;
    let snapshot = match stored.as_ref() {
        Some(active) => active.snapshot.clone(),
        None => capture(&mut clipboard),
    };
    set_for_paste(&mut clipboard, text)?;
    let generation = NEXT.fetch_add(1, Ordering::Relaxed) + 1;
    *stored = Some(Lease {
        generation,
        snapshot: snapshot.clone(),
    });
    drop(stored);

    let expected = text.to_string();
    let _ = thread::Builder::new()
        .name("parlo-clipboard".to_string())
        .spawn(move || restore_after(snapshot, generation, expected, after));
    Ok(generation)
}

pub fn current() -> Result<Option<String>, String> {
    let mut clipboard =
        Clipboard::new().map_err(|error| format!("the system clipboard did not open: {error}"))?;
    Ok(clipboard.text())
}

pub fn keep(text: &str) -> Result<(), String> {
    let text = text.trim();
    if text.is_empty() {
        return Err("the clipboard cannot take an empty sentence".to_string());
    }
    let mut stored = state()
        .lock()
        .map_err(|_| "the clipboard lease is unavailable".to_string())?;
    let mut clipboard =
        Clipboard::new().map_err(|error| format!("the system clipboard did not open: {error}"))?;
    keep_with(&mut clipboard, &mut stored, text)
}

fn keep_with(
    backend: &mut dyn Backend,
    stored: &mut Option<Lease>,
    text: &str,
) -> Result<(), String> {
    set_for_paste(backend, text)?;
    *stored = None;
    Ok(())
}

fn capture(backend: &mut dyn Backend) -> Snapshot {
    Snapshot {
        text: backend.text(),
        image: backend.image(),
    }
}

fn restore(backend: &mut dyn Backend, snapshot: &Snapshot) -> Result<(), String> {
    if let Some(image) = &snapshot.image {
        return backend.set_image(image);
    }
    match &snapshot.text {
        Some(text) => backend.set_text(text),
        None => Ok(()),
    }
}

fn restore_after(snapshot: Snapshot, generation: u64, expected: String, after: Duration) {
    thread::sleep(after);
    let mut lease = match state().lock() {
        Ok(lease) => lease,
        Err(error) => error.into_inner(),
    };
    if lease.as_ref().map(|active| active.generation) != Some(generation) {
        return;
    }
    let mut clipboard = match Clipboard::new() {
        Ok(clipboard) => clipboard,
        Err(_) => return,
    };
    let _ = restore_if_current(&mut clipboard, &mut lease, &snapshot, generation, &expected);
}

fn restore_if_current(
    backend: &mut dyn Backend,
    lease: &mut Option<Lease>,
    snapshot: &Snapshot,
    generation: u64,
    expected: &str,
) -> Result<(), String> {
    if lease.as_ref().map(|active| active.generation) != Some(generation) {
        return Ok(());
    }
    if backend.text().as_deref() != Some(expected) {
        *lease = None;
        return Ok(());
    }
    *lease = None;
    restore(backend, snapshot)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Memory {
        text: Option<String>,
        image: Option<Picture>,
        writes: usize,
    }

    impl Backend for Memory {
        fn text(&mut self) -> Option<String> {
            self.text.clone()
        }

        fn image(&mut self) -> Option<Picture> {
            self.image.clone()
        }

        fn set_text(&mut self, text: &str) -> Result<(), String> {
            self.writes += 1;
            self.text = Some(text.to_string());
            self.image = None;
            Ok(())
        }

        fn set_image(&mut self, image: &Picture) -> Result<(), String> {
            self.writes += 1;
            self.image = Some(image.clone());
            self.text = None;
            Ok(())
        }
    }

    fn picture() -> Picture {
        Picture {
            width: 2,
            height: 2,
            bytes: vec![1, 2, 3, 4],
        }
    }

    #[test]
    fn a_paste_keeps_spacing_and_line_breaks_in_a_replacement_tail() {
        let mut board = Memory::default();
        for text in [" next word", " ", "\nsecond line", "• "] {
            set_for_paste(&mut board, text).unwrap();
            assert_eq!(board.text.as_deref(), Some(text));
        }
        assert!(set_for_paste(&mut board, "").is_err());
    }

    #[test]
    fn permanent_copy_cannot_be_replaced_by_an_older_lease() {
        let mut board = Memory {
            text: Some("temporary".into()),
            ..Memory::default()
        };
        let snapshot = Snapshot {
            text: Some("older".into()),
            image: None,
        };
        let mut lease = Some(Lease {
            generation: 7,
            snapshot: snapshot.clone(),
        });
        keep_with(&mut board, &mut lease, "recognized text").unwrap();
        restore_if_current(&mut board, &mut lease, &snapshot, 7, "temporary").unwrap();
        assert_eq!(board.text.as_deref(), Some("recognized text"));
        assert_eq!(board.writes, 1);
    }

    #[test]
    fn an_image_comes_back_after_a_copy() {
        let mut board = Memory {
            text: None,
            image: Some(picture()),
            writes: 0,
        };
        let snapshot = capture(&mut board);
        board.text = Some("translated".to_string());
        board.image = None;

        restore(&mut board, &snapshot).unwrap();

        assert_eq!(board.image, Some(picture()));
        assert_eq!(board.text, None);
    }

    #[test]
    fn a_format_we_cannot_read_is_left_alone() {
        let mut files = Memory::default();
        let snapshot = capture(&mut files);
        files.text = Some("translated".to_string());

        restore(&mut files, &snapshot).unwrap();

        assert_eq!(files.writes, 0);
        assert_eq!(files.text.as_deref(), Some("translated"));
    }
}
