use std::fs;
use std::path::PathBuf;

use serde_json::Value;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("measures")
        .join(name)
}

#[test]
fn measures_db_fixture_is_object_of_measure_records() {
    let raw = fs::read_to_string(fixture_path("measures_db.sample.json"))
        .expect("failed to read measures_db fixture");
    let parsed: Value = serde_json::from_str(&raw).expect("measures_db fixture must be valid json");

    let root = parsed
        .as_object()
        .expect("measures_db fixture must be a top-level object");
    assert!(root.contains_key("trust_science"));
    assert!(root.contains_key("social_cohesion"));

    for (key, value) in root {
        let record = value
            .as_object()
            .unwrap_or_else(|| panic!("record '{key}' must be an object"));
        assert!(
            record.get("description").and_then(Value::as_str).is_some(),
            "record '{key}' should include description for baseline quality tests"
        );
    }
}

#[test]
fn measures_db_fixture_includes_unknown_nested_field() {
    let raw = fs::read_to_string(fixture_path("measures_db.sample.json"))
        .expect("failed to read measures_db fixture");
    let parsed: Value = serde_json::from_str(&raw).expect("measures_db fixture must be valid json");

    let social = parsed
        .get("social_cohesion")
        .and_then(Value::as_object)
        .expect("social_cohesion fixture record must be present");
    let extra = social
        .get("extra_field")
        .and_then(Value::as_object)
        .expect("fixture should include extra_field object for passthrough tests");

    assert_eq!(
        extra.get("origin").and_then(Value::as_str),
        Some("fixture")
    );
}
