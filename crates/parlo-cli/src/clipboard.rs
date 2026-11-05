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
    let text = text.trim();
    if text.is_empty() {
        return Err("the clipboard cannot take an empty sentence".to_string());
    }
    let mut clipboard =
        Clipboard::new().map_err(|error| format!("the system clipboard did not open: {error}"))?;
    let mut lease = state()
        .lock()
        .map_err(|_| "the clipboard lease is unavailable".to_string())?;
    let snapshot = match lease.as_ref() {
        Some(active) => active.snapshot.clone(),
        None => capture(&mut clipboard),
    };
    clipboard
        .set_text(text)
        .map_err(|error| format!("the system clipboard refused the text: {error}"))?;
    let generation = NEXT.fetch_add(1, Ordering::Relaxed) + 1;
    *lease = Some(Lease {
        generation,
        snapshot: snapshot.clone(),
    });
    drop(lease);

    let expected = text.to_string();
    let _ = thread::Builder::new()
        .name("parlo-clipboard".to_string())
        .spawn(move || restore_after_lease(snapshot, generation, expected));
    Ok(generation)
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

fn restore_after_lease(snapshot: Snapshot, generation: u64, expected: String) {
    thread::sleep(LEASE);
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
    if clipboard.get_text().ok().as_deref() != Some(expected.as_str()) {
        *lease = None;
        return;
    }
    *lease = None;
    drop(lease);
    let _ = restore(&mut clipboard, &snapshot);
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
