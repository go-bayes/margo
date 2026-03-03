use std::fs;
use std::path::PathBuf;

#[path = "../src/data/measures.rs"]
mod measures;

use measures::{
    load_measure_records_from_path, render_measure_records_for_path, save_measure_records_to_path,
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
fn render_measure_records_matches_target_extension() {
    let fixture = fixture_path("measures_db.sample.json");
    let (_, records) = load_measure_records_from_path(&fixture).expect("load fixture");

    let rendered_json = render_measure_records_for_path(PathBuf::from("out/measures_db.json").as_path(), &records)
        .expect("render json");
    let rendered_tsv =
        render_measure_records_for_path(PathBuf::from("out/variable_metadata.tsv").as_path(), &records)
            .expect("render tsv");

    assert!(rendered_json.trim_start().starts_with('{'));
    assert!(rendered_tsv.starts_with("variable\tdescription"));
}

#[test]
fn save_measure_records_writes_file_and_reports_format() {
    let dir = tempfile::tempdir().expect("tempdir");
    let out_path = dir.path().join("measures_db.json");
    let fixture = fixture_path("measures_db.sample.json");
    let (_, records) = load_measure_records_from_path(&fixture).expect("load fixture");

    let source = save_measure_records_to_path(&out_path, &records, false).expect("save should work");
    let saved = fs::read_to_string(&out_path).expect("saved file should exist");

    assert_eq!(source.format, MeasureFileFormat::MeasuresDbJson);
    assert!(saved.contains("\"trust_science\""));
}

#[test]
fn save_measure_records_creates_backup_when_requested() {
    let dir = tempfile::tempdir().expect("tempdir");
    let out_path = dir.path().join("measures_db.json");

    fs::write(&out_path, "{\"old\":true}\n").expect("seed output file");

    let fixture = fixture_path("measures_db.sample.json");
    let (_, records) = load_measure_records_from_path(&fixture).expect("load fixture");
    save_measure_records_to_path(&out_path, &records, true).expect("save should work");

    let entries: Vec<PathBuf> = fs::read_dir(dir.path())
        .expect("read dir")
        .filter_map(|entry| entry.ok().map(|value| value.path()))
        .collect();
    assert!(
        entries
            .iter()
            .any(|path| path.file_name().and_then(|v| v.to_str()).is_some_and(|name| {
                name.starts_with("measures_db.json.bak.")
            })),
        "expected backup file to be created"
    );
}
