use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde_json::Value;

struct VariableMetadataStore {
    descriptions: HashMap<String, String>,
    source: Option<String>,
}

static VARIABLE_METADATA: OnceLock<VariableMetadataStore> = OnceLock::new();

#[derive(Clone, Copy)]
enum MetadataProvider {
    Local,
    Boilerplate,
    Bptui,
}

impl MetadataProvider {
    fn label(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Boilerplate => "boilerplate",
            Self::Bptui => "bptui",
        }
    }
}

pub fn lookup_variable_description(variable: &str) -> Option<String> {
    let store = VARIABLE_METADATA.get_or_init(load_variable_metadata);
    store.descriptions.get(variable).cloned()
}

pub fn variable_metadata_source() -> Option<String> {
    let store = VARIABLE_METADATA.get_or_init(load_variable_metadata);
    store.source.clone()
}

fn load_variable_metadata() -> VariableMetadataStore {
    for provider in [
        MetadataProvider::Local,
        MetadataProvider::Boilerplate,
        MetadataProvider::Bptui,
    ] {
        if let Some(store) = load_metadata_from_provider(provider) {
            return store;
        }
    }

    VariableMetadataStore {
        descriptions: HashMap::new(),
        source: None,
    }
}

fn load_metadata_from_provider(provider: MetadataProvider) -> Option<VariableMetadataStore> {
    for path in metadata_candidates(provider) {
        if !path.exists() {
            continue;
        }

        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };

        let descriptions = parse_metadata_file(&path, &content);
        if !descriptions.is_empty() {
            return Some(VariableMetadataStore {
                descriptions,
                source: Some(format!("{} ({})", path.display(), provider.label())),
            });
        }
    }

    None
}

fn metadata_candidates(provider: MetadataProvider) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    match provider {
        MetadataProvider::Local => {
            if let Ok(path) = std::env::var("MARGO_VAR_METADATA") {
                if !path.trim().is_empty() {
                    push_path(&mut paths, PathBuf::from(path));
                }
            }

            push_path(&mut paths, cwd.join("storage/variable_metadata.tsv"));
            push_path(&mut paths, cwd.join("storage/variable_metadata.csv"));
            push_path(&mut paths, cwd.join("storage/variables.tsv"));
            push_path(&mut paths, cwd.join("storage/variables.csv"));
        }
        MetadataProvider::Boilerplate => {
            push_env_override(
                &mut paths,
                "MARGO_BOILERPLATE_METADATA",
                &["boilerplate_unified.json", "measures_db.json", "measures_db.csv"],
            );

            for base in [
                cwd.clone(),
                cwd.join("storage"),
                cwd.join("data"),
                cwd.join(".."),
                cwd.join("../boilerplate"),
                cwd.join("../boilerplate/data"),
                cwd.join("../templates/boilerplate/data"),
            ] {
                push_path(&mut paths, base.join("boilerplate_unified.json"));
                push_path(&mut paths, base.join("measures_db.json"));
                push_path(&mut paths, base.join("measures_db.csv"));
            }

            push_path(
                &mut paths,
                cwd.join("../boilerplate/inst/extdata/example_measures.csv"),
            );
        }
        MetadataProvider::Bptui => {
            push_env_override(
                &mut paths,
                "MARGO_BPTUI_METADATA",
                &["boilerplate_unified.json", "measures_db.json", "measures_db.csv"],
            );

            for base in [
                cwd.join("../bptui"),
                cwd.join("../bptui/storage"),
                cwd.join("storage"),
            ] {
                push_path(&mut paths, base.join("boilerplate_unified.json"));
                push_path(&mut paths, base.join("measures_db.json"));
                push_path(&mut paths, base.join("measures_db.csv"));
            }
        }
    }

    paths
}

fn push_env_override(paths: &mut Vec<PathBuf>, env_var: &str, file_names: &[&str]) {
    let Ok(raw) = std::env::var(env_var) else {
        return;
    };

    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return;
    }

    let base = PathBuf::from(trimmed);
    if base.is_dir() {
        for name in file_names {
            push_path(paths, base.join(name));
        }
    } else {
        push_path(paths, base);
    }
}

fn push_path(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if !paths.contains(&path) {
        paths.push(path);
    }
}

fn parse_metadata_file(path: &Path, content: &str) -> HashMap<String, String> {
    match path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .as_deref()
    {
        Some("json") => parse_measure_json_metadata(content),
        _ => parse_metadata(content),
    }
}

fn parse_metadata(content: &str) -> HashMap<String, String> {
    let mut lines = content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'));

    let Some(first_line) = lines.next() else {
        return HashMap::new();
    };

    let delimiter = if first_line.contains('\t') { '\t' } else { ',' };
    let first_fields = split_delimited_line(first_line, delimiter);
    let mut map = HashMap::new();

    if let Some((name_idx, description_idx)) = detect_header_indexes(&first_fields) {
        for line in lines {
            if let Some((key, value)) = parse_row(line, delimiter, name_idx, description_idx) {
                map.insert(key, value);
            }
        }
        return map;
    }

    if let Some((key, value)) = parse_row_from_fields(&first_fields, 0, 1) {
        if !looks_like_header(&key, &value) {
            map.insert(key, value);
        }
    }

    for line in lines {
        if let Some((key, value)) = parse_row(line, delimiter, 0, 1) {
            map.insert(key, value);
        }
    }

    map
}

fn detect_header_indexes(headers: &[String]) -> Option<(usize, usize)> {
    let lower: Vec<String> = headers
        .iter()
        .map(|header| trim_quotes(header).to_lowercase())
        .collect();

    let name_idx = lower.iter().position(|header| {
        matches!(
            header.as_str(),
            "name" | "variable" | "var" | "variable_name" | "measure" | "measure_name" | "id"
        )
    })?;

    let description_idx = [
        "description",
        "label",
        "question",
        "prompt",
        "meaning",
        "notes",
        "scale",
    ]
    .iter()
    .find_map(|candidate| {
        lower
            .iter()
            .position(|header| header.eq_ignore_ascii_case(candidate))
    })?;

    Some((name_idx, description_idx))
}

fn parse_row(
    line: &str,
    delimiter: char,
    name_idx: usize,
    description_idx: usize,
) -> Option<(String, String)> {
    let fields = split_delimited_line(line, delimiter);
    parse_row_from_fields(&fields, name_idx, description_idx)
}

fn parse_row_from_fields(
    fields: &[String],
    name_idx: usize,
    description_idx: usize,
) -> Option<(String, String)> {
    let raw_name = fields.get(name_idx)?.trim();
    let raw_description = fields.get(description_idx)?.trim();

    let name = trim_quotes(raw_name).to_lowercase();
    let description = trim_quotes(raw_description).to_string();

    if name.is_empty() || description.is_empty() {
        return None;
    }

    Some((name, description))
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

fn trim_quotes(value: &str) -> &str {
    value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
}

fn looks_like_header(name: &str, description: &str) -> bool {
    let name = name.to_lowercase();
    let description = description.to_lowercase();

    let name_header = matches!(
        name.as_str(),
        "name" | "variable" | "var" | "variable_name" | "id"
    );
    let description_header = matches!(
        description.as_str(),
        "description" | "label" | "question" | "prompt" | "meaning"
    );

    name_header && description_header
}

fn parse_measure_json_metadata(content: &str) -> HashMap<String, String> {
    let Ok(value) = serde_json::from_str::<Value>(content) else {
        return HashMap::new();
    };

    let mut descriptions = HashMap::new();
    let root = value
        .as_object()
        .and_then(|obj| obj.get("measures"))
        .unwrap_or(&value);

    extract_measure_descriptions(root, &mut descriptions);
    descriptions
}

fn extract_measure_descriptions(value: &Value, out: &mut HashMap<String, String>) {
    match value {
        Value::Object(obj) => {
            for (key, entry) in obj {
                match entry {
                    Value::Object(measure) => {
                        if let Some((name, description)) = parse_measure_record(Some(key), measure) {
                            out.insert(name, description);
                        } else {
                            extract_measure_descriptions(entry, out);
                        }
                    }
                    Value::Array(_) => extract_measure_descriptions(entry, out),
                    _ => {}
                }
            }
        }
        Value::Array(items) => {
            for entry in items {
                if let Some(measure) = entry.as_object() {
                    if let Some((name, description)) = parse_measure_record(None, measure) {
                        out.insert(name, description);
                    }
                }
            }
        }
        _ => {}
    }
}

fn parse_measure_record(
    key_hint: Option<&str>,
    measure: &serde_json::Map<String, Value>,
) -> Option<(String, String)> {
    if !looks_like_measure_record(measure) {
        return None;
    }

    let name = measure
        .get("name")
        .and_then(Value::as_str)
        .or(key_hint)
        .map(|value| trim_quotes(value).to_lowercase())?;

    if name.is_empty() {
        return None;
    }

    let description = extract_measure_description(measure)?;
    if description.is_empty() {
        return None;
    }

    Some((name, description))
}

fn looks_like_measure_record(measure: &serde_json::Map<String, Value>) -> bool {
    [
        "name",
        "description",
        "label",
        "question",
        "prompt",
        "meaning",
        "notes",
        "scale",
        "reference",
        "waves",
        "keywords",
        "items",
        "standardised",
    ]
    .iter()
    .any(|field| measure.contains_key(*field))
}

fn extract_measure_description(measure: &serde_json::Map<String, Value>) -> Option<String> {
    [
        "description",
        "label",
        "question",
        "prompt",
        "meaning",
        "notes",
        "scale",
    ]
    .iter()
    .find_map(|field| {
        measure
            .get(*field)
            .and_then(Value::as_str)
            .map(trim_quotes)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
    })
}
