use std::collections::HashMap;
use std::path::Path;

use anyhow::{anyhow, Result};

use super::measures::{
    load_measure_records_from_path, new_measure_session_from_source, MeasureRecord,
    MeasureSessionState, MeasureSourceInfo,
};

#[derive(Debug, Clone, Default)]
pub struct MeasureWorkspace {
    pub session: MeasureSessionState,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MeasureValidationReport {
    pub duplicate_names: Vec<String>,
    pub missing_description: Vec<String>,
}

impl MeasureWorkspace {
    pub fn load(path: &Path) -> Result<Self> {
        let (source, records) = load_measure_records_from_path(path)?;
        Ok(Self::from_source(source, records))
    }

    pub fn from_source(source: MeasureSourceInfo, records: Vec<MeasureRecord>) -> Self {
        Self {
            session: new_measure_session_from_source(source, records),
        }
    }

    pub fn source(&self) -> Option<&MeasureSourceInfo> {
        self.session.source.as_ref()
    }

    pub fn is_dirty(&self) -> bool {
        self.session.dirty
    }

    pub fn record_count(&self) -> usize {
        self.session.records.len()
    }

    pub fn list<'a>(&'a self, pattern: Option<&str>) -> Vec<&'a MeasureRecord> {
        let query = pattern.unwrap_or("").trim().to_lowercase();
        let mut out: Vec<&MeasureRecord> = self
            .session
            .records
            .iter()
            .filter(|record| {
                if query.is_empty() {
                    return true;
                }
                record.name.to_lowercase().contains(&query)
                    || record
                        .description
                        .as_ref()
                        .is_some_and(|value| value.to_lowercase().contains(&query))
            })
            .collect();
        out.sort_by(|a, b| a.name.cmp(&b.name));
        out
    }

    pub fn get(&self, name: &str) -> Option<&MeasureRecord> {
        self.session.records.iter().find(|record| record.name == name)
    }

    pub fn add(&mut self, name: &str) -> Result<()> {
        let clean = name.trim();
        if clean.is_empty() {
            return Err(anyhow!("measure name cannot be empty"));
        }
        if self.get(clean).is_some() {
            return Err(anyhow!("measure already exists: {clean}"));
        }
        self.session.records.push(MeasureRecord::new(clean.to_string()));
        self.session.records.sort_by(|a, b| a.name.cmp(&b.name));
        self.session.dirty = true;
        Ok(())
    }

    pub fn delete(&mut self, name: &str) -> bool {
        let before = self.session.records.len();
        self.session.records.retain(|record| record.name != name);
        let changed = self.session.records.len() != before;
        if changed {
            self.session.dirty = true;
        }
        changed
    }

    pub fn rename(&mut self, old_name: &str, new_name: &str) -> Result<()> {
        let clean_new = new_name.trim();
        if clean_new.is_empty() {
            return Err(anyhow!("new measure name cannot be empty"));
        }
        if self.get(clean_new).is_some() {
            return Err(anyhow!("measure already exists: {clean_new}"));
        }

        let Some(record) = self.session.records.iter_mut().find(|record| record.name == old_name) else {
            return Err(anyhow!("measure not found: {old_name}"));
        };
        record.name = clean_new.to_string();
        self.session.records.sort_by(|a, b| a.name.cmp(&b.name));
        self.session.dirty = true;
        Ok(())
    }

    pub fn edit_field(&mut self, name: &str, field: &str, value: &str) -> Result<()> {
        let clean_field = field.trim().to_lowercase();
        let clean_value = value.trim();
        let Some(record) = self.session.records.iter_mut().find(|record| record.name == name) else {
            return Err(anyhow!("measure not found: {name}"));
        };

        match clean_field.as_str() {
            "description" => {
                record.description = if clean_value.is_empty() {
                    None
                } else {
                    Some(clean_value.to_string())
                };
            }
            "reference" => {
                record.reference = if clean_value.is_empty() {
                    None
                } else {
                    Some(clean_value.to_string())
                };
            }
            "waves" => {
                record.waves = if clean_value.is_empty() {
                    None
                } else {
                    Some(clean_value.to_string())
                };
            }
            "keywords" => {
                record.keywords = if clean_value.is_empty() {
                    None
                } else {
                    Some(clean_value.to_string())
                };
            }
            "label" => {
                record.label = if clean_value.is_empty() {
                    None
                } else {
                    Some(clean_value.to_string())
                };
            }
            "scale" => {
                record.scale = if clean_value.is_empty() {
                    None
                } else {
                    Some(clean_value.to_string())
                };
            }
            "notes" => {
                record.notes = if clean_value.is_empty() {
                    None
                } else {
                    Some(clean_value.to_string())
                };
            }
            "standardised" => {
                record.standardised = if clean_value.is_empty() {
                    None
                } else {
                    Some(parse_bool(clean_value)?)
                };
            }
            "standardised_date" => {
                record.standardised_date = if clean_value.is_empty() {
                    None
                } else {
                    Some(clean_value.to_string())
                };
            }
            "items" => {
                record.items = split_items(clean_value);
            }
            _ => return Err(anyhow!("unknown measure field: {field}")),
        }

        self.session.dirty = true;
        Ok(())
    }

    pub fn export_missing(&self, field: &str) -> Vec<String> {
        let field = field.trim().to_lowercase();
        let mut names: Vec<String> = self
            .session
            .records
            .iter()
            .filter(|record| match field.as_str() {
                "description" => record
                    .description
                    .as_ref()
                    .map(|value| value.trim().is_empty())
                    .unwrap_or(true),
                "reference" => record
                    .reference
                    .as_ref()
                    .map(|value| value.trim().is_empty())
                    .unwrap_or(true),
                "waves" => record
                    .waves
                    .as_ref()
                    .map(|value| value.trim().is_empty())
                    .unwrap_or(true),
                "label" => record
                    .label
                    .as_ref()
                    .map(|value| value.trim().is_empty())
                    .unwrap_or(true),
                "scale" => record
                    .scale
                    .as_ref()
                    .map(|value| value.trim().is_empty())
                    .unwrap_or(true),
                "notes" => record
                    .notes
                    .as_ref()
                    .map(|value| value.trim().is_empty())
                    .unwrap_or(true),
                _ => record
                    .description
                    .as_ref()
                    .map(|value| value.trim().is_empty())
                    .unwrap_or(true),
            })
            .map(|record| record.name.clone())
            .collect();
        names.sort();
        names
    }

    pub fn validate_basic(&self) -> MeasureValidationReport {
        let mut counts: HashMap<String, usize> = HashMap::new();
        let mut missing_description = Vec::new();

        for record in &self.session.records {
            *counts.entry(record.name.clone()).or_insert(0) += 1;
            if record
                .description
                .as_ref()
                .map(|value| value.trim().is_empty())
                .unwrap_or(true)
            {
                missing_description.push(record.name.clone());
            }
        }

        let mut duplicate_names: Vec<String> = counts
            .into_iter()
            .filter_map(|(name, count)| if count > 1 { Some(name) } else { None })
            .collect();
        duplicate_names.sort();
        missing_description.sort();

        MeasureValidationReport {
            duplicate_names,
            missing_description,
        }
    }
}

fn parse_bool(value: &str) -> Result<bool> {
    match value.trim().to_lowercase().as_str() {
        "true" | "1" | "yes" | "y" => Ok(true),
        "false" | "0" | "no" | "n" => Ok(false),
        _ => Err(anyhow!("invalid boolean value: {value}")),
    }
}

fn split_items(value: &str) -> Vec<String> {
    value
        .split(|ch| ch == '|' || ch == ';')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToString::to_string)
        .collect()
}
