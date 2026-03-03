use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

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

    fn write_records(&self, records: &[MeasureRecord]) -> Result<String> {
        write_boilerplate_unified_records(records)
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

    fn write_records(&self, records: &[MeasureRecord]) -> Result<String> {
        write_measures_db_records(records)
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

    fn write_records(&self, records: &[MeasureRecord]) -> Result<String> {
        write_variable_metadata_records(records, '\t')
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

    fn write_records(&self, records: &[MeasureRecord]) -> Result<String> {
        write_variable_metadata_records(records, ',')
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

pub fn infer_measure_file_format(path: &Path) -> MeasureFileFormat {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .unwrap_or_default();

    if file_name == "boilerplate_unified.json" {
        return MeasureFileFormat::BoilerplateUnifiedJson;
    }
    if file_name == "measures_db.json" {
        return MeasureFileFormat::MeasuresDbJson;
    }
    if file_name == "variable_metadata.tsv" || file_name == "variables.tsv" {
        return MeasureFileFormat::VariableMetadataTsv;
    }
    if file_name == "variable_metadata.csv" || file_name == "variables.csv" {
        return MeasureFileFormat::VariableMetadataCsv;
    }
    if file_name.ends_with(".tsv") {
        return MeasureFileFormat::VariableMetadataTsv;
    }
    if file_name.ends_with(".csv") {
        return MeasureFileFormat::MeasuresDbCsv;
    }
    if file_name.ends_with(".json") {
        return MeasureFileFormat::MeasuresDbJson;
    }

    MeasureFileFormat::Unknown
}

pub fn load_measure_records_from_path(path: &Path) -> Result<(MeasureSourceInfo, Vec<MeasureRecord>)> {
    let raw = fs::read_to_string(path)?;
    let format = infer_measure_file_format(path);
    let records = parse_records_by_format(&raw, format)?;
    Ok((MeasureSourceInfo::new(path.to_path_buf(), format), records))
}

pub fn new_measure_session_from_source(source: MeasureSourceInfo, records: Vec<MeasureRecord>) -> MeasureSessionState {
    MeasureSessionState {
        source: Some(source),
        records,
        dirty: false,
        checkpoints: Vec::new(),
    }
}

pub fn render_measure_records_for_path(path: &Path, records: &[MeasureRecord]) -> Result<String> {
    let format = infer_measure_file_format(path);
    match format {
        MeasureFileFormat::BoilerplateUnifiedJson => write_boilerplate_unified_records(records),
        MeasureFileFormat::MeasuresDbJson => write_measures_db_records(records),
        MeasureFileFormat::MeasuresDbCsv => write_variable_metadata_records(records, ','),
        MeasureFileFormat::VariableMetadataTsv => write_variable_metadata_records(records, '\t'),
        MeasureFileFormat::VariableMetadataCsv => write_variable_metadata_records(records, ','),
        MeasureFileFormat::Unknown => write_measures_db_records(records),
    }
}

pub fn save_measure_records_to_path(
    path: &Path,
    records: &[MeasureRecord],
    create_backup: bool,
) -> Result<MeasureSourceInfo> {
    let rendered = render_measure_records_for_path(path, records)?;
    write_text_atomically(path, &rendered, create_backup)?;
    let format = infer_measure_file_format(path);
    Ok(MeasureSourceInfo::new(path.to_path_buf(), format))
}

fn parse_records_by_format(content: &str, format: MeasureFileFormat) -> Result<Vec<MeasureRecord>> {
    match format {
        MeasureFileFormat::BoilerplateUnifiedJson => BoilerplateUnifiedJsonAdapter.read_records(content),
        MeasureFileFormat::MeasuresDbJson => MeasuresDbJsonAdapter.read_records(content),
        MeasureFileFormat::MeasuresDbCsv => parse_variable_metadata_records(content, ','),
        MeasureFileFormat::VariableMetadataTsv => VariableMetadataTsvAdapter.read_records(content),
        MeasureFileFormat::VariableMetadataCsv => VariableMetadataCsvAdapter.read_records(content),
        MeasureFileFormat::Unknown => {
            if let Ok(records) = BoilerplateUnifiedJsonAdapter.read_records(content) {
                return Ok(records);
            }
            if let Ok(records) = MeasuresDbJsonAdapter.read_records(content) {
                return Ok(records);
            }
            if let Ok(records) = VariableMetadataTsvAdapter.read_records(content) {
                return Ok(records);
            }
            if let Ok(records) = VariableMetadataCsvAdapter.read_records(content) {
                return Ok(records);
            }
            Err(anyhow!("unable to detect supported measure file format"))
        }
    }
}

pub fn write_boilerplate_unified_records(records: &[MeasureRecord]) -> Result<String> {
    let mut measures = Map::new();
    let mut sorted = records.to_vec();
    sorted.sort_by(|a, b| a.name.cmp(&b.name));

    for record in sorted {
        if record.name.trim().is_empty() {
            continue;
        }
        measures.insert(record.name.clone(), measure_record_to_json_object(&record));
    }

    let mut root = Map::new();
    root.insert("measures".to_string(), Value::Object(measures));
    serde_json::to_string_pretty(&Value::Object(root)).map_err(Into::into)
}

pub fn write_measures_db_records(records: &[MeasureRecord]) -> Result<String> {
    let mut root = Map::new();
    let mut sorted = records.to_vec();
    sorted.sort_by(|a, b| a.name.cmp(&b.name));

    for record in sorted {
        if record.name.trim().is_empty() {
            continue;
        }
        root.insert(record.name.clone(), measure_record_to_json_object(&record));
    }

    serde_json::to_string_pretty(&Value::Object(root)).map_err(Into::into)
}

pub fn write_variable_metadata_records(records: &[MeasureRecord], delimiter: char) -> Result<String> {
    let mut sorted = records.to_vec();
    sorted.sort_by(|a, b| a.name.cmp(&b.name));

    let sep = delimiter.to_string();
    let mut lines = vec![format!("variable{sep}description")];

    for record in sorted {
        if record.name.trim().is_empty() {
            continue;
        }
        let Some(description) = record.description.as_ref() else {
            continue;
        };
        let escaped_name = escape_delimited_value(&record.name, delimiter);
        let escaped_description = escape_delimited_value(description, delimiter);
        lines.push(format!("{escaped_name}{sep}{escaped_description}"));
    }

    Ok(lines.join("\n") + "\n")
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

fn measure_record_to_json_object(record: &MeasureRecord) -> Value {
    let mut obj = Map::new();

    if let Some(value) = record.label.as_ref() {
        if !value.trim().is_empty() {
            obj.insert("name".to_string(), Value::String(value.trim().to_string()));
        }
    }
    if let Some(value) = record.description.as_ref() {
        if !value.trim().is_empty() {
            obj.insert(
                "description".to_string(),
                Value::String(value.trim().to_string()),
            );
        }
    }
    if let Some(value) = record.reference.as_ref() {
        if !value.trim().is_empty() {
            obj.insert("reference".to_string(), Value::String(value.trim().to_string()));
        }
    }
    if let Some(value) = record.waves.as_ref() {
        if !value.trim().is_empty() {
            obj.insert("waves".to_string(), Value::String(value.trim().to_string()));
        }
    }
    if let Some(value) = record.keywords.as_ref() {
        if !value.trim().is_empty() {
            obj.insert("keywords".to_string(), Value::String(value.trim().to_string()));
        }
    }
    if !record.items.is_empty() {
        let items = record
            .items
            .iter()
            .map(|item| Value::String(item.trim().to_string()))
            .collect();
        obj.insert("items".to_string(), Value::Array(items));
    }
    if let Some(value) = record.standardised {
        obj.insert("standardised".to_string(), Value::Bool(value));
    }
    if let Some(value) = record.standardised_date.as_ref() {
        if !value.trim().is_empty() {
            obj.insert(
                "standardised_date".to_string(),
                Value::String(value.trim().to_string()),
            );
        }
    }
    if let Some(value) = record.scale.as_ref() {
        if !value.trim().is_empty() {
            obj.insert("scale".to_string(), Value::String(value.trim().to_string()));
        }
    }
    if let Some(value) = record.notes.as_ref() {
        if !value.trim().is_empty() {
            obj.insert("notes".to_string(), Value::String(value.trim().to_string()));
        }
    }

    for (key, value) in &record.passthrough {
        if !obj.contains_key(key) {
            obj.insert(key.clone(), value.clone());
        }
    }

    Value::Object(obj)
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

fn escape_delimited_value(value: &str, delimiter: char) -> String {
    let needs_quotes = value.contains(delimiter) || value.contains('"') || value.contains('\n');
    if !needs_quotes {
        return value.trim().to_string();
    }

    let escaped = value.trim().replace('"', "\"\"");
    format!("\"{escaped}\"")
}

fn write_text_atomically(path: &Path, content: &str, create_backup: bool) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    if create_backup && path.exists() {
        let backup_path = backup_path_for(path);
        fs::copy(path, backup_path)?;
    }

    let tmp_path = temporary_path_for(path);
    fs::write(&tmp_path, content)?;
    fs::rename(&tmp_path, path)?;
    Ok(())
}

fn backup_path_for(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("measures");
    let stamp = unix_timestamp_string();
    let backup_name = format!("{file_name}.bak.{stamp}");
    path.with_file_name(backup_name)
}

fn temporary_path_for(path: &Path) -> PathBuf {
    let pid = std::process::id();
    let stamp = unix_timestamp_string();
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("measures");
    let tmp_name = format!("{file_name}.tmp.{pid}.{stamp}");
    path.with_file_name(tmp_name)
}

fn unix_timestamp_string() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    now.as_secs().to_string()
}
