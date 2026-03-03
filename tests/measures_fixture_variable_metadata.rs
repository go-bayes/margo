use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("measures")
        .join(name)
}

#[test]
fn variable_metadata_fixture_has_expected_header() {
    let raw = fs::read_to_string(fixture_path("variable_metadata.sample.tsv"))
        .expect("failed to read variable_metadata fixture");
    let mut lines = raw.lines();

    let header = lines.next().expect("fixture must include a header row");
    assert_eq!(header, "variable\tdescription");
}

#[test]
fn variable_metadata_fixture_has_unique_variable_names() {
    let raw = fs::read_to_string(fixture_path("variable_metadata.sample.tsv"))
        .expect("failed to read variable_metadata fixture");

    let mut seen = HashSet::new();
    let mut row_count = 0usize;

    for line in raw.lines().skip(1) {
        let parts: Vec<&str> = line.splitn(2, '\t').collect();
        assert_eq!(parts.len(), 2, "each row should have two tsv columns");
        assert!(
            !parts[0].trim().is_empty(),
            "variable name should not be empty"
        );
        assert!(
            !parts[1].trim().is_empty(),
            "description should not be empty"
        );
        assert!(
            seen.insert(parts[0].trim().to_string()),
            "duplicate variable name found in fixture: {}",
            parts[0]
        );
        row_count += 1;
    }

    assert!(row_count >= 3, "fixture should include at least three rows");
}
