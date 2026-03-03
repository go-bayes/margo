use std::fs;
use std::path::{Path, PathBuf};

#[path = "../src/data/measures.rs"]
mod measures;

use measures::{
    infer_measure_file_format, load_measure_records_from_path, new_measure_session_from_source,
    MeasureFileFormat,
};

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("measures")
        .join(name)
}

#[test]
fn infer_measure_file_format_matches_expected_names() {
    assert_eq!(
        infer_measure_file_format(Path::new("/tmp/boilerplate_unified.json")),
        MeasureFileFormat::BoilerplateUnifiedJson
    );
    assert_eq!(
        infer_measure_file_format(Path::new("/tmp/measures_db.json")),
        MeasureFileFormat::MeasuresDbJson
    );
    assert_eq!(
        infer_measure_file_format(Path::new("/tmp/variable_metadata.tsv")),
        MeasureFileFormat::VariableMetadataTsv
    );
    assert_eq!(
        infer_measure_file_format(Path::new("/tmp/variable_metadata.csv")),
        MeasureFileFormat::VariableMetadataCsv
    );
}

#[test]
fn load_measure_records_from_path_reads_fixture_with_source_info() {
    let fixture = fixture_path("boilerplate_unified.sample.json");
    let (source, records) = load_measure_records_from_path(&fixture)
        .expect("loading measure fixture from path should succeed");

    assert_eq!(source.path, fixture);
    assert_eq!(source.format, MeasureFileFormat::BoilerplateUnifiedJson);
    assert_eq!(records.len(), 2);
}

#[test]
fn new_measure_session_from_source_starts_clean() {
    let fixture = fixture_path("boilerplate_unified.sample.json");
    let (source, records) = load_measure_records_from_path(&fixture)
        .expect("loading measure fixture from path should succeed");

    let session = new_measure_session_from_source(source, records);
    assert!(!session.dirty);
    assert_eq!(session.checkpoints.len(), 0);
    assert!(session.source.is_some());
    assert_eq!(session.records.len(), 2);
}

#[test]
fn unknown_format_falls_back_to_content_detection() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("custom_measures_input.dat");
    let content = fs::read_to_string(fixture_path("measures_db.sample.json"))
        .expect("failed to read measures_db fixture");
    fs::write(&path, content).expect("failed to write temp input");

    let (source, records) =
        load_measure_records_from_path(&path).expect("content fallback parser should succeed");
    assert_eq!(source.format, MeasureFileFormat::Unknown);
    assert_eq!(records.len(), 2);
}
