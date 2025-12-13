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

/// create a temp config file with paths set up
fn setup_config(tmp: &tempfile::TempDir) {
    let config_dir = tmp.path().join(".config").join("margo");
    fs::create_dir_all(&config_dir).expect("failed to create config dir");

    let config_content = format!(
        r#"[paths]
pull_data = "{}"
push_mods = "{}"
"#,
        tmp.path().display(),
        tmp.path().join("outputs").display()
    );

    fs::write(config_dir.join("config.toml"), config_content)
        .expect("failed to write config file");
}

#[test]
fn test_grf_creates_files_in_current_directory() {
    let tmp = temp_dir();
    setup_config(&tmp);

    let output = Command::new(margo_bin())
        .args(["init", "grf", "test_exposure"])
        .current_dir(tmp.path())
        .env("HOME", tmp.path())
        .output()
        .expect("failed to execute margo");

    assert!(output.status.success(), "margo init grf failed: {:?}", output);

    // files are created in current directory (not a subdirectory)
    let study_toml = tmp.path().join("study.toml");
    assert!(study_toml.exists(), "study.toml was not created in current directory");
}

#[test]
fn test_grf_creates_all_expected_files() {
    let tmp = temp_dir();
    setup_config(&tmp);

    Command::new(margo_bin())
        .args(["init", "grf", "test_exposure"])
        .current_dir(tmp.path())
        .env("HOME", tmp.path())
        .output()
        .expect("failed to execute margo");

    let expected_files = [
        "study.toml",
        "README.md",
        ".gitignore",
        "00-setup.R",
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
        let file_path = tmp.path().join(file);
        assert!(file_path.exists(), "expected file {} was not created", file);
    }
}

#[test]
fn test_grf_study_toml_is_valid_toml() {
    let tmp = temp_dir();
    setup_config(&tmp);

    Command::new(margo_bin())
        .args(["init", "grf", "test_exposure"])
        .current_dir(tmp.path())
        .env("HOME", tmp.path())
        .output()
        .expect("failed to execute margo");

    let toml_path = tmp.path().join("study.toml");
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
    setup_config(&tmp);

    // also create a baselines template
    let baselines_dir = tmp.path().join(".config").join("margo").join("baselines");
    fs::create_dir_all(&baselines_dir).expect("failed to create baselines dir");
    let default_baselines = r#"vars = [
    "age",
    "male_binary",
    "agreeableness",
    "conscientiousness",
    "extraversion",
    "honesty_humility",
    "neuroticism",
    "openness",
    "rwa",
    "sdo"
]"#;
    fs::write(baselines_dir.join("default.toml"), default_baselines)
        .expect("failed to write baselines template");

    Command::new(margo_bin())
        .args(["init", "grf", "test_exposure"])
        .current_dir(tmp.path())
        .env("HOME", tmp.path())
        .output()
        .expect("failed to execute margo");

    let toml_path = tmp.path().join("study.toml");
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
}

#[test]
fn test_grf_creates_setup_script_with_renv() {
    let tmp = temp_dir();
    setup_config(&tmp);

    Command::new(margo_bin())
        .args(["init", "grf", "test_exposure"])
        .current_dir(tmp.path())
        .env("HOME", tmp.path())
        .output()
        .expect("failed to execute margo");

    // check that 00-setup.R was created with renv content
    let setup_path = tmp.path().join("00-setup.R");
    assert!(setup_path.exists(), "00-setup.R should exist");

    let setup_content = fs::read_to_string(&setup_path).expect("failed to read 00-setup.R");
    assert!(
        setup_content.contains("renv::init()"),
        "00-setup.R should contain renv::init() by default"
    );
}

#[test]
fn test_grf_study_toml_ordinal_vars_are_subset_of_baseline() {
    let tmp = temp_dir();
    setup_config(&tmp);

    // create a baselines template with ordinal vars
    let baselines_dir = tmp.path().join(".config").join("margo").join("baselines");
    fs::create_dir_all(&baselines_dir).expect("failed to create baselines dir");
    let default_baselines = r#"vars = [
    "age",
    "education_level_coarsen",
    "eth_cat",
    "rural_gch_2018_l"
]"#;
    fs::write(baselines_dir.join("default.toml"), default_baselines)
        .expect("failed to write baselines template");

    Command::new(margo_bin())
        .args(["init", "grf", "test_exposure"])
        .current_dir(tmp.path())
        .env("HOME", tmp.path())
        .output()
        .expect("failed to execute margo");

    let toml_path = tmp.path().join("study.toml");
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
fn test_grf_exposure_appears_in_study_toml() {
    let tmp = temp_dir();
    setup_config(&tmp);

    Command::new(margo_bin())
        .args(["init", "grf", "church_attendance"])
        .current_dir(tmp.path())
        .env("HOME", tmp.path())
        .output()
        .expect("failed to execute margo");

    // check study.toml contains exposure name
    let toml_content = fs::read_to_string(tmp.path().join("study.toml")).unwrap();
    assert!(
        toml_content.contains("church_attendance"),
        "study.toml should contain exposure name"
    );
}

#[test]
fn test_grf_waves_default_to_time_10_11_12() {
    let tmp = temp_dir();
    setup_config(&tmp);

    Command::new(margo_bin())
        .args(["init", "grf", "test_exposure"])
        .current_dir(tmp.path())
        .env("HOME", tmp.path())
        .output()
        .expect("failed to execute margo");

    let toml_path = tmp.path().join("study.toml");
    let content = fs::read_to_string(&toml_path).expect("failed to read study.toml");
    let parsed: toml::Table = content.parse().expect("study.toml is not valid TOML");

    let waves = parsed.get("waves").expect("missing waves section");

    let baseline = waves.get("baseline").expect("missing waves.baseline").as_str().unwrap();
    let outcome = waves.get("outcome").expect("missing waves.outcome").as_str().unwrap();

    assert_eq!(baseline, "Time 10", "baseline wave should be Time 10");
    assert_eq!(outcome, "Time 12", "outcome wave should be Time 12");
}
