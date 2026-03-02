use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::Result;
use serde_json::Value;

pub const CANONICAL_MEASURE_FIELDS: &[&str] = &[
    "name",
    "description",
    "reference",
    "waves",
    "keywords",
    "items",
    "standardised",
    "standardised_date",
    "label",
    "scale",
    "notes",
];

#[derive(Debug, Clone, PartialEq)]
pub struct MeasureRecord {
    pub name: String,
    pub description: Option<String>,
    pub reference: Option<String>,
    pub waves: Option<String>,
    pub keywords: Option<String>,
    pub items: Vec<String>,
    pub standardised: Option<bool>,
    pub standardised_date: Option<String>,
    pub label: Option<String>,
    pub scale: Option<String>,
    pub notes: Option<String>,
    pub passthrough: BTreeMap<String, Value>,
}

impl Default for MeasureRecord {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: None,
            reference: None,
            waves: None,
            keywords: None,
            items: Vec::new(),
            standardised: None,
            standardised_date: None,
            label: None,
            scale: None,
            notes: None,
            passthrough: BTreeMap::new(),
        }
    }
}

impl MeasureRecord {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Self::default()
        }
    }

    pub fn canonical_field_names() -> &'static [&'static str] {
        CANONICAL_MEASURE_FIELDS
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeasureFileFormat {
    BoilerplateUnifiedJson,
    MeasuresDbJson,
    MeasuresDbCsv,
    VariableMetadataTsv,
    VariableMetadataCsv,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeasureSourceInfo {
    pub path: PathBuf,
    pub format: MeasureFileFormat,
}

impl MeasureSourceInfo {
    pub fn new(path: PathBuf, format: MeasureFileFormat) -> Self {
        Self { path, format }
    }
}

pub trait MeasureAdapter {
    fn format(&self) -> MeasureFileFormat;
    fn read_records(&self, content: &str) -> Result<Vec<MeasureRecord>>;
    fn write_records(&self, records: &[MeasureRecord]) -> Result<String>;
}

#[derive(Debug, Clone, Default)]
pub struct MeasureSessionState {
    pub source: Option<MeasureSourceInfo>,
    pub records: Vec<MeasureRecord>,
    pub dirty: bool,
    pub checkpoints: Vec<MeasureCheckpoint>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasureCheckpoint {
    pub label: String,
    pub records: Vec<MeasureRecord>,
}

impl MeasureCheckpoint {
    pub fn new(label: impl Into<String>, records: Vec<MeasureRecord>) -> Self {
        Self {
            label: label.into(),
            records,
        }
    }
}
