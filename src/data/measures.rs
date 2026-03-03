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

#[derive(Debug, Clone, Copy, Default)]
pub struct MeasuresDbJsonAdapter;

impl MeasureAdapter for MeasuresDbJsonAdapter {
    fn format(&self) -> MeasureFileFormat {
        MeasureFileFormat::MeasuresDbJson
    }

    fn read_records(&self, content: &str) -> Result<Vec<MeasureRecord>> {
        parse_measures_db_records(content)
    }

    fn write_records(&self, _records: &[MeasureRecord]) -> Result<String> {
        Err(anyhow!("measures db json writer is not implemented yet"))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct VariableMetadataTsvAdapter;

impl MeasureAdapter for VariableMetadataTsvAdapter {
    fn format(&self) -> MeasureFileFormat {
        MeasureFileFormat::VariableMetadataTsv
    }

    fn read_records(&self, content: &str) -> Result<Vec<MeasureRecord>> {
        parse_variable_metadata_records(content, '\t')
    }

    fn write_records(&self, _records: &[MeasureRecord]) -> Result<String> {
        Err(anyhow!(
            "variable metadata tsv writer is not implemented yet"
        ))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct VariableMetadataCsvAdapter;

impl MeasureAdapter for VariableMetadataCsvAdapter {
    fn format(&self) -> MeasureFileFormat {
        MeasureFileFormat::VariableMetadataCsv
    }

    fn read_records(&self, content: &str) -> Result<Vec<MeasureRecord>> {
        parse_variable_metadata_records(content, ',')
    }

    fn write_records(&self, _records: &[MeasureRecord]) -> Result<String> {
        Err(anyhow!(
            "variable metadata csv writer is not implemented yet"
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
        let record = parse_measure_entry(key, value);

        if !record.name.is_empty() {
            records.push(record);
        }
    }

    records.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(records)
}

pub fn parse_measures_db_records(content: &str) -> Result<Vec<MeasureRecord>> {
    let parsed: Value = serde_json::from_str(content)?;
    let root = parsed
        .as_object()
        .ok_or_else(|| anyhow!("expected top-level json object for measures db"))?;

    let mut records = Vec::new();

    for (key, value) in root {
        let record = parse_measure_entry(key, value);
        if !record.name.is_empty() {
            records.push(record);
        }
    }

    records.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(records)
}

pub fn parse_variable_metadata_records(content: &str, delimiter: char) -> Result<Vec<MeasureRecord>> {
    let mut lines = content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'));

    let first_line = lines
        .next()
        .ok_or_else(|| anyhow!("metadata table is empty"))?;

    let first_fields = split_delimited_line(first_line, delimiter);
    let (name_idx, description_idx, has_header) =
        detect_name_description_columns(&first_fields).unwrap_or((0, 1, false));

    let mut records = Vec::new();

    if !has_header {
        if let Some(record) = parse_variable_metadata_row(&first_fields, name_idx, description_idx) {
            records.push(record);
        }
    }

    for line in lines {
        let fields = split_delimited_line(line, delimiter);
        if let Some(record) = parse_variable_metadata_row(&fields, name_idx, description_idx) {
            records.push(record);
        }
    }

    records.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(records)
}

fn parse_measure_entry(key: &str, value: &Value) -> MeasureRecord {
    let mut record = MeasureRecord::new(key.trim().to_string());

    if let Some(obj) = value.as_object() {
        record.description = read_opt_string(obj, "description");
        record.reference = read_opt_string(obj, "reference");
        record.waves = read_opt_string(obj, "waves");
        record.keywords = read_opt_string(obj, "keywords");
        record.items = read_items(obj.get("items"));
        record.standardised = read_opt_bool(obj, "standardised");
        record.standardised_date = read_opt_string(obj, "standardised_date");
        record.label = read_opt_string(obj, "label").or_else(|| read_opt_string(obj, "name"));
        record.scale = read_opt_string(obj, "scale");
        record.notes = read_opt_string(obj, "notes");
        record.passthrough = collect_passthrough_fields(obj);
    } else if let Some(text) = value.as_str() {
        let text = text.trim();
        if !text.is_empty() {
            record.description = Some(text.to_string());
        }
    }

    record
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

fn detect_name_description_columns(fields: &[String]) -> Option<(usize, usize, bool)> {
    let lower: Vec<String> = fields
        .iter()
        .map(|value| value.trim().trim_matches('"').to_lowercase())
        .collect();

    let name_idx = lower.iter().position(|value| {
        matches!(
            value.as_str(),
            "name" | "measure" | "variable" | "var" | "measure_name" | "variable_name" | "id"
        )
    })?;

    let description_idx = lower.iter().position(|value| {
        matches!(
            value.as_str(),
            "description" | "label" | "question" | "prompt" | "meaning" | "notes"
        )
    })?;

    Some((name_idx, description_idx, true))
}

fn parse_variable_metadata_row(
    fields: &[String],
    name_idx: usize,
    description_idx: usize,
) -> Option<MeasureRecord> {
    let name = fields.get(name_idx)?.trim().trim_matches('"').to_string();
    let description = fields
        .get(description_idx)?
        .trim()
        .trim_matches('"')
        .to_string();

    if name.is_empty() || description.is_empty() {
        return None;
    }

    let mut record = MeasureRecord::new(name);
    record.description = Some(description);
    Some(record)
}

fn split_delimited_line(line: &str, delimiter: char) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                if in_quotes && matches!(chars.peek(), Some('"')) {
                    current.push('"');
                    let _ = chars.next();
                } else {
                    in_quotes = !in_quotes;
                }
            }
            value if value == delimiter && !in_quotes => {
                fields.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    fields.push(current.trim().to_string());
    fields
}
