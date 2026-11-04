use serde::Serialize;

const THRESHOLD: f32 = 0.72;
const MAX_SPEAKERS: usize = 8;
const HOP_MS: u64 = 750;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Turn {
    pub start: u64,
    pub end: u64,
    pub speaker: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Diarization {
    pub speakers: usize,
    pub turns: Vec<Turn>,
}

pub fn group(
    vectors: &[Vec<f32>],
    starts: &[u64],
    ends: &[u64],
    hint: Option<usize>,
) -> Diarization {
    let limit = hint.unwrap_or(MAX_SPEAKERS).clamp(1, 20);
    let count = vectors.len().min(starts.len()).min(ends.len());
    let mut centroids: Vec<Vec<f32>> = Vec::new();
    let mut lines: Vec<usize> = Vec::new();
    let mut labels: Vec<Option<usize>> = Vec::with_capacity(count);
    for vector in vectors.iter().take(count) {
        let unit = normalised(vector);
        if unit.is_empty() {
            labels.push(None);
            continue;
        }
        let closest = centroids
            .iter()
            .enumerate()
            .map(|(slot, centroid)| (slot, dot(&unit, centroid)))
            .max_by(|left, right| left.1.total_cmp(&right.1));
        if let Some((slot, score)) = closest {
            if score >= THRESHOLD || centroids.len() >= limit {
                labels.push(Some(slot));
                absorb(&mut centroids[slot], &mut lines[slot], &unit);
                continue;
            }
        }
        centroids.push(unit);
        lines.push(1);
        labels.push(Some(centroids.len() - 1));
    }

    let mut turns: Vec<Turn> = Vec::new();
    for (index, label) in labels.iter().enumerate() {
        let Some(label) = label else {
            continue;
        };
        let start = starts[index];
        let end = ends[index];
        if let Some(last) = turns.last_mut() {
            if last.speaker == label + 1 && start <= last.end.saturating_add(HOP_MS) {
                last.end = last.end.max(end);
                continue;
            }
        }
        turns.push(Turn {
            start,
            end,
            speaker: label + 1,
        });
    }
    Diarization {
        speakers: centroids.len(),
        turns,
    }
}

fn absorb(centroid: &mut Vec<f32>, lines: &mut usize, unit: &[f32]) {
    let weight = (*lines).min(8) as f32;
    for (slot, value) in centroid.iter_mut().zip(unit) {
        *slot = (*slot * weight + value) / (weight + 1.0);
    }
    *centroid = normalised(centroid);
    *lines += 1;
}

fn normalised(values: &[f32]) -> Vec<f32> {
    if values.iter().any(|value| !value.is_finite()) {
        return Vec::new();
    }
    let length = values.iter().map(|value| value * value).sum::<f32>().sqrt();
    if length <= f32::EPSILON {
        return Vec::new();
    }
    values.iter().map(|value| value / length).collect()
}

fn dot(left: &[f32], right: &[f32]) -> f32 {
    left.iter().zip(right).map(|(one, other)| one * other).sum()
}
