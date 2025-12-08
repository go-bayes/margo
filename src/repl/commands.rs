// slash command handlers

use anyhow::{bail, Result};
use std::fs;
use std::process::Command;

use crate::commands::init;
use crate::config::Config;
use crate::theme;

use super::fuzzy;
use super::picker;

/// handle a slash command (without the leading /)
pub fn handle_slash(cmd: &str) -> Result<()> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let (command, args) = parts.split_first().map(|(&c, a)| (c, a)).unwrap_or(("", &[]));

    match command {
        "help" | "h" | "?" => cmd_help(),
        "config" => cmd_config(args),
        "templates" => cmd_templates(args),
        "vars" | "v" => cmd_vars(args),
        "clear" => cmd_clear(),
        _ => {
            println!(
                "{} unknown command: /{}",
                theme::yellow().paint("warning:"),
                theme::text().paint(command)
            );
            println!(
                "  type {} for available commands",
                theme::sapphire().paint("/help")
            );
            Ok(())
        }
    }
}

/// handle an init command
pub fn handle_init(cmd: &str) -> Result<()> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();

    // bare "init" - guide through everything
    if parts.len() < 2 {
        println!();
        let model = match picker::pick_model()? {
            Some(m) => m,
            None => {
                println!("{}", theme::yellow().paint("cancelled"));
                return Ok(());
            }
        };

        return match model.as_str() {
            "grf" => handle_init_grf(&[]),
            "grf-event" => handle_init_grf_event(&[]),
            "lmtp" => {
                println!(
                    "{} LMTP template not yet implemented",
                    theme::yellow().paint("warning:")
                );
                Ok(())
            }
            _ => Ok(()),
        };
    }

    match parts[1] {
        "grf" => handle_init_grf(&parts[2..]),
        "grf-event" => handle_init_grf_event(&parts[2..]),
        "lmtp" => {
            println!(
                "{} LMTP template not yet implemented",
                theme::yellow().paint("warning:")
            );
            Ok(())
        }
        _ => {
            println!(
                "{} unknown template: {}",
                theme::yellow().paint("warning:"),
                theme::text().paint(parts[1])
            );
            print_init_usage();
            Ok(())
        }
    }
}

fn handle_init_grf(args: &[&str]) -> Result<()> {
    // parse args: init grf [baseline] <exposure> [outcomes...]
    // flags: -t templates, -n name, -w who-mode
    let mut baselines: Option<String> = None;
    let mut exposure: Option<String> = None;
    let mut outcomes: Vec<String> = Vec::new();
    let mut templates: Option<Vec<String>> = None;
    let mut name: Option<String> = None;
    let mut who_mode = "default".to_string();

    let mut i = 0;
    while i < args.len() {
        let arg = args[i];

        if arg == "-t" || arg == "--templates" {
            i += 1;
            if i < args.len() {
                templates = Some(args[i].split(',').map(String::from).collect());
            }
        } else if arg == "-n" || arg == "--name" {
            i += 1;
            if i < args.len() {
                name = Some(args[i].to_string());
            }
        } else if arg == "-w" || arg == "--who-mode" {
            i += 1;
            if i < args.len() {
                who_mode = args[i].to_string();
            }
        } else if baselines.is_none() {
            // first positional could be baseline or exposure
            // check if it's a known baseline template
            let available_baselines = Config::list_baselines();
            if available_baselines.contains(&arg.to_string()) {
                baselines = Some(arg.to_string());
            } else {
                // assume it's the exposure
                exposure = Some(arg.to_string());
            }
        } else if exposure.is_none() {
            exposure = Some(arg.to_string());
        } else {
            outcomes.push(arg.to_string());
        }
        i += 1;
    }

    println!();

    // step 1: baseline template (if not provided)
    let baselines = if let Some(b) = baselines {
        b
    } else {
        let available = Config::list_baselines();
        if available.is_empty() {
            println!(
                "{}",
                theme::subtext0().paint("No baseline templates found, using default")
            );
            "default".to_string()
        } else {
            match picker::pick_baseline(&available)? {
                Some(selected) => selected,
                None => "default".to_string(),
            }
        }
    };

    // step 2: exposure variable (if not provided)
    let exposure = if let Some(e) = exposure {
        e
    } else {
        match picker::pick_exposure()? {
            Some(selected) => selected,
            None => {
                println!("{}", theme::yellow().paint("cancelled"));
                return Ok(());
            }
        }
    };

    // step 3: outcome variables (if not provided and no templates)
    if outcomes.is_empty() && templates.is_none() {
        match picker::pick_outcomes()? {
            Some(selected) if !selected.is_empty() => outcomes = selected,
            _ => {
                println!(
                    "{}",
                    theme::subtext0().paint("No outcomes selected - add them to study.toml later")
                );
            }
        }
    }

    println!();

    init::grf_from_config(
        &exposure,
        if outcomes.is_empty() {
            None
        } else {
            Some(&outcomes)
        },
        templates.as_deref(),
        &baselines,
        name.as_deref(),
        &who_mode,
    )
}

fn handle_init_grf_event(args: &[&str]) -> Result<()> {
    // parse args: init grf-event [baseline] <exposure> [-o outcome] [-w waves]
    let mut baselines: Option<String> = None;
    let mut exposure: Option<String> = None;
    let mut outcome: Option<String> = None;
    let mut waves: Option<Vec<String>> = None;
    let mut reference: Option<String> = None;
    let mut name: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        let arg = args[i];

        if arg == "-o" || arg == "--outcome" {
            i += 1;
            if i < args.len() {
                outcome = Some(args[i].to_string());
            }
        } else if arg == "-w" || arg == "--waves" {
            i += 1;
            if i < args.len() {
                waves = Some(args[i].split(',').map(String::from).collect());
            }
        } else if arg == "-r" || arg == "--reference" {
            i += 1;
            if i < args.len() {
                reference = Some(args[i].to_string());
            }
        } else if arg == "-n" || arg == "--name" {
            i += 1;
            if i < args.len() {
                name = Some(args[i].to_string());
            }
        } else if baselines.is_none() {
            // first positional could be baseline or exposure
            let available_baselines = Config::list_baselines();
            if available_baselines.contains(&arg.to_string()) {
                baselines = Some(arg.to_string());
            } else {
                exposure = Some(arg.to_string());
            }
        } else if exposure.is_none() {
            exposure = Some(arg.to_string());
        }
        i += 1;
    }

    println!();

    // step 1: baseline template
    let baselines = if let Some(b) = baselines {
        b
    } else {
        let available = Config::list_baselines();
        if available.is_empty() {
            "default".to_string()
        } else {
            match picker::pick_baseline(&available)? {
                Some(selected) => selected,
                None => "default".to_string(),
            }
        }
    };

    // step 2: exposure variable
    let exposure = if let Some(e) = exposure {
        e
    } else {
        match picker::pick_exposure()? {
            Some(selected) => selected,
            None => {
                println!("{}", theme::yellow().paint("cancelled"));
                return Ok(());
            }
        }
    };

    // step 3: outcome variable (single for event study)
    if outcome.is_none() {
        match picker::pick_variable("Select outcome variable:")? {
            Some(selected) => outcome = Some(selected),
            None => {}
        }
    }

    println!();

    init::grf_event_from_config(
        &exposure,
        outcome.as_deref(),
        waves.as_deref(),
        reference.as_deref(),
        &baselines,
        name.as_deref(),
    )
}

fn print_init_usage() {
    println!("  usage: init <template> [options]");
    println!();
    println!("  templates:");
    println!(
        "    {}   {}",
        theme::sapphire().paint("grf"),
        theme::subtext0().paint("generalised random forests")
    );
    println!(
        "    {}   {}",
        theme::sapphire().paint("grf-event"),
        theme::subtext0().paint("grf event study (multi-wave)")
    );
    println!(
        "    {}   {}",
        theme::overlay0().paint("lmtp"),
        theme::overlay0().paint("longitudinal modified treatment policies (coming soon)")
    );
}

fn cmd_help() -> Result<()> {
    println!();
    println!("  {}", theme::peach().paint("Commands"));
    println!(
        "  {}",
        theme::overlay0().paint("─────────────────────────────────────────────")
    );
    println!();

    println!("  {}", theme::subtext1().paint("Slash commands"));
    print_help_item("/help, /h", "show this help");
    print_help_item("/config", "show current configuration");
    print_help_item("/config edit", "edit config in $EDITOR");
    print_help_item("/config init", "create default config");
    print_help_item("/templates", "list all templates");
    print_help_item("/templates outcomes", "list outcome templates");
    print_help_item("/templates baselines", "list baseline templates");
    print_help_item("/templates edit <name>", "edit template in $EDITOR");
    print_help_item("/vars [pattern]", "fuzzy search variables");
    print_help_item("/clear", "clear the screen");
    print_help_item("/quit, /q", "exit margo");
    println!();

    println!("  {}", theme::subtext1().paint("Init commands"));
    print_help_item(
        "init grf <exposure> [outcomes...]",
        "create grf project",
    );
    print_help_item("init grf-event <exposure>", "create grf event study");
    println!();

    println!("  {}", theme::subtext1().paint("Options for init grf"));
    print_help_item("-t, --templates <list>", "outcome templates (comma-sep)");
    print_help_item("-b, --baselines <name>", "baseline template");
    print_help_item("-n, --name <name>", "custom project name");
    print_help_item("-w, --who-mode <mode>", "default, cat, or num");
    println!();

    println!("  {}", theme::subtext1().paint("Keybindings (vi mode)"));
    print_help_item("Esc", "switch to normal mode");
    print_help_item("i, a", "switch to insert mode");
    print_help_item("Ctrl+R", "reverse search history");
    print_help_item("Tab", "autocomplete");
    println!();

    Ok(())
}

fn print_help_item(cmd: &str, desc: &str) {
    println!(
        "    {:<32} {}",
        theme::sapphire().paint(cmd),
        theme::subtext0().paint(desc)
    );
}

fn cmd_config(args: &[&str]) -> Result<()> {
    let subcommand = args.first().copied().unwrap_or("");

    match subcommand {
        "" => {
            // show current config
            let config = Config::load();
            println!();
            println!("  {}", theme::peach().paint("Configuration"));
            println!(
                "  {}",
                theme::overlay0().paint("─────────────────────────────────────────────")
            );

            let config_path = Config::config_path();
            println!(
                "  {}: {}",
                theme::subtext0().paint("config file"),
                theme::text().paint(config_path.display().to_string())
            );
            println!();

            println!("  {}", theme::subtext1().paint("[paths]"));
            print_config_value(
                "pull_data",
                config.pull_data.as_deref().unwrap_or("(not set)"),
            );
            print_config_value(
                "push_mods",
                config.push_mods.as_deref().unwrap_or("(not set)"),
            );
            println!();

            println!("  {}", theme::subtext1().paint("[defaults]"));
            print_config_value(
                "baselines",
                config.baselines.as_deref().unwrap_or("default"),
            );
            print_config_value(
                "who_mode",
                config.who_mode.as_deref().unwrap_or("default"),
            );
            println!();

            Ok(())
        }
        "edit" => {
            let config_path = Config::config_path();
            let config_dir = Config::config_dir();

            // create if doesn't exist
            if !config_path.exists() {
                fs::create_dir_all(&config_dir)?;
                fs::write(&config_path, Config::default_config_content())?;
            }

            open_in_editor(&config_path.to_string_lossy())
        }
        "init" => {
            let config_path = Config::config_path();
            let config_dir = Config::config_dir();

            if config_path.exists() {
                println!(
                    "{} config already exists at: {}",
                    theme::yellow().paint("note:"),
                    config_path.display()
                );
                println!(
                    "  edit with: {}",
                    theme::sapphire().paint("/config edit")
                );
            } else {
                fs::create_dir_all(&config_dir)?;
                fs::write(&config_path, Config::default_config_content())?;
                println!(
                    "{} created config at: {}",
                    theme::green().paint("success:"),
                    config_path.display()
                );
            }
            Ok(())
        }
        "path" => {
            println!("{}", Config::config_path().display());
            Ok(())
        }
        _ => {
            println!(
                "{} unknown config subcommand: {}",
                theme::yellow().paint("warning:"),
                theme::text().paint(subcommand)
            );
            println!("  try: /config, /config edit, /config init, /config path");
            Ok(())
        }
    }
}

fn print_config_value(key: &str, value: &str) {
    println!(
        "    {} = {}",
        theme::sapphire().paint(key),
        theme::text().paint(value)
    );
}

fn cmd_templates(args: &[&str]) -> Result<()> {
    let subcommand = args.first().copied().unwrap_or("");

    match subcommand {
        "" => {
            // list all templates
            println!();
            list_templates("outcomes", &Config::list_outcomes());
            list_templates("baselines", &Config::list_baselines());
            Ok(())
        }
        "outcomes" => {
            println!();
            list_templates("outcomes", &Config::list_outcomes());
            Ok(())
        }
        "baselines" => {
            println!();
            list_templates("baselines", &Config::list_baselines());
            Ok(())
        }
        "edit" => {
            if args.len() < 2 {
                println!(
                    "{} missing template name",
                    theme::red().paint("error:")
                );
                println!("  usage: /templates edit <name>");
                return Ok(());
            }
            let name = args[1];

            // try outcomes first, then baselines
            let outcomes_path = Config::outcomes_dir().join(format!("{}.toml", name));
            let baselines_path = Config::baselines_dir().join(format!("{}.toml", name));

            if outcomes_path.exists() {
                open_in_editor(&outcomes_path.to_string_lossy())
            } else if baselines_path.exists() {
                open_in_editor(&baselines_path.to_string_lossy())
            } else {
                println!(
                    "{} template not found: {}",
                    theme::red().paint("error:"),
                    theme::text().paint(name)
                );
                println!(
                    "  check {} or {}",
                    Config::outcomes_dir().display(),
                    Config::baselines_dir().display()
                );
                Ok(())
            }
        }
        "new" => {
            if args.len() < 3 {
                println!(
                    "{} missing type and name",
                    theme::red().paint("error:")
                );
                println!("  usage: /templates new <outcomes|baselines> <name>");
                return Ok(());
            }
            let kind = args[1];
            let name = args[2];

            let (dir, template_content) = match kind {
                "outcomes" | "outcome" => (Config::outcomes_dir(), template_outcomes_content()),
                "baselines" | "baseline" => (Config::baselines_dir(), template_baselines_content()),
                _ => {
                    println!(
                        "{} type must be 'outcomes' or 'baselines'",
                        theme::red().paint("error:")
                    );
                    return Ok(());
                }
            };

            fs::create_dir_all(&dir)?;
            let path = dir.join(format!("{}.toml", name));

            if path.exists() {
                println!(
                    "{} template already exists: {}",
                    theme::yellow().paint("warning:"),
                    path.display()
                );
                println!(
                    "  edit with: {}",
                    theme::sapphire().paint(format!("/templates edit {}", name))
                );
                return Ok(());
            }

            fs::write(&path, template_content)?;
            println!(
                "{} created template: {}",
                theme::green().paint("success:"),
                path.display()
            );

            // open in editor
            open_in_editor(&path.to_string_lossy())
        }
        _ => {
            println!(
                "{} unknown templates subcommand: {}",
                theme::yellow().paint("warning:"),
                theme::text().paint(subcommand)
            );
            println!("  try: /templates, /templates outcomes, /templates baselines, /templates edit <name>");
            Ok(())
        }
    }
}

fn list_templates(kind: &str, templates: &[String]) {
    println!(
        "  {} {}",
        theme::peach().paint(kind),
        theme::overlay0().paint(format!("({})", templates.len()))
    );

    if templates.is_empty() {
        let dir = if kind == "outcomes" {
            Config::outcomes_dir()
        } else {
            Config::baselines_dir()
        };
        println!(
            "    {} none found in {}",
            theme::overlay0().paint("•"),
            dir.display()
        );
    } else {
        for name in templates {
            println!(
                "    {} {}",
                theme::overlay0().paint("•"),
                theme::sapphire().paint(name)
            );
        }
    }
    println!();
}

fn template_outcomes_content() -> String {
    r#"# outcome variables template
# add variable names from your dataset

vars = [
    # "wellbeing_index",
    # "life_satisfaction",
]
"#
    .to_string()
}

fn template_baselines_content() -> String {
    r#"# baseline covariate template
# add variable names to include as covariates

vars = [
    # "age",
    # "male",
    # "education_level_coarsen",
]
"#
    .to_string()
}

fn cmd_vars(args: &[&str]) -> Result<()> {
    let pattern = args.first().copied().unwrap_or("");

    let matches = fuzzy::search_variables(pattern);

    println!();
    if pattern.is_empty() {
        println!(
            "  {} {} variables available",
            theme::peach().paint("Variables:"),
            theme::text().paint(matches.len().to_string())
        );
        println!(
            "  {}",
            theme::subtext0().paint("use /vars <pattern> to search")
        );
    } else {
        println!(
            "  {} for '{}' ({} matches)",
            theme::peach().paint("Variables"),
            theme::sapphire().paint(pattern),
            theme::text().paint(matches.len().to_string())
        );
    }
    println!(
        "  {}",
        theme::overlay0().paint("─────────────────────────────────────────────")
    );

    let limit = if pattern.is_empty() { 20 } else { 30 };
    for var in matches.iter().take(limit) {
        println!(
            "    {} {}",
            theme::overlay0().paint("•"),
            theme::teal().paint(*var)
        );
    }

    if matches.len() > limit {
        println!(
            "    {} {} more...",
            theme::overlay0().paint("..."),
            theme::subtext0().paint((matches.len() - limit).to_string())
        );
    }
    println!();

    Ok(())
}

fn cmd_clear() -> Result<()> {
    // ansi escape to clear screen and move cursor to top
    print!("\x1B[2J\x1B[1;1H");
    Ok(())
}

fn open_in_editor(path: &str) -> Result<()> {
    // try config editor, then $EDITOR, then fall back to nvim
    let config = Config::load();
    let editor = config
        .editor
        .unwrap_or_else(|| std::env::var("EDITOR").unwrap_or_else(|_| "nvim".to_string()));

    // handle $EDITOR in config value
    let editor = if editor == "$EDITOR" {
        std::env::var("EDITOR").unwrap_or_else(|_| "nvim".to_string())
    } else {
        editor
    };

    // split editor command in case it has args (e.g., "code --wait")
    let parts: Vec<&str> = editor.split_whitespace().collect();
    let (cmd, args) = parts.split_first().map(|(&c, a)| (c, a)).unwrap_or(("nvim", &[]));

    let status = Command::new(cmd)
        .args(args)
        .arg(path)
        .status()?;

    if !status.success() {
        bail!("editor '{}' exited with error", editor);
    }

    Ok(())
}
