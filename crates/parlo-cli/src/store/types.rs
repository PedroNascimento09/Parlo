use std::collections::BTreeMap;

use serde::Serialize;

use super::now;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Imported {
    pub history: usize,
    pub meetings: usize,
    pub settings: usize,
    pub already: bool,
}

pub struct StoredJob {
    pub id: String,
    pub kind: String,
    pub stage: String,
    pub progress: f32,
    pub input_hash: String,
    pub title: String,
    pub created_at: i64,
}

pub struct JobRecord<'a> {
    pub id: &'a str,
    pub kind: &'a str,
    pub state: &'a str,
    pub stage: &'a str,
    pub progress: f32,
    pub input_hash: &'a str,
    pub title: &'a str,
    pub error: Option<&'a str>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DubProject {
    pub id: String,
    pub title: String,
    pub source_path: Option<String>,
    pub from: String,
    pub to: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub targets: Vec<String>,
    pub cast: BTreeMap<String, String>,
    pub segments: Vec<DubSegment>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DubSummary {
    pub id: String,
    pub title: String,
    pub source_path: Option<String>,
    pub from: String,
    pub to: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub segments: usize,
    pub invalid: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DubSegment {
    pub id: String,
    pub position: usize,
    pub start_ms: u64,
    pub end_ms: u64,
    pub speaker: Option<String>,
    pub voice_id: Option<String>,
    pub translations: BTreeMap<String, String>,
    pub source_text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_text: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub revisions: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pace: Option<f64>,
    pub target_text: Option<String>,
    pub state: String,
}

impl DubProject {
    pub fn new(
        id: String,
        title: String,
        from: &str,
        to: &str,
        source_path: Option<String>,
    ) -> Self {
        let stamp = now();
        Self {
            id,
            title,
            source_path,
            from: from.to_string(),
            to: to.to_string(),
            created_at: stamp,
            updated_at: stamp,
            targets: vec![to.to_string()],
            cast: BTreeMap::new(),
            segments: Vec::new(),
        }
    }
}
