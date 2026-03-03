use std::fs;
use std::path::PathBuf;

#[path = "../src/data/measures.rs"]
mod measures;

use measures::{
    BoilerplateUnifiedJsonAdapter, MeasureAdapter, MeasuresDbJsonAdapter, VariableMetadataTsvAdapter,
};

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("measures")
        .join(name)
}

#[test]
fn measures_db_writer_outputs_stable_sorted_keys() {
    let raw = fs::read_to_string(fixture_path("measures_db.sample.json"))
        .expect("failed to read measures_db fixture");

    let adapter = MeasuresDbJsonAdapter;
    let records = adapter
        .read_records(&raw)
        .expect("measures db adapter should parse fixture");
    let written = adapter
        .write_records(&records)
        .expect("measures db writer should succeed");

    let social_index = written
        .find("\"social_cohesion\"")
        .expect("social_cohesion key should exist");
    let trust_index = written
        .find("\"trust_science\"")
        .expect("trust_science key should exist");
    assert!(social_index < trust_index, "keys should be sorted deterministically");
    assert!(written.contains("\"extra_field\""));
}

#[test]
fn variable_metadata_tsv_writer_emits_header_and_rows() {
    let raw = fs::read_to_string(fixture_path("variable_metadata.sample.tsv"))
        .expect("failed to read variable metadata tsv fixture");

    let adapter = VariableMetadataTsvAdapter;
    let records = adapter
        .read_records(&raw)
        .expect("tsv adapter should parse fixture");
    let written = adapter
        .write_records(&records)
        .expect("tsv writer should succeed");

    let mut lines = written.lines();
    assert_eq!(lines.next(), Some("variable\tdescription"));
    assert_eq!(
        lines.next(),
        Some("daily_stress\tdaily stress rating (higher means more stress).")
    );
}

#[test]
fn boilerplate_writer_wraps_records_under_measures() {
    let raw = fs::read_to_string(fixture_path("boilerplate_unified.sample.json"))
        .expect("failed to read boilerplate fixture");

    let adapter = BoilerplateUnifiedJsonAdapter;
    let records = adapter
        .read_records(&raw)
        .expect("boilerplate adapter should parse fixture");
    let written = adapter
        .write_records(&records)
        .expect("boilerplate writer should succeed");

    assert!(written.contains("\"measures\""));
    assert!(written.contains("\"trust_science\""));
    assert!(written.contains("\"social_cohesion\""));
}
