use std::path::PathBuf;

const WAV: &str = "docs/benchmarks/d10-speech.wav";
const REPORT: &str = "docs/benchmarks/d10-speech.json";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn chunk<'a>(bytes: &'a [u8], wanted: &[u8; 4]) -> Option<&'a [u8]> {
    let mut offset = 12;
    while offset + 8 <= bytes.len() {
        let name = &bytes[offset..offset + 4];
        let size = u32::from_le_bytes([
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]) as usize;
        let body = offset + 8;
        if body + size > bytes.len() {
            return None;
        }
        if name == wanted {
            return Some(&bytes[body..body + size]);
        }
        offset = body + size + (size % 2);
    }
    None
}

#[test]
fn the_stored_sentence_is_audible_audio_at_the_engine_rate() {
    let bytes = std::fs::read(root().join(WAV)).expect("the spoken sentence");
    assert_eq!(&bytes[0..4], b"RIFF", "not a RIFF file");
    assert_eq!(&bytes[8..12], b"WAVE", "not a WAVE file");
    let format = chunk(&bytes, b"fmt ").expect("a fmt chunk");
    let encoding = u16::from_le_bytes([format[0], format[1]]);
    let channels = u16::from_le_bytes([format[2], format[3]]);
    let rate = u32::from_le_bytes([format[4], format[5], format[6], format[7]]);
    let bits = u16::from_le_bytes([format[14], format[15]]);
    assert_eq!(encoding, 1, "not uncompressed pcm");
    assert_eq!(channels, 1, "{channels} channels");
    assert_eq!(rate, 16_000, "{rate} Hz");
    assert_eq!(bits, 16, "{bits} bits per sample");

    let data = chunk(&bytes, b"data").expect("a data chunk");
    let samples: Vec<i16> = data
        .chunks_exact(2)
        .map(|pair| i16::from_le_bytes([pair[0], pair[1]]))
        .collect();
    let frames = samples.len();
    let seconds = frames as f64 / f64::from(rate);
    assert!(
        (2.0..=10.0).contains(&seconds),
        "the sentence lasts {seconds:.2} s"
    );
    let rms = (samples
        .iter()
        .map(|value| f64::from(*value) * f64::from(*value))
        .sum::<f64>()
        / frames as f64)
        .sqrt();
    assert!(rms > 300.0, "the audio is near silence: rms {rms:.0}");
    let tail_from = frames.saturating_sub(rate as usize / 50);
    let tail_rms = (samples[tail_from..]
        .iter()
        .map(|value| f64::from(*value) * f64::from(*value))
        .sum::<f64>()
        / (frames - tail_from) as f64)
        .sqrt();
    assert!(
        tail_rms < rms / 3.0,
        "the sentence does not end: rms {rms:.0}, tail {tail_rms:.0}"
    );
}

#[test]
fn the_report_keeps_the_spoken_text_and_what_came_back() {
    let report = std::fs::read_to_string(root().join(REPORT)).expect("the report");
    assert!(report.contains(
        "\"text\": \"Hoje está um dia bonito em Lisboa e apetece passear à beira do rio.\""
    ));
    assert!(report.contains(
        "\"respelled\": \"Hoje está um dia bonito em Lisboa e apetece passear à beira do rio.\""
    ));
    assert!(report.contains("\"voice\": \"voice-pt\""));
    assert!(report.contains("\"sample_rate\": 16000"));
    assert!(
        report.contains("\"transcript\": \"Ojista tem um dia"),
        "{report}"
    );
    assert!(report.contains("Parakeet 0.6B v3"));
}
