use std::path::PathBuf;

mod data {
    pub mod measures {
        include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/data/measures.rs"));
    }

    pub mod measure_workspace {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/data/measure_workspace.rs"
        ));
    }
}

use data::measure_workspace::MeasureWorkspace;
use data::measures::load_measure_records_from_path;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("measures")
        .join(name)
}

#[test]
fn workspace_loads_source_and_lists_records() {
    let fixture = fixture_path("measures_db.sample.json");
    let workspace = MeasureWorkspace::load(&fixture).expect("workspace load should succeed");

    assert_eq!(workspace.record_count(), 2);
    assert!(!workspace.is_dirty());
    assert!(workspace.source().is_some());
    assert_eq!(workspace.list(None).len(), 2);
    assert_eq!(workspace.list(Some("science")).len(), 1);
}

#[test]
fn workspace_add_edit_rename_delete_flow_marks_dirty() {
    let fixture = fixture_path("measures_db.sample.json");
    let (_, records) = load_measure_records_from_path(&fixture).expect("load records");
    let mut workspace = MeasureWorkspace::from_source(
        data::measures::MeasureSourceInfo::new(
            fixture,
            data::measures::MeasureFileFormat::MeasuresDbJson,
        ),
        records,
    );

    workspace.add("new_measure").expect("add");
    workspace
        .edit_field("new_measure", "description", "new description")
        .expect("edit");
    workspace
        .rename("new_measure", "renamed_measure")
        .expect("rename");
    assert!(workspace.delete("renamed_measure"));
    assert!(workspace.is_dirty());
}

#[test]
fn workspace_validate_and_export_missing_description() {
    let fixture = fixture_path("measures_db.sample.json");
    let mut workspace = MeasureWorkspace::load(&fixture).expect("workspace load should succeed");

    workspace.add("empty_measure").expect("add");

    let report = workspace.validate_basic();
    assert!(report.missing_description.contains(&"empty_measure".to_string()));

    let missing = workspace.export_missing("description");
    assert!(missing.contains(&"empty_measure".to_string()));
}
