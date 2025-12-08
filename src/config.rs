// user configuration
// loaded from ~/.config/margo/config.toml
// templates from ~/.config/margo/baselines/ and ~/.config/margo/outcomes/

use std::fs;
use std::path::PathBuf;

/// user configuration for margo projects
#[derive(Debug, Clone, Default)]
pub struct Config {
    // paths
    pub pull_data: Option<String>,   // where source data lives
    pub push_mods: Option<String>,   // base directory for outputs
    // defaults
    pub baselines: Option<String>,   // default baseline template name
    pub who_mode: Option<String>,    // "default", "cat", or "num"
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
                    "who_mode" => config.who_mode = Some(value.to_string()),
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
# set your paths here, then use: margo init grf <exposure> [--outcomes X]

[paths]
# where your .qs data files are stored (read from)
# pull_data = "/path/to/nzavs-data"

# base directory for model outputs (written to)
# a project subfolder will be created: {push_mods}/2025-exposure-outcomes/
# push_mods = "/path/to/outputs"

[defaults]
# default baseline template (from ~/.config/margo/baselines/)
# baselines = "default"

# BMI/exercise variable mode: "default", "cat", or "num"
# who_mode = "default"

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
                if let Some(name) = entry.path().file_stem() {
                    if entry.path().extension().map(|e| e == "toml").unwrap_or(false) {
                        templates.push(name.to_string_lossy().to_string());
                    }
                }
            }
        }
        templates.sort();
        templates
    }
}

// keep Defaults as alias for backwards compatibility
pub type Defaults = Config;

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
who_mode = "cat"
"#;

        let config = Config::parse(content);
        assert_eq!(config.pull_data, Some("/Users/joseph/data/nzavs".to_string()));
        assert_eq!(config.push_mods, Some("/Users/joseph/outputs".to_string()));
        assert_eq!(config.baselines, Some("default".to_string()));
        assert_eq!(config.who_mode, Some("cat".to_string()));
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
}
