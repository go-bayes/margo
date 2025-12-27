// user configuration
// loaded from ~/.config/margo/config.toml
// templates from ~/.config/margo/baselines/ and ~/.config/margo/outcomes/

use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

/// user configuration for margo projects
#[derive(Debug, Clone, Default)]
pub struct Config {
    // paths
    pub pull_data: Option<String>,   // where source data lives
    pub push_mods: Option<String>,   // base directory for outputs
    // defaults
    pub baselines: Option<String>,   // default baseline template name
    pub use_rv: Option<bool>,        // whether to include rv setup in generated scripts
    // editor
    pub editor: Option<String>,      // editor for /config edit, /templates edit
    // theme
    pub theme: Option<String>,       // "catppuccin" (default), "basic", or "plain"
}

/// a template (baselines or outcomes)
#[derive(Debug, Clone, Default)]
pub struct Template {
    #[allow(dead_code)]
    pub name: String,
    pub vars: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateKind {
    Baselines,
    Outcomes,
}

impl TemplateKind {
    fn as_str(&self) -> &'static str {
        match self {
            TemplateKind::Baselines => "baselines",
            TemplateKind::Outcomes => "outcomes",
        }
    }

    fn from_str(kind: &str) -> Option<Self> {
        match kind {
            "baselines" => Some(TemplateKind::Baselines),
            "outcomes" => Some(TemplateKind::Outcomes),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
struct TemplateAsset {
    kind: TemplateKind,
    name: &'static str,
    content: &'static str,
}

#[derive(Debug, Clone)]
struct ManifestEntry {
    kind: TemplateKind,
    name: String,
    hash: String,
}

#[derive(Debug, Clone)]
struct TemplateManifest {
    version: String,
    entries: Vec<ManifestEntry>,
}

#[derive(Debug, Clone, Default)]
pub struct TemplateRefreshReport {
    pub created: Vec<String>,
    pub updated: Vec<String>,
    pub unchanged: Vec<String>,
    pub skipped_modified: Vec<String>,
    pub sidecar: Vec<String>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TemplateRefreshOptions {
    pub force: bool,
    pub sidecar: bool,
    pub dry_run: bool,
}

impl Config {
    /// load config from ~/.config/margo/config.toml
    pub fn load() -> Self {
        let path = Self::config_path();

        if !path.exists() {
            return Self::default();
        }

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return Self::default(),
        };

        Self::parse(&content)
    }

    /// path to config file
    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    /// path to config directory (~/.config/margo)
    pub fn config_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".config")
            .join("margo")
    }

    /// path to baselines templates directory
    pub fn baselines_dir() -> PathBuf {
        Self::config_dir().join("baselines")
    }

    /// path to outcomes templates directory
    pub fn outcomes_dir() -> PathBuf {
        Self::config_dir().join("outcomes")
    }

    /// path to example baselines templates directory
    pub fn baselines_examples_dir() -> PathBuf {
        Self::baselines_dir().join("examples")
    }

    /// path to example outcomes templates directory
    pub fn outcomes_examples_dir() -> PathBuf {
        Self::outcomes_dir().join("examples")
    }

    /// parse toml content into config
    fn parse(content: &str) -> Self {
        let mut config = Self::default();

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() || line.starts_with('[') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_matches('"');

                match key {
                    "pull_data" => config.pull_data = Some(value.to_string()),
                    "push_mods" => config.push_mods = Some(value.to_string()),
                    "baselines" => config.baselines = Some(value.to_string()),
                    "use_rv" | "use_renv" => config.use_rv = Some(value == "true"),
                    "editor" | "command" => config.editor = Some(value.to_string()),
                    "theme" => config.theme = Some(value.to_string()),
                    _ => {}
                }
            }
        }

        config
    }

    /// generate default config file content
    pub fn default_config_content() -> String {
        r#"# margo configuration
# set your paths here, then run: init grf

[paths]
# where your .qs data files are stored (read from)
# pull_data = "/path/to/nzavs-data"

# base directory for model outputs (written to)
# a project subfolder will be created: {push_mods}/2025-exposure-outcomes/
# push_mods = "/path/to/outputs"

[defaults]
# default baseline template (from ~/.config/margo/baselines/)
# baselines = "default"

# include rv setup in generated R scripts (recommended for reproducibility)
# set to false if you manage R environments differently
use_rv = true

[editor]
# editor for /config edit, /templates edit
# uses $EDITOR if set, otherwise falls back to nvim
# command = "$EDITOR"

[theme]
# colour theme: "catppuccin" (default), "basic" (16 colours), "plain" (no colours)
# use "basic" or "plain" if colours don't display correctly in your terminal
# theme = "catppuccin"
"#.to_string()
    }

    /// ensure baseline/outcome templates exist; if none, write bundled defaults
    pub fn ensure_templates_initialized() -> Result<Vec<String>, String> {
        fs::create_dir_all(Self::baselines_dir())
            .map_err(|e| format!("failed to create baselines dir: {}", e))?;
        fs::create_dir_all(Self::outcomes_dir())
            .map_err(|e| format!("failed to create outcomes dir: {}", e))?;

        let mut created = Vec::new();
        let shipped = shipped_templates();

        for kind in [TemplateKind::Baselines, TemplateKind::Outcomes] {
            let dir = match kind {
                TemplateKind::Baselines => Self::baselines_dir(),
                TemplateKind::Outcomes => Self::outcomes_dir(),
            };

            if has_user_templates(&dir) {
                continue;
            }

            for asset in shipped.iter().filter(|a| a.kind == kind) {
                let path = dir.join(format!("{}.toml", asset.name));
                if path.exists() {
                    continue;
                }
                fs::write(&path, asset.content)
                    .map_err(|e| format!("failed to write {}: {}", path.display(), e))?;
                created.push(path.display().to_string());
            }
        }

        if !created.is_empty() {
            let manifest_path = templates_manifest_path();
            save_manifest(&manifest_path, &shipped_manifest())
                .map_err(|e| format!("failed to write manifest: {}", e))?;
        }

        Ok(created)
    }

    /// refresh templates against bundled defaults (hash-aware, non-destructive)
    pub fn refresh_templates(opts: TemplateRefreshOptions) -> Result<TemplateRefreshReport, String> {
        fs::create_dir_all(Self::baselines_dir())
            .map_err(|e| format!("failed to create baselines dir: {}", e))?;
        fs::create_dir_all(Self::outcomes_dir())
            .map_err(|e| format!("failed to create outcomes dir: {}", e))?;

        let shipped_manifest = shipped_manifest();
        let shipped_assets = shipped_templates();
        let previous_manifest = load_manifest(&templates_manifest_path());

        let mut report = TemplateRefreshReport::default();

        for asset in shipped_assets {
            let dest_dir = match asset.kind {
                TemplateKind::Baselines => Self::baselines_dir(),
                TemplateKind::Outcomes => Self::outcomes_dir(),
            };
            let dest_path = dest_dir.join(format!("{}.toml", asset.name));
            let dest_display = dest_path.display().to_string();

            let new_hash = hash_content(asset.content);
            let current_content = fs::read_to_string(&dest_path).ok();
            let current_hash = current_content.as_ref().map(|c| hash_content(c));
            let old_hash = previous_manifest
                .as_ref()
                .and_then(|m| m.find_hash(asset.kind, asset.name));

            // missing file -> create it
            if !dest_path.exists() {
                if !opts.dry_run {
                    fs::write(&dest_path, asset.content)
                        .map_err(|e| format!("failed to write {}: {}", dest_display, e))?;
                }
                report.created.push(dest_display);
                continue;
            }

            // force overwrite regardless of modification state
            if opts.force {
                if current_hash.as_deref() != Some(&new_hash) && !opts.dry_run {
                    fs::write(&dest_path, asset.content)
                        .map_err(|e| format!("failed to write {}: {}", dest_display, e))?;
                }
                if current_hash.as_deref() == Some(&new_hash) {
                    report.unchanged.push(dest_display);
                } else {
                    report.updated.push(dest_display);
                }
                continue;
            }

            // if hash matches previous shipped default, treat as unmodified
            if let (Some(prev_hash), Some(curr_hash)) = (old_hash, current_hash.as_deref()) {
                if prev_hash == curr_hash {
                    if curr_hash != new_hash {
                        if !opts.dry_run {
                            fs::write(&dest_path, asset.content)
                                .map_err(|e| format!("failed to write {}: {}", dest_display, e))?;
                        }
                        report.updated.push(dest_display);
                    } else {
                        report.unchanged.push(dest_display);
                    }
                    continue;
                }
            }

            // already up to date
            if current_hash.as_deref() == Some(&new_hash) {
                report.unchanged.push(dest_display);
                continue;
            }

            // modified by user
            if opts.sidecar {
                let sidecar_path = templates_sidecar_path(asset.kind).join(format!("{}.toml", asset.name));
                if !opts.dry_run {
                    if let Some(parent) = sidecar_path.parent() {
                        fs::create_dir_all(parent)
                            .map_err(|e| format!("failed to create sidecar dir: {}", e))?;
                    }
                    fs::write(&sidecar_path, asset.content)
                        .map_err(|e| format!("failed to write sidecar {}: {}", sidecar_path.display(), e))?;
                }
                report.sidecar.push(sidecar_path.display().to_string());
            } else {
                report.skipped_modified.push(dest_display);
            }
        }

        if !opts.dry_run {
            save_manifest(&templates_manifest_path(), &shipped_manifest)
                .map_err(|e| format!("failed to write manifest: {}", e))?;
        }

        Ok(report)
    }

    /// load a baselines template by name
    pub fn load_baselines(name: &str) -> Option<Template> {
        let path = Self::baselines_dir().join(format!("{}.toml", name));
        Self::load_template(&path, name)
    }

    /// load an outcomes template by name
    pub fn load_outcomes(name: &str) -> Option<Template> {
        let path = Self::outcomes_dir().join(format!("{}.toml", name));
        Self::load_template(&path, name)
    }

    /// load a template from a path
    fn load_template(path: &PathBuf, name: &str) -> Option<Template> {
        let content = fs::read_to_string(path).ok()?;
        let vars = Self::parse_vars(&content);

        if vars.is_empty() {
            None
        } else {
            Some(Template {
                name: name.to_string(),
                vars,
            })
        }
    }

    /// parse vars = [...] from template toml
    fn parse_vars(content: &str) -> Vec<String> {
        let mut vars = Vec::new();
        let mut in_vars = false;

        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("vars") && line.contains('[') {
                in_vars = true;
                // handle single-line: vars = ["a", "b"]
                if let Some(start) = line.find('[') {
                    if let Some(end) = line.find(']') {
                        let items = &line[start+1..end];
                        for item in items.split(',') {
                            let var = item.trim().trim_matches('"').trim_matches('\'');
                            if !var.is_empty() {
                                vars.push(var.to_string());
                            }
                        }
                        in_vars = false;
                    }
                }
                continue;
            }

            if in_vars {
                if line.contains(']') {
                    in_vars = false;
                }
                // parse items from multi-line array
                let line = line.trim_end_matches(',').trim_end_matches(']');
                let var = line.trim().trim_matches('"').trim_matches('\'');
                if !var.is_empty() && !var.starts_with('#') {
                    vars.push(var.to_string());
                }
            }
        }

        vars
    }

    /// list available baseline templates
    pub fn list_baselines() -> Vec<String> {
        Self::list_templates(&Self::baselines_dir())
    }

    /// list available outcome templates
    pub fn list_outcomes() -> Vec<String> {
        Self::list_templates(&Self::outcomes_dir())
    }

    fn list_templates(dir: &PathBuf) -> Vec<String> {
        let mut templates = Vec::new();
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                // skip the examples subdirectory when listing user templates
                if path.is_dir() {
                    continue;
                }
                if let Some(name) = path.file_stem() {
                    if path.extension().map(|e| e == "toml").unwrap_or(false) {
                        templates.push(name.to_string_lossy().to_string());
                    }
                }
            }
        }
        templates.sort();
        templates
    }

    /// list example baseline templates
    pub fn list_baselines_examples() -> Vec<String> {
        Self::list_templates(&Self::baselines_examples_dir())
    }

    /// list example outcome templates
    pub fn list_outcomes_examples() -> Vec<String> {
        Self::list_templates(&Self::outcomes_examples_dir())
    }

    /// copy an example template to user's config directory
    /// returns Ok(destination_path) on success
    pub fn copy_example(kind: &str, name: &str) -> Result<PathBuf, String> {
        let (examples_dir, user_dir) = match kind {
            "baselines" | "baseline" => (Self::baselines_examples_dir(), Self::baselines_dir()),
            "outcomes" | "outcome" => (Self::outcomes_examples_dir(), Self::outcomes_dir()),
            _ => return Err(format!("unknown template kind: {}", kind)),
        };

        let source = examples_dir.join(format!("{}.toml", name));
        if !source.exists() {
            return Err(format!("example '{}' not found in {}/examples/", name, kind));
        }

        // ensure user directory exists
        if let Err(e) = fs::create_dir_all(&user_dir) {
            return Err(format!("failed to create {}: {}", user_dir.display(), e));
        }

        let dest = user_dir.join(format!("{}.toml", name));
        if dest.exists() {
            return Err(format!(
                "'{}' already exists in your {} directory",
                name, kind
            ));
        }

        match fs::copy(&source, &dest) {
            Ok(_) => Ok(dest),
            Err(e) => Err(format!("failed to copy: {}", e)),
        }
    }

    /// get content of an example template (for viewing)
    #[allow(dead_code)]
    pub fn read_example(kind: &str, name: &str) -> Option<String> {
        let examples_dir = match kind {
            "baselines" | "baseline" => Self::baselines_examples_dir(),
            "outcomes" | "outcome" => Self::outcomes_examples_dir(),
            _ => return None,
        };

        let path = examples_dir.join(format!("{}.toml", name));
        fs::read_to_string(path).ok()
    }

    /// initialise example templates (creates examples/ directories with bundled templates)
    /// only writes files that don't already exist
    pub fn init_examples() -> Result<Vec<String>, String> {
        let mut created = Vec::new();

        // create directories
        let baselines_examples = Self::baselines_examples_dir();
        let outcomes_examples = Self::outcomes_examples_dir();

        fs::create_dir_all(&baselines_examples)
            .map_err(|e| format!("failed to create baselines/examples: {}", e))?;
        fs::create_dir_all(&outcomes_examples)
            .map_err(|e| format!("failed to create outcomes/examples: {}", e))?;

        // bundled baseline templates
        let baseline_templates = vec![
            ("default", Self::bundled_baseline_default()),
            ("minimal", Self::bundled_baseline_minimal()),
            ("extended", Self::bundled_baseline_extended()),
        ];

        for (name, content) in baseline_templates {
            let path = baselines_examples.join(format!("{}.toml", name));
            if !path.exists() {
                fs::write(&path, content)
                    .map_err(|e| format!("failed to write {}: {}", name, e))?;
                created.push(format!("baselines/examples/{}.toml", name));
            }
        }

        // bundled outcome templates
        let outcome_templates = vec![
            ("wellbeing", Self::bundled_outcomes_wellbeing()),
            ("health", Self::bundled_outcomes_health()),
        ];

        for (name, content) in outcome_templates {
            let path = outcomes_examples.join(format!("{}.toml", name));
            if !path.exists() {
                fs::write(&path, content)
                    .map_err(|e| format!("failed to write {}: {}", name, e))?;
                created.push(format!("outcomes/examples/{}.toml", name));
            }
        }

        Ok(created)
    }

    // bundled template contents

    fn bundled_baseline_default() -> &'static str {
        r#"# default baseline covariates
# standard set for NZAVS causal inference studies

vars = [
  # demographics
  "age",
  "born_nz_binary",
  "education_level_coarsen",
  "employed_binary",
  "eth_cat",
  "male_binary",
  "not_heterosexual_binary",
  "parent_binary",
  "partner_binary",
  "religion_identification_level",
  "rural_gch_2018_l",
  "sample_frame_opt_in_binary",
  # personality - Big Six
  "agreeableness",
  "conscientiousness",
  "extraversion",
  "honesty_humility",
  "neuroticism",
  "openness",
  # health/lifestyle
  "alcohol_frequency_weekly",
  "alcohol_intensity",
  "hlth_bmi",
  "hlth_disability_binary",
  "hlth_fatigue",
  "kessler_latent_anxiety",
  "kessler_latent_depression",
  "log_hours_children",
  "log_hours_commute",
  "log_hours_exercise",
  "log_hours_housework",
  "log_household_inc",
  "short_form_health",
  "smoker_binary",
  # social/psychological
  "belong",
  "nz_dep2018",
  "nzsei_13_l",
  "political_conservative",
  "rwa",
  "sdo",
  "support"
]
"#
    }

    fn bundled_baseline_minimal() -> &'static str {
        r#"# minimal baseline covariates
# core demographics only

vars = [
  "age",
  "male_binary",
  "eth_cat",
  "education_level_coarsen",
  "employed_binary",
  "partner_binary",
  "nz_dep2018"
]
"#
    }

    fn bundled_baseline_extended() -> &'static str {
        r#"# extended baseline covariates
# comprehensive set including additional psychological measures

vars = [
  # demographics
  "age",
  "born_nz_binary",
  "education_level_coarsen",
  "employed_binary",
  "eth_cat",
  "male_binary",
  "not_heterosexual_binary",
  "parent_binary",
  "partner_binary",
  "religion_identification_level",
  "rural_gch_2018_l",
  "sample_frame_opt_in_binary",
  # personality - Big Six
  "agreeableness",
  "conscientiousness",
  "extraversion",
  "honesty_humility",
  "neuroticism",
  "openness",
  # health/lifestyle
  "alcohol_frequency_weekly",
  "alcohol_intensity",
  "hlth_bmi",
  "hlth_disability_binary",
  "hlth_fatigue",
  "kessler_latent_anxiety",
  "kessler_latent_depression",
  "log_hours_children",
  "log_hours_commute",
  "log_hours_exercise",
  "log_hours_housework",
  "log_household_inc",
  "short_form_health",
  "smoker_binary",
  # social/psychological
  "belong",
  "nz_dep2018",
  "nzsei_13_l",
  "political_conservative",
  "rwa",
  "sdo",
  "support",
  # additional measures
  "gratitude",
  "modesty",
  "perfectionism",
  "self_esteem",
  "vengeful_rumination"
]
"#
    }

    fn bundled_outcomes_wellbeing() -> &'static str {
        r#"# wellbeing outcome variables
# psychological wellbeing measures

vars = [
  "life_satisfaction",
  "pwi",
  "self_esteem",
  "meaning_purpose",
  "gratitude"
]
"#
    }

    fn bundled_outcomes_health() -> &'static str {
        r#"# health outcome variables
# physical and mental health measures

vars = [
  "short_form_health",
  "kessler_latent_anxiety",
  "kessler_latent_depression",
  "hlth_fatigue",
  "hlth_sleep_hours"
]
"#
    }
}

// keep Defaults as alias for backwards compatibility
pub type Defaults = Config;

fn shipped_templates() -> Vec<TemplateAsset> {
    vec![
        TemplateAsset {
            kind: TemplateKind::Baselines,
            name: "default",
            content: Config::bundled_baseline_default(),
        },
        TemplateAsset {
            kind: TemplateKind::Baselines,
            name: "minimal",
            content: Config::bundled_baseline_minimal(),
        },
        TemplateAsset {
            kind: TemplateKind::Baselines,
            name: "extended",
            content: Config::bundled_baseline_extended(),
        },
        TemplateAsset {
            kind: TemplateKind::Outcomes,
            name: "wellbeing",
            content: Config::bundled_outcomes_wellbeing(),
        },
        TemplateAsset {
            kind: TemplateKind::Outcomes,
            name: "health",
            content: Config::bundled_outcomes_health(),
        },
    ]
}

fn shipped_manifest() -> TemplateManifest {
    let entries = shipped_templates()
        .iter()
        .map(|asset| ManifestEntry {
            kind: asset.kind,
            name: asset.name.to_string(),
            hash: hash_content(asset.content),
        })
        .collect();

    TemplateManifest {
        version: env!("CARGO_PKG_VERSION").to_string(),
        entries,
    }
}

fn hash_content(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

fn has_user_templates(dir: &Path) -> bool {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                continue;
            }
            if path.extension().map(|e| e == "toml").unwrap_or(false) {
                return true;
            }
        }
    }
    false
}

fn templates_manifest_path() -> PathBuf {
    Config::config_dir().join("templates.manifest")
}

fn templates_sidecar_path(kind: TemplateKind) -> PathBuf {
    let base = Config::config_dir().join("templates.new");
    match kind {
        TemplateKind::Baselines => base.join("baselines"),
        TemplateKind::Outcomes => base.join("outcomes"),
    }
}

impl TemplateManifest {
    fn find_hash(&self, kind: TemplateKind, name: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|e| e.kind == kind && e.name == name)
            .map(|e| e.hash.as_str())
    }
}

fn save_manifest(path: &Path, manifest: &TemplateManifest) -> std::io::Result<()> {
    let mut content = format!("version={}\n", manifest.version);
    for entry in &manifest.entries {
        content.push_str(&format!(
            "{}:{}:{}\n",
            entry.kind.as_str(),
            entry.name,
            entry.hash
        ));
    }
    fs::write(path, content)
}

fn load_manifest(path: &Path) -> Option<TemplateManifest> {
    let content = fs::read_to_string(path).ok()?;
    let mut version = String::new();
    let mut entries = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some(rest) = line.strip_prefix("version=") {
            version = rest.to_string();
            continue;
        }

        let mut parts = line.splitn(3, ':');
        let kind_str = parts.next()?;
        let name = parts.next()?;
        let hash = parts.next()?;

        let kind = TemplateKind::from_str(kind_str)?;

        entries.push(ManifestEntry {
            kind,
            name: name.to_string(),
            hash: hash.to_string(),
        });
    }

    if version.is_empty() {
        return None;
    }

    Some(TemplateManifest { version, entries })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config() {
        let content = r#"
[paths]
pull_data = "/Users/joseph/data/nzavs"
push_mods = "/Users/joseph/outputs"

[defaults]
baselines = "default"
use_rv = true
"#;

        let config = Config::parse(content);
        assert_eq!(config.pull_data, Some("/Users/joseph/data/nzavs".to_string()));
        assert_eq!(config.push_mods, Some("/Users/joseph/outputs".to_string()));
        assert_eq!(config.baselines, Some("default".to_string()));
        assert_eq!(config.use_rv, Some(true));
    }

    #[test]
    fn test_parse_config_accepts_use_renv() {
        let content = r#"
[defaults]
use_renv = false
"#;

        let config = Config::parse(content);
        assert_eq!(config.use_rv, Some(false));
    }

    #[test]
    fn test_empty_returns_default() {
        let config = Config::parse("");
        assert!(config.pull_data.is_none());
    }

    #[test]
    fn test_parse_vars_single_line() {
        let content = r#"vars = ["age", "male", "eth_cat"]"#;
        let vars = Config::parse_vars(content);
        assert_eq!(vars, vec!["age", "male", "eth_cat"]);
    }

    #[test]
    fn test_parse_vars_multi_line() {
        let content = r#"
vars = [
  "age",
  "male",
  "eth_cat",
]
"#;
        let vars = Config::parse_vars(content);
        assert_eq!(vars, vec!["age", "male", "eth_cat"]);
    }

    #[test]
    fn test_manifest_roundtrip() {
        let manifest = shipped_manifest();
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("templates.manifest");

        save_manifest(&path, &manifest).expect("save manifest");
        let loaded = load_manifest(&path).expect("load manifest");

        assert_eq!(manifest.version, loaded.version);
        assert_eq!(manifest.entries.len(), loaded.entries.len());
    }
}
