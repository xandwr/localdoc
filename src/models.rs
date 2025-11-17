use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub docpack_format: u32,
    pub project: ProjectInfo,
    pub generated_at: String,
    pub language_summary: HashMap<String, u32>,
    pub stats: Stats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    pub version: String,
    pub repo: String,
    pub commit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    pub symbols_extracted: u32,
    pub docs_generated: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub id: String,
    pub kind: String,
    pub file: String,
    pub line: usize,
    pub signature: String,
    pub doc_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Documentation {
    pub symbol: String,
    pub summary: String,
    pub description: String,
    pub parameters: Vec<Parameter>,
    pub returns: String,
    pub example: String,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
    pub description: String,
}
