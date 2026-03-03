use std::fs;
use std::path::PathBuf;

#[path = "../src/data/measures.rs"]
mod measures;

use measures::{BoilerplateUnifiedJsonAdapter, MeasureAdapter};

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("measures")
        .join(name)
}

#[test]
fn boilerplate_adapter_maps_canonical_fields_and_passthrough() {
    let raw = fs::read_to_string(fixture_path("boilerplate_unified.sample.json"))
        .expect("failed to read boilerplate_unified fixture");

    let adapter = BoilerplateUnifiedJsonAdapter;
    let records = adapter
        .read_records(&raw)
        .expect("boilerplate adapter should parse fixture");

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
    assert_eq!(trust.scale.as_deref(), Some("1-7"));
    assert_eq!(trust.items.len(), 2);
    assert_eq!(
        trust
            .passthrough
            .get("custom_field")
            .and_then(|value| value.as_str()),
        Some("preserve-me")
    );
}

#[test]
fn boilerplate_adapter_returns_stable_name_order() {
    let raw = fs::read_to_string(fixture_path("boilerplate_unified.sample.json"))
        .expect("failed to read boilerplate_unified fixture");

    let adapter = BoilerplateUnifiedJsonAdapter;
    let records = adapter
        .read_records(&raw)
        .expect("boilerplate adapter should parse fixture");

    let names: Vec<&str> = records.iter().map(|record| record.name.as_str()).collect();
    assert_eq!(names, vec!["social_cohesion", "trust_science"]);
}
