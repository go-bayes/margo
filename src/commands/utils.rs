// shared command utilities

use anyhow::Result;
use std::process::Command;

use crate::config::Config;

/// resolve the editor command from config or environment
/// priority: config.editor -> $EDITOR -> nvim
pub fn resolve_editor(config: &Config) -> String {
    let editor = config
        .editor
        .clone()
        .unwrap_or_else(|| std::env::var("EDITOR").unwrap_or_else(|_| "nvim".to_string()));

    // handle $EDITOR in config value
    if editor == "$EDITOR" {
        std::env::var("EDITOR").unwrap_or_else(|_| "nvim".to_string())
    } else {
        editor
    }
}

/// open a file in the user's preferred editor
/// returns Ok(true) if editor succeeded, Ok(false) if it failed
pub fn open_in_editor(filename: &str, config: &Config) -> Result<bool> {
    let editor = resolve_editor(config);

    // split editor command in case it has args (e.g., "code --wait")
    let parts: Vec<&str> = editor.split_whitespace().collect();
    let (cmd, args) = parts
        .split_first()
        .map(|(&c, a)| (c, a))
        .unwrap_or(("nvim", &[]));

    let status = Command::new(cmd)
        .args(args)
        .arg(filename)
        .status()?;

    Ok(status.success())
}
