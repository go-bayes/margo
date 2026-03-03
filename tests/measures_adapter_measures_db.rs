use std::fs;
use std::path::PathBuf;

#[path = "../src/data/measures.rs"]
mod measures;

use measures::{MeasureAdapter, MeasuresDbJsonAdapter};

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("measures")
        .join(name)
}

#[test]
fn measures_db_adapter_maps_records_and_passthrough() {
    let raw = fs::read_to_string(fixture_path("measures_db.sample.json"))
        .expect("failed to read measures_db fixture");

    let adapter = MeasuresDbJsonAdapter;
    let records = adapter
        .read_records(&raw)
        .expect("measures db adapter should parse fixture");

    assert_eq!(records.len(), 2);

    let trust = records
        .iter()
        .find(|record| record.name == "trust_science")
        .expect("trust_science should be present");
    assert_eq!(
        trust.description.as_deref(),
        Some("trust in science was measured with a short validated scale.")
    );
    assert_eq!(trust.standardised, Some(true));
    assert_eq!(trust.label.as_deref(), Some("Trust in science"));

    let social = records
        .iter()
        .find(|record| record.name == "social_cohesion")
        .expect("social_cohesion should be present");
    assert_eq!(
        social
            .passthrough
            .get("extra_field")
            .and_then(|value| value.get("origin"))
            .and_then(|value| value.as_str()),
        Some("fixture")
    );
}

#[test]
fn measures_db_adapter_sorts_records_by_name() {
    let raw = fs::read_to_string(fixture_path("measures_db.sample.json"))
        .expect("failed to read measures_db fixture");

    let adapter = MeasuresDbJsonAdapter;
    let records = adapter
        .read_records(&raw)
        .expect("measures db adapter should parse fixture");

    let names: Vec<&str> = records.iter().map(|record| record.name.as_str()).collect();
    assert_eq!(names, vec!["social_cohesion", "trust_science"]);
}
