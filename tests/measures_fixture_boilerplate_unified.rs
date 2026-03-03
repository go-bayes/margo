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
fn boilerplate_unified_fixture_has_measures_map() {
    let raw = fs::read_to_string(fixture_path("boilerplate_unified.sample.json"))
        .expect("failed to read boilerplate_unified fixture");
    let parsed: Value =
        serde_json::from_str(&raw).expect("boilerplate_unified fixture must be valid json");

    let measures = parsed
        .get("measures")
        .and_then(Value::as_object)
        .expect("fixture must include a top-level 'measures' object");

    assert!(measures.contains_key("trust_science"));
    assert!(measures.contains_key("social_cohesion"));
}

#[test]
fn boilerplate_unified_fixture_contains_passthrough_field_example() {
    let raw = fs::read_to_string(fixture_path("boilerplate_unified.sample.json"))
        .expect("failed to read boilerplate_unified fixture");
    let parsed: Value =
        serde_json::from_str(&raw).expect("boilerplate_unified fixture must be valid json");

    let trust = parsed
        .get("measures")
        .and_then(Value::as_object)
        .and_then(|map| map.get("trust_science"))
        .and_then(Value::as_object)
        .expect("trust_science fixture record must be an object");

    let custom = trust
        .get("custom_field")
        .and_then(Value::as_str)
        .expect("fixture should include one unknown passthrough key");
    assert_eq!(custom, "preserve-me");
}
