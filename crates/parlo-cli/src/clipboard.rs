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

#[derive(Clone)]
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

fn capture(clipboard: &mut Clipboard) -> Snapshot {
    let text = clipboard.get_text().ok();
    let image = clipboard.get_image().ok().map(|image| Picture {
        width: image.width,
        height: image.height,
        bytes: image.bytes.into_owned(),
    });
    Snapshot { text, image }
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

fn restore(clipboard: &mut Clipboard, snapshot: &Snapshot) -> Result<(), String> {
    if let Some(image) = &snapshot.image {
        return clipboard
            .set_image(ImageData {
                width: image.width,
                height: image.height,
                bytes: image.bytes.clone().into(),
            })
            .map_err(|error| error.to_string());
    }
    if let Some(text) = &snapshot.text {
        return clipboard.set_text(text).map_err(|error| error.to_string());
    }
    clipboard.clear().map_err(|error| error.to_string())
}
