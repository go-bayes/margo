use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use serde_json::{Map, Value};

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

#[derive(Debug, Clone, Copy, Default)]
pub struct BoilerplateUnifiedJsonAdapter;

impl MeasureAdapter for BoilerplateUnifiedJsonAdapter {
    fn format(&self) -> MeasureFileFormat {
        MeasureFileFormat::BoilerplateUnifiedJson
    }

    fn read_records(&self, content: &str) -> Result<Vec<MeasureRecord>> {
        parse_boilerplate_unified_records(content)
    }

    fn write_records(&self, _records: &[MeasureRecord]) -> Result<String> {
        Err(anyhow!(
            "boilerplate unified json writer is not implemented yet"
        ))
    }
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

pub fn parse_boilerplate_unified_records(content: &str) -> Result<Vec<MeasureRecord>> {
    let parsed: Value = serde_json::from_str(content)?;
    let root = parsed
        .as_object()
        .ok_or_else(|| anyhow!("expected top-level json object"))?;

    let measures_value = root
        .get("measures")
        .ok_or_else(|| anyhow!("expected 'measures' key in unified json"))?;
    let measures_obj = measures_value
        .as_object()
        .ok_or_else(|| anyhow!("expected 'measures' to be a json object"))?;

    let mut records = Vec::new();

    for (key, value) in measures_obj {
        let mut record = MeasureRecord::new(key.trim().to_string());

        if let Some(obj) = value.as_object() {
            record.description = read_opt_string(obj, "description");
            record.reference = read_opt_string(obj, "reference");
            record.waves = read_opt_string(obj, "waves");
            record.keywords = read_opt_string(obj, "keywords");
            record.items = read_items(obj.get("items"));
            record.standardised = read_opt_bool(obj, "standardised");
            record.standardised_date = read_opt_string(obj, "standardised_date");
            record.label =
                read_opt_string(obj, "label").or_else(|| read_opt_string(obj, "name"));
            record.scale = read_opt_string(obj, "scale");
            record.notes = read_opt_string(obj, "notes");
            record.passthrough = collect_passthrough_fields(obj);
        } else if let Some(text) = value.as_str() {
            let text = text.trim();
            if !text.is_empty() {
                record.description = Some(text.to_string());
            }
        }

        if !record.name.is_empty() {
            records.push(record);
        }
    }

    records.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(records)
}

fn read_opt_string(obj: &Map<String, Value>, field: &str) -> Option<String> {
    obj.get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn read_opt_bool(obj: &Map<String, Value>, field: &str) -> Option<bool> {
    obj.get(field).and_then(Value::as_bool)
}

fn read_items(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(Value::as_str)
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(ToString::to_string)
            .collect(),
        Some(Value::String(raw)) => raw
            .split('|')
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(ToString::to_string)
            .collect(),
        _ => Vec::new(),
    }
}

fn collect_passthrough_fields(obj: &Map<String, Value>) -> BTreeMap<String, Value> {
    let mut passthrough = BTreeMap::new();
    for (key, value) in obj {
        if !is_known_source_field(key) {
            passthrough.insert(key.clone(), value.clone());
        }
    }
    passthrough
}

fn is_known_source_field(field: &str) -> bool {
    matches!(
        field,
        "name"
            | "description"
            | "reference"
            | "waves"
            | "keywords"
            | "items"
            | "standardised"
            | "standardised_date"
            | "label"
            | "scale"
            | "notes"
    )
}
