// tab completion with fuzzy matching

use reedline::{Completer, Span, Suggestion};

use crate::config::Config;

use super::fuzzy;

pub struct MargoCompleter {
    commands: Vec<&'static str>,
    slash_commands: Vec<&'static str>,
}

impl MargoCompleter {
    pub fn new() -> Self {
        Self {
            commands: vec!["init grf", "init grf-event", "init lmtp", "quit", "exit"],
            slash_commands: vec![
                "/help",
                "/h",
                "/config",
                "/config edit",
                "/config init",
                "/config path",
                "/templates",
                "/templates outcomes",
                "/templates baselines",
                "/templates edit",
                "/templates new outcomes",
                "/templates new baselines",
                "/vars",
                "/view",
                "/save",
                "/save outcomes",
                "/save baselines",
                "/theme",
                "/theme light",
                "/theme dark",
                "/theme toggle",
                "/clear",
                "/quit",
                "/q",
            ],
        }
    }
}

impl Default for MargoCompleter {
    fn default() -> Self {
        Self::new()
    }
}

impl Completer for MargoCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        let line_to_cursor = &line[..pos];

        // slash commands
        if line_to_cursor.starts_with('/') {
            return complete_slash(line_to_cursor, &self.slash_commands);
        }

        // template name after -t or --templates
        if let Some(prefix) = extract_after_flag(line_to_cursor, &["-t", "--templates"]) {
            return complete_templates(prefix, "outcomes");
        }

        // baseline name after -b or --baselines
        if let Some(prefix) = extract_after_flag(line_to_cursor, &["-b", "--baselines"]) {
            return complete_templates(prefix, "baselines");
        }

        // init command completion
        if line_to_cursor.starts_with("init ") {
            let after_init = &line_to_cursor[5..];

            // complete template type
            if !after_init.contains(' ') {
                return complete_init_template(after_init);
            }

            // after "init grf " - complete variables
            if after_init.starts_with("grf ") || after_init.starts_with("grf-event ") {
                let parts: Vec<&str> = after_init.split_whitespace().collect();
                if let Some(last) = parts.last() {
                    // don't complete flags
                    if !last.starts_with('-') {
                        return complete_variables(last);
                    }
                }
            }
        }

        // command completion at start
        complete_commands(line_to_cursor, &self.commands)
    }
}

fn complete_slash(prefix: &str, commands: &[&str]) -> Vec<Suggestion> {
    let matches = fuzzy::search_templates(prefix, &commands.iter().map(|s| s.to_string()).collect::<Vec<_>>());

    matches
        .into_iter()
        .map(|cmd| Suggestion {
            value: cmd.clone(),
            description: None,
            style: None,
            extra: None,
            span: Span::new(0, prefix.len()),
            append_whitespace: true,
        })
        .collect()
}

fn complete_commands(prefix: &str, commands: &[&str]) -> Vec<Suggestion> {
    commands
        .iter()
        .filter(|cmd| cmd.starts_with(prefix))
        .map(|cmd| Suggestion {
            value: cmd.to_string(),
            description: None,
            style: None,
            extra: None,
            span: Span::new(0, prefix.len()),
            append_whitespace: true,
        })
        .collect()
}

fn complete_init_template(prefix: &str) -> Vec<Suggestion> {
    let templates = ["grf", "grf-event", "lmtp"];

    templates
        .iter()
        .filter(|t| t.starts_with(prefix))
        .map(|t| Suggestion {
            value: t.to_string(),
            description: Some(match *t {
                "grf" => "generalised random forests".to_string(),
                "grf-event" => "grf event study".to_string(),
                "lmtp" => "longitudinal modified treatment policies".to_string(),
                _ => String::new(),
            }),
            style: None,
            extra: None,
            span: Span::new(5, 5 + prefix.len()), // after "init "
            append_whitespace: true,
        })
        .collect()
}

fn complete_variables(prefix: &str) -> Vec<Suggestion> {
    let matches = fuzzy::search_variables(prefix);

    matches
        .into_iter()
        .take(20) // limit suggestions
        .map(|var| Suggestion {
            value: var.to_string(),
            description: None,
            style: None,
            extra: None,
            span: Span::new(0, 0), // will be calculated by caller
            append_whitespace: true,
        })
        .collect()
}

fn complete_templates(prefix: &str, kind: &str) -> Vec<Suggestion> {
    let templates = match kind {
        "outcomes" => Config::list_outcomes(),
        "baselines" => Config::list_baselines(),
        _ => Vec::new(),
    };

    let matches = fuzzy::search_templates(prefix, &templates);

    matches
        .into_iter()
        .map(|name| Suggestion {
            value: name.clone(),
            description: None,
            style: None,
            extra: None,
            span: Span::new(0, prefix.len()),
            append_whitespace: true,
        })
        .collect()
}

fn extract_after_flag<'a>(line: &'a str, flags: &[&str]) -> Option<&'a str> {
    for flag in flags {
        if let Some(pos) = line.rfind(flag) {
            let after_flag = &line[pos + flag.len()..];
            // must have space after flag
            if after_flag.starts_with(' ') {
                let value = after_flag.trim_start();
                // only if we're at the end (no more spaces means we're typing the value)
                if !value.contains(' ') {
                    return Some(value);
                }
            }
        }
    }
    None
}
