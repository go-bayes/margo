use std::fs;
use std::path::PathBuf;

#[path = "../src/data/measures.rs"]
mod measures;

use measures::{MeasureAdapter, VariableMetadataCsvAdapter, VariableMetadataTsvAdapter};

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("measures")
        .join(name)
}

#[test]
fn variable_metadata_tsv_adapter_parses_name_description_rows() {
    let raw = fs::read_to_string(fixture_path("variable_metadata.sample.tsv"))
        .expect("failed to read variable metadata tsv fixture");

    let adapter = VariableMetadataTsvAdapter;
    let records = adapter
        .read_records(&raw)
        .expect("tsv adapter should parse fixture");

    assert_eq!(records.len(), 3);
    let names: Vec<&str> = records.iter().map(|record| record.name.as_str()).collect();
    assert_eq!(names, vec!["daily_stress", "social_cohesion", "trust_science"]);
    assert!(
        records
            .iter()
            .all(|record| record.description.as_ref().is_some_and(|value| !value.is_empty()))
    );
}

#[test]
fn variable_metadata_csv_adapter_parses_equivalent_shape() {
    let csv = "variable,description\ntrust_science,trust in science summary score.\nsocial_cohesion,social cohesion summary score.\n";

    let adapter = VariableMetadataCsvAdapter;
    let records = adapter
        .read_records(csv)
        .expect("csv adapter should parse inline fixture");

    assert_eq!(records.len(), 2);
    assert_eq!(records[0].name, "social_cohesion");
    assert_eq!(records[1].name, "trust_science");
}
