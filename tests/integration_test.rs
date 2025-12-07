// integration tests for margo CLI

use std::fs;
use std::process::Command;

/// get path to the margo binary
fn margo_bin() -> std::path::PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // remove test binary name
    path.pop(); // remove 'deps'
    path.push("margo");
    path
}

/// create a temporary directory for test projects
fn temp_dir() -> tempfile::TempDir {
    tempfile::tempdir().expect("failed to create temp dir")
}

#[test]
fn test_grf_creates_project_directory() {
    let tmp = temp_dir();
    let project_path = tmp.path().join("test-project");

    let output = Command::new(margo_bin())
        .args(["init", "grf", "test-project"])
        .current_dir(tmp.path())
        .output()
        .expect("failed to execute margo");

    assert!(output.status.success(), "margo init grf failed: {:?}", output);
    assert!(project_path.exists(), "project directory was not created");
}

#[test]
fn test_grf_creates_all_expected_files() {
    let tmp = temp_dir();
    let project_path = tmp.path().join("test-project");

    Command::new(margo_bin())
        .args(["init", "grf", "test-project"])
        .current_dir(tmp.path())
        .output()
        .expect("failed to execute margo");

    let expected_files = [
        "study.toml",
        "README.md",
        ".gitignore",
        "01-data-prep.R",
        "02-wide-format.R",
        "03-causal-forest.R",
        "04-heterogeneity.R",
        "05-policy-tree.R",
        "06-positivity.R",
        "07-tables.R",
        "08-plots.R",
    ];

    for file in expected_files {
        let file_path = project_path.join(file);
        assert!(file_path.exists(), "expected file {} was not created", file);
    }
}

#[test]
fn test_grf_study_toml_is_valid_toml() {
    let tmp = temp_dir();
    let project_path = tmp.path().join("test-project");

    Command::new(margo_bin())
        .args(["init", "grf", "test-project"])
        .current_dir(tmp.path())
        .output()
        .expect("failed to execute margo");

    let toml_path = project_path.join("study.toml");
    let content = fs::read_to_string(&toml_path).expect("failed to read study.toml");

    // parse as TOML - will panic if invalid
    let parsed: toml::Table = content.parse().expect("study.toml is not valid TOML");

    // check key sections exist
    assert!(parsed.contains_key("paths"), "missing [paths] section");
    assert!(parsed.contains_key("waves"), "missing [waves] section");
    assert!(parsed.contains_key("exposure"), "missing [exposure] section");
    assert!(parsed.contains_key("outcomes"), "missing [outcomes] section");
    assert!(parsed.contains_key("baseline"), "missing [baseline] section");
    assert!(parsed.contains_key("confounders"), "missing [confounders] section");
    assert!(parsed.contains_key("ordinal"), "missing [ordinal] section");
    assert!(parsed.contains_key("grf"), "missing [grf] section");
}

#[test]
fn test_grf_study_toml_has_standard_baseline_vars() {
    let tmp = temp_dir();
    let project_path = tmp.path().join("test-project");

    Command::new(margo_bin())
        .args(["init", "grf", "test-project"])
        .current_dir(tmp.path())
        .output()
        .expect("failed to execute margo");

    let toml_path = project_path.join("study.toml");
    let content = fs::read_to_string(&toml_path).expect("failed to read study.toml");
    let parsed: toml::Table = content.parse().expect("study.toml is not valid TOML");

    let baseline = parsed.get("baseline").expect("missing baseline section");
    let vars = baseline
        .get("vars")
        .expect("missing baseline.vars")
        .as_array()
        .expect("baseline.vars is not an array");

    // check for key baseline variables
    let var_names: Vec<&str> = vars
        .iter()
        .filter_map(|v| v.as_str())
        .collect();

    assert!(var_names.contains(&"age"), "missing 'age' in baseline vars");
    assert!(var_names.contains(&"male_binary"), "missing 'male_binary' in baseline vars");
    assert!(var_names.contains(&"agreeableness"), "missing 'agreeableness' in baseline vars");
    assert!(var_names.contains(&"conscientiousness"), "missing 'conscientiousness' in baseline vars");
    assert!(var_names.contains(&"extraversion"), "missing 'extraversion' in baseline vars");
    assert!(var_names.contains(&"honesty_humility"), "missing 'honesty_humility' in baseline vars");
    assert!(var_names.contains(&"neuroticism"), "missing 'neuroticism' in baseline vars");
    assert!(var_names.contains(&"openness"), "missing 'openness' in baseline vars");
    assert!(var_names.contains(&"rwa"), "missing 'rwa' in baseline vars");
    assert!(var_names.contains(&"sdo"), "missing 'sdo' in baseline vars");

    // check we have approximately the right count (39 standard vars)
    assert!(vars.len() >= 35, "expected at least 35 baseline vars, got {}", vars.len());
}

#[test]
fn test_grf_study_toml_has_who_mode() {
    let tmp = temp_dir();
    let project_path = tmp.path().join("test-project");

    Command::new(margo_bin())
        .args(["init", "grf", "test-project"])
        .current_dir(tmp.path())
        .output()
        .expect("failed to execute margo");

    let toml_path = project_path.join("study.toml");
    let content = fs::read_to_string(&toml_path).expect("failed to read study.toml");
    let parsed: toml::Table = content.parse().expect("study.toml is not valid TOML");

    let baseline = parsed.get("baseline").expect("missing baseline section");
    let who_mode = baseline
        .get("who_mode")
        .expect("missing baseline.who_mode")
        .as_str()
        .expect("who_mode is not a string");

    assert_eq!(who_mode, "default", "who_mode should default to 'default'");
}

#[test]
fn test_grf_study_toml_ordinal_vars_are_subset_of_baseline() {
    let tmp = temp_dir();
    let project_path = tmp.path().join("test-project");

    Command::new(margo_bin())
        .args(["init", "grf", "test-project"])
        .current_dir(tmp.path())
        .output()
        .expect("failed to execute margo");

    let toml_path = project_path.join("study.toml");
    let content = fs::read_to_string(&toml_path).expect("failed to read study.toml");
    let parsed: toml::Table = content.parse().expect("study.toml is not valid TOML");

    // get baseline vars
    let baseline = parsed.get("baseline").expect("missing baseline section");
    let baseline_vars: Vec<String> = baseline
        .get("vars")
        .expect("missing baseline.vars")
        .as_array()
        .expect("baseline.vars is not an array")
        .iter()
        .filter_map(|v| v.as_str())
        .map(|s| format!("t0_{}", s))
        .collect();

    // get ordinal vars
    let ordinal = parsed.get("ordinal").expect("missing ordinal section");
    let ordinal_vars: Vec<&str> = ordinal
        .get("vars")
        .expect("missing ordinal.vars")
        .as_array()
        .expect("ordinal.vars is not an array")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();

    // check each ordinal var has a corresponding baseline var
    for ordinal_var in &ordinal_vars {
        assert!(
            baseline_vars.contains(&ordinal_var.to_string()),
            "ordinal var '{}' has no corresponding baseline var",
            ordinal_var
        );
    }
}

#[test]
fn test_grf_project_name_appears_in_files() {
    let tmp = temp_dir();
    let project_name = "my-custom-study";
    let project_path = tmp.path().join(project_name);

    Command::new(margo_bin())
        .args(["init", "grf", project_name])
        .current_dir(tmp.path())
        .output()
        .expect("failed to execute margo");

    // check study.toml contains project name
    let toml_content = fs::read_to_string(project_path.join("study.toml")).unwrap();
    assert!(
        toml_content.contains(project_name),
        "study.toml should contain project name"
    );

    // check README contains project name
    let readme_content = fs::read_to_string(project_path.join("README.md")).unwrap();
    assert!(
        readme_content.contains(project_name),
        "README.md should contain project name"
    );
}

#[test]
fn test_grf_waves_default_to_time_10_11_12() {
    let tmp = temp_dir();
    let project_path = tmp.path().join("test-project");

    Command::new(margo_bin())
        .args(["init", "grf", "test-project"])
        .current_dir(tmp.path())
        .output()
        .expect("failed to execute margo");

    let toml_path = project_path.join("study.toml");
    let content = fs::read_to_string(&toml_path).expect("failed to read study.toml");
    let parsed: toml::Table = content.parse().expect("study.toml is not valid TOML");

    let waves = parsed.get("waves").expect("missing waves section");

    let baseline = waves.get("baseline").expect("missing waves.baseline").as_str().unwrap();
    let outcome = waves.get("outcome").expect("missing waves.outcome").as_str().unwrap();

    assert_eq!(baseline, "Time 10", "baseline wave should be Time 10");
    assert_eq!(outcome, "Time 12", "outcome wave should be Time 12");
}
