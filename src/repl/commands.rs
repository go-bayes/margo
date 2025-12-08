// slash command handlers

use anyhow::{bail, Result};
use std::env;
use std::fs;
use std::process::Command;

use crate::commands::init;
use crate::config::Config;
use crate::data::VARIABLES;
use crate::theme;

use super::fuzzy;
use super::picker;
use super::welcome;

/// handle a slash command (without the leading /)
pub fn handle_slash(cmd: &str) -> Result<()> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let (command, args) = parts.split_first().map(|(&c, a)| (c, a)).unwrap_or(("", &[]));

    match command {
        "" => cmd_picker(),
        "help" | "h" | "?" => cmd_help(),
        "config" => cmd_config(args),
        "templates" => cmd_templates(args),
        "view" => cmd_view(args),
        "save" => cmd_save(args),
        "vars" | "v" => cmd_vars(args),
        "theme" | "t" => cmd_theme(args),
        "here" | "pwd" => cmd_here(),
        "home" | "~" => cmd_home(),
        "cd" => cmd_cd(args),
        "e" | "o" => cmd_quick_edit(args),
        "refresh" | "r" => cmd_refresh(),
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

    // step 1: validate exposure FIRST if provided on command line
    let exposure = if let Some(e) = exposure {
        if VARIABLES.contains(&e.as_str()) {
            Some(e)
        } else {
            // fuzzy search for similar variables
            let matches: Vec<String> = fuzzy::search_variables(&e)
                .into_iter()
                .take(10)
                .map(String::from)
                .collect();

            if matches.is_empty() {
                println!(
                    "{} variable '{}' not found",
                    theme::red().paint("error:"),
                    e
                );
                return Ok(());
            }

            println!(
                "{} '{}' not found, showing similar:",
                theme::yellow().paint("note:"),
                e
            );

            match picker::pick_from_matches(&matches)? {
                Some(selected) => Some(selected),
                None => {
                    println!("{}", theme::yellow().paint("cancelled"));
                    return Ok(());
                }
            }
        }
    } else {
        None
    };

    // step 2: baseline template (if not provided)
    let (baseline, baseline_vars_override) = if let Some(b) = baselines {
        (b, None)
    } else {
        let available = Config::list_baselines();
        if available.is_empty() {
            println!(
                "{}",
                theme::subtext0().paint("no baseline templates found, using default")
            );
            ("default".to_string(), None)
        } else {
            // offer choice: use template as-is, modify, or pick custom
            let methods = vec![
                "template     — use saved baseline template",
                "modify       — edit template variables",
                "custom       — pick individual variables",
            ];

            let method = inquire::Select::new("Select baseline from:", methods)
                .with_help_message("↑↓ navigate, Enter select, Esc cancel")
                .prompt_skippable()?;

            match method {
                Some(m) if m.starts_with("template") => {
                    match picker::pick_baseline(&available)? {
                        Some(selected) => (selected, None),
                        None => {
                            println!("{}", theme::yellow().paint("cancelled"));
                            return Ok(());
                        }
                    }
                }
                Some(m) if m.starts_with("modify") => {
                    // pick template then edit its variables
                    let tpl_name = match picker::pick_baseline(&available)? {
                        Some(selected) => selected,
                        None => {
                            println!("{}", theme::yellow().paint("cancelled"));
                            return Ok(());
                        }
                    };
                    // load template vars and let user modify
                    let current_vars = Config::load_baselines(&tpl_name)
                        .map(|t| t.vars)
                        .unwrap_or_default();
                    match picker::edit_template(&tpl_name, &current_vars)? {
                        Some(vars) => (tpl_name, Some(vars)),
                        None => {
                            println!("{}", theme::yellow().paint("cancelled"));
                            return Ok(());
                        }
                    }
                }
                Some(m) if m.starts_with("custom") => {
                    // pick individual variables
                    match picker::pick_outcomes()? {
                        Some(vars) if !vars.is_empty() => ("custom".to_string(), Some(vars)),
                        _ => {
                            println!("{}", theme::yellow().paint("cancelled"));
                            return Ok(());
                        }
                    }
                }
                _ => {
                    println!("{}", theme::yellow().paint("cancelled"));
                    return Ok(());
                }
            }
        }
    };

    // step 3: exposure picker (if not provided on command line)
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

    // step 4: outcome variables (if not provided and no templates)
    if outcomes.is_empty() && templates.is_none() {
        // offer choice: templates or individual variables
        let available_templates = Config::list_outcomes();

        if available_templates.is_empty() {
            // no templates, just pick variables
            match picker::pick_outcomes()? {
                Some(selected) if !selected.is_empty() => outcomes = selected,
                _ => {
                    println!("{}", theme::yellow().paint("cancelled"));
                    return Ok(());
                }
            }
        } else {
            // offer method choice
            let methods = vec![
                "templates    — use saved outcome templates",
                "variables    — pick individual variables",
            ];

            let method = inquire::Select::new("Select outcomes from:", methods)
                .with_help_message("↑↓ navigate, Enter select, Esc cancel")
                .prompt_skippable()?;

            match method {
                Some(m) if m.starts_with("templates") => {
                    // pick from templates
                    match picker::browse_templates("Select outcome template:", &available_templates)? {
                        Some(tpl_name) => {
                            templates = Some(vec![tpl_name]);
                        }
                        None => {
                            println!("{}", theme::yellow().paint("cancelled"));
                            return Ok(());
                        }
                    }
                }
                Some(m) if m.starts_with("variables") => {
                    match picker::pick_outcomes()? {
                        Some(selected) if !selected.is_empty() => outcomes = selected,
                        _ => {
                            println!("{}", theme::yellow().paint("cancelled"));
                            return Ok(());
                        }
                    }
                }
                _ => {
                    println!("{}", theme::yellow().paint("cancelled"));
                    return Ok(());
                }
            }
        }
    }

    // step 4: show summary and confirm
    println!();
    println!("  {}", theme::peach().paint("Project Summary"));
    println!(
        "  {}",
        theme::overlay0().paint("─────────────────────────────────────────────")
    );
    println!(
        "  {} {}",
        theme::subtext0().paint("exposure:"),
        theme::text().paint(&exposure)
    );
    println!(
        "  {} {}",
        theme::subtext0().paint("baseline:"),
        theme::text().paint(&baseline)
    );

    // show outcomes (from direct args or templates)
    let outcome_display = if !outcomes.is_empty() {
        format_outcomes_list(&outcomes)
    } else if let Some(ref tpls) = templates {
        format!("from templates: {}", tpls.join(", "))
    } else {
        "none".to_string()
    };
    println!(
        "  {} {}",
        theme::subtext0().paint("outcomes:"),
        theme::text().paint(&outcome_display)
    );

    // show output directory
    let config = Config::load();
    let project_name = name.clone().unwrap_or_else(|| {
        let year = chrono_year();
        format!("{}-{}", year, exposure.replace('_', "-"))
    });
    if let Some(ref push_mods) = config.push_mods {
        println!(
            "  {} {}/{}",
            theme::subtext0().paint("output:"),
            theme::text().paint(shorten_path(push_mods)),
            theme::text().paint(&project_name)
        );
    }
    println!();

    // confirm before creating
    if !picker::confirm_create()? {
        println!("{}", theme::yellow().paint("cancelled"));
        return Ok(());
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
        &baseline,
        baseline_vars_override.as_deref(),
        name.as_deref(),
        &who_mode,
    )
}

fn format_outcomes_list(outcomes: &[String]) -> String {
    if outcomes.len() <= 3 {
        outcomes.join(", ")
    } else {
        format!(
            "{}, ... ({} total)",
            outcomes[..3].join(", "),
            outcomes.len()
        )
    }
}

fn chrono_year() -> String {
    // simple year extraction without chrono dependency
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // approximate year calculation (good enough for project naming)
    let years_since_1970 = secs / 31_536_000; // seconds per year
    format!("{}", 1970 + years_since_1970)
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

    // step 1: validate exposure FIRST if provided on command line
    let exposure = if let Some(e) = exposure {
        if VARIABLES.contains(&e.as_str()) {
            Some(e)
        } else {
            let matches: Vec<String> = fuzzy::search_variables(&e)
                .into_iter()
                .take(10)
                .map(String::from)
                .collect();

            if matches.is_empty() {
                println!(
                    "{} variable '{}' not found",
                    theme::red().paint("error:"),
                    e
                );
                return Ok(());
            }

            println!(
                "{} '{}' not found, showing similar:",
                theme::yellow().paint("note:"),
                e
            );

            match picker::pick_from_matches(&matches)? {
                Some(selected) => Some(selected),
                None => {
                    println!("{}", theme::yellow().paint("cancelled"));
                    return Ok(());
                }
            }
        }
    } else {
        None
    };

    // step 2: baseline template
    let baseline = if let Some(b) = baselines {
        b
    } else {
        let available = Config::list_baselines();
        if available.is_empty() {
            println!(
                "{}",
                theme::subtext0().paint("no baseline templates found, using default")
            );
            "default".to_string()
        } else {
            match picker::pick_baseline(&available)? {
                Some(selected) => selected,
                None => {
                    println!("{}", theme::yellow().paint("cancelled"));
                    return Ok(());
                }
            }
        }
    };

    // step 3: exposure picker (if not provided on command line)
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
    let outcome = if let Some(o) = outcome {
        // validate outcome
        if VARIABLES.contains(&o.as_str()) {
            Some(o)
        } else {
            let matches: Vec<String> = fuzzy::search_variables(&o)
                .into_iter()
                .take(10)
                .map(String::from)
                .collect();

            if matches.is_empty() {
                println!(
                    "{} outcome '{}' not found",
                    theme::red().paint("error:"),
                    o
                );
                return Ok(());
            }

            println!(
                "{} '{}' not found, showing similar:",
                theme::yellow().paint("note:"),
                o
            );

            match picker::pick_from_matches(&matches)? {
                Some(selected) => Some(selected),
                None => {
                    println!("{}", theme::yellow().paint("cancelled"));
                    return Ok(());
                }
            }
        }
    } else {
        match picker::pick_variable("Select outcome variable:")? {
            Some(selected) => Some(selected),
            None => {
                println!("{}", theme::yellow().paint("cancelled"));
                return Ok(());
            }
        }
    };

    // step 4: show summary and confirm
    println!();
    println!("  {}", theme::peach().paint("Project Summary"));
    println!(
        "  {}",
        theme::overlay0().paint("─────────────────────────────────────────────")
    );
    println!(
        "  {} {}",
        theme::subtext0().paint("type:"),
        theme::text().paint("grf-event (longitudinal)")
    );
    println!(
        "  {} {}",
        theme::subtext0().paint("exposure:"),
        theme::text().paint(&exposure)
    );
    println!(
        "  {} {}",
        theme::subtext0().paint("baseline:"),
        theme::text().paint(&baseline)
    );
    if let Some(ref o) = outcome {
        println!(
            "  {} {}",
            theme::subtext0().paint("outcome:"),
            theme::text().paint(o)
        );
    }
    if let Some(ref w) = waves {
        println!(
            "  {} {}",
            theme::subtext0().paint("waves:"),
            theme::text().paint(&w.join(", "))
        );
    }

    let config = Config::load();
    let project_name = name.clone().unwrap_or_else(|| {
        let year = chrono_year();
        format!("{}-{}-event", year, exposure.replace('_', "-"))
    });
    if let Some(ref push_mods) = config.push_mods {
        println!(
            "  {} {}/{}",
            theme::subtext0().paint("output:"),
            theme::text().paint(shorten_path(push_mods)),
            theme::text().paint(&project_name)
        );
    }
    println!();

    if !picker::confirm_create()? {
        println!("{}", theme::yellow().paint("cancelled"));
        return Ok(());
    }

    println!();

    init::grf_event_from_config(
        &exposure,
        outcome.as_deref(),
        waves.as_deref(),
        reference.as_deref(),
        &baseline,
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
    print_help_item("/templates edit <name>", "interactive variable picker");
    print_help_item("/templates open <name>", "open template in $EDITOR");
    print_help_item("/templates new <type> <name>", "create new template");
    print_help_item("/vars [pattern]", "fuzzy search variables");
    print_help_item("/view [name]", "browse templates and their variables");
    print_help_item("/save <type> <name>", "create new template from variable picker");
    print_help_item("/theme [light|dark]", "toggle or set theme");
    print_help_item("/e, /o [name]", "quick edit template in $EDITOR");
    print_help_item("/here, /pwd", "show current directory");
    print_help_item("/home, /~", "go home + refresh");
    print_help_item("/cd <path>", "change directory");
    print_help_item("/refresh, /r", "clear + show welcome");
    print_help_item("/quit, /q, q", "exit margo");
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

            let (path, kind) = if outcomes_path.exists() {
                (outcomes_path, "outcomes")
            } else if baselines_path.exists() {
                (baselines_path, "baselines")
            } else {
                println!(
                    "{} template not found: {}",
                    theme::red().paint("error:"),
                    theme::text().paint(name)
                );
                println!(
                    "  create with: {}",
                    theme::sapphire().paint(format!("/templates new outcomes {}", name))
                );
                return Ok(());
            };

            // load current vars
            let template = if kind == "outcomes" {
                Config::load_outcomes(name)
            } else {
                Config::load_baselines(name)
            };

            let current_vars = template.map(|t| t.vars).unwrap_or_default();

            // interactive edit
            println!();
            match picker::edit_template(name, &current_vars)? {
                Some(new_vars) => {
                    save_template(&path, &new_vars)?;
                    println!(
                        "{} saved {} variables to {}",
                        theme::green().paint("success:"),
                        new_vars.len(),
                        name
                    );
                }
                None => {
                    println!("{}", theme::yellow().paint("cancelled"));
                }
            }
            Ok(())
        }
        "open" => {
            if args.len() < 2 {
                println!(
                    "{} missing template name",
                    theme::red().paint("error:")
                );
                println!("  usage: /templates open <name>");
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

fn cmd_view(args: &[&str]) -> Result<()> {
    let name = args.first().copied();

    match name {
        // view specific template by name
        Some(template_name) => view_template(template_name),
        // interactive picker
        None => view_template_picker(),
    }
}

fn view_template(name: &str) -> Result<()> {
    // try outcomes first, then baselines
    let template = Config::load_outcomes(name).or_else(|| Config::load_baselines(name));

    match template {
        Some(t) => {
            println!();
            println!(
                "  {} ({} variables)",
                theme::sapphire().paint(name),
                theme::text().paint(t.vars.len().to_string())
            );
            println!(
                "  {}",
                theme::overlay0().paint("─────────────────────────────────────────────")
            );

            for var in &t.vars {
                println!(
                    "    {} {}",
                    theme::overlay0().paint("•"),
                    theme::teal().paint(var.as_str())
                );
            }
            println!();
            Ok(())
        }
        None => {
            println!();
            println!(
                "  {} template '{}' not found",
                theme::yellow().paint("warning:"),
                theme::sapphire().paint(name)
            );
            println!(
                "  {} use /templates to list available templates",
                theme::overlay0().paint("hint:")
            );
            println!();
            Ok(())
        }
    }
}

fn view_template_picker() -> Result<()> {
    // collect all templates with their variable counts
    let outcomes = Config::list_outcomes();
    let baselines = Config::list_baselines();

    if outcomes.is_empty() && baselines.is_empty() {
        println!();
        println!(
            "  {} no templates found",
            theme::yellow().paint("warning:")
        );
        println!(
            "  {} use /templates new <type> <name> to create one",
            theme::overlay0().paint("hint:")
        );
        println!();
        return Ok(());
    }

    // build list with type prefix and variable count
    let mut items: Vec<String> = Vec::new();

    for name in &outcomes {
        if let Some(t) = Config::load_outcomes(name) {
            items.push(format!("outcomes/{} ({} vars)", name, t.vars.len()));
        }
    }

    for name in &baselines {
        if let Some(t) = Config::load_baselines(name) {
            items.push(format!("baselines/{} ({} vars)", name, t.vars.len()));
        }
    }

    let selection = picker::browse_templates("Select template to view:", &items)?;

    if let Some(selected) = selection {
        // extract template name from "outcomes/name (N vars)" format
        if let Some(name) = selected.split('/').nth(1) {
            if let Some(name) = name.split_whitespace().next() {
                view_template(name)?;
            }
        }
    }

    Ok(())
}

fn cmd_save(args: &[&str]) -> Result<()> {
    // usage: /save <type> <name>
    // type: outcomes or baselines
    // name: template name (alphanumeric + underscore)

    if args.len() < 2 {
        print_save_usage();
        return Ok(());
    }

    let template_type = args[0];
    let name = args[1];

    // validate type
    if template_type != "outcomes" && template_type != "baselines" {
        println!();
        println!(
            "  {} invalid template type '{}'",
            theme::yellow().paint("warning:"),
            theme::text().paint(template_type)
        );
        println!(
            "  {} use 'outcomes' or 'baselines'",
            theme::overlay0().paint("hint:")
        );
        println!();
        return Ok(());
    }

    // validate name (alphanumeric + underscore)
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        println!();
        println!(
            "  {} invalid template name '{}'",
            theme::yellow().paint("warning:"),
            theme::text().paint(name)
        );
        println!(
            "  {} use only letters, numbers, and underscores",
            theme::overlay0().paint("hint:")
        );
        println!();
        return Ok(());
    }

    // check if template already exists
    let existing = if template_type == "outcomes" {
        Config::load_outcomes(name)
    } else {
        Config::load_baselines(name)
    };

    if existing.is_some() {
        println!();
        println!(
            "  {} template '{}/{}' already exists",
            theme::yellow().paint("warning:"),
            theme::overlay0().paint(template_type),
            theme::sapphire().paint(name)
        );
        println!(
            "  {} use /templates edit {} to modify it",
            theme::overlay0().paint("hint:"),
            name
        );
        println!();
        return Ok(());
    }

    // open variable picker for selection
    let prompt = format!("Select variables for '{}':", name);
    let selection = picker::pick_outcomes_for_save(&prompt)?;

    match selection {
        Some(vars) if !vars.is_empty() => {
            // build template content
            let content = format_template_toml(&vars);

            // determine path
            let dir = if template_type == "outcomes" {
                Config::outcomes_dir()
            } else {
                Config::baselines_dir()
            };

            // ensure directory exists
            if let Err(e) = fs::create_dir_all(&dir) {
                println!();
                println!(
                    "  {} failed to create directory: {}",
                    theme::red().paint("error:"),
                    e
                );
                println!();
                return Ok(());
            }

            // write template
            let path = dir.join(format!("{}.toml", name));
            if let Err(e) = fs::write(&path, content) {
                println!();
                println!(
                    "  {} failed to write template: {}",
                    theme::red().paint("error:"),
                    e
                );
                println!();
                return Ok(());
            }

            println!();
            println!(
                "  {} saved {} variables to {}/{}.toml",
                theme::green().paint("✓"),
                theme::text().paint(vars.len().to_string()),
                theme::overlay0().paint(template_type),
                theme::sapphire().paint(name)
            );
            println!();
        }
        _ => {
            println!();
            println!(
                "  {} no variables selected, template not created",
                theme::yellow().paint("cancelled:")
            );
            println!();
        }
    }

    Ok(())
}

fn print_save_usage() {
    println!();
    println!(
        "  {} /save <type> <name>",
        theme::peach().paint("Usage:")
    );
    println!();
    println!(
        "  {} template type ('outcomes' or 'baselines')",
        theme::overlay0().paint("<type>")
    );
    println!(
        "  {} template name (letters, numbers, underscores)",
        theme::overlay0().paint("<name>")
    );
    println!();
    println!(
        "  {} /save outcomes wellbeing",
        theme::subtext0().paint("Example:")
    );
    println!(
        "           /save baselines minimal",
        );
    println!();
}

fn format_template_toml(vars: &[String]) -> String {
    let mut content = String::from("# template created with /save\n\nvars = [\n");
    for var in vars {
        content.push_str(&format!("  \"{}\",\n", var));
    }
    content.push_str("]\n");
    content
}

fn cmd_vars(args: &[&str]) -> Result<()> {
    let pattern = args.first().copied().unwrap_or("");

    let matches = fuzzy::search_variables(pattern);

    if matches.is_empty() {
        println!();
        println!(
            "  {} no variables matching '{}'",
            theme::yellow().paint("warning:"),
            theme::sapphire().paint(pattern)
        );
        println!();
        return Ok(());
    }

    // use interactive picker for browsing
    let prompt = if pattern.is_empty() {
        format!("Browse variables ({} total):", matches.len())
    } else {
        format!("Variables matching '{}' ({} matches):", pattern, matches.len())
    };

    let _ = picker::browse_variables(&prompt, &matches)?;

    Ok(())
}

fn cmd_theme(args: &[&str]) -> Result<()> {
    let subcommand = args.first().copied().unwrap_or("");

    match subcommand {
        // toggle between light and dark
        "" | "toggle" => {
            theme::toggle_theme();
            let current = theme::current_theme();
            println!();
            println!(
                "  {} switched to {} theme",
                theme::green().paint("✓"),
                theme::sapphire().paint(current)
            );
            println!();
        }
        // set specific theme
        "light" | "latte" => {
            theme::set_theme("light");
            println!();
            println!(
                "  {} switched to {} theme",
                theme::green().paint("✓"),
                theme::sapphire().paint("light")
            );
            println!();
        }
        "dark" | "mocha" => {
            theme::set_theme("dark");
            println!();
            println!(
                "  {} switched to {} theme",
                theme::green().paint("✓"),
                theme::sapphire().paint("dark")
            );
            println!();
        }
        // show current theme
        "show" | "current" => {
            let current = theme::current_theme();
            println!();
            println!(
                "  {} {}",
                theme::peach().paint("Theme:"),
                theme::sapphire().paint(current)
            );
            println!();
        }
        _ => {
            println!();
            println!(
                "  {} /theme [toggle|light|dark|show]",
                theme::peach().paint("Usage:")
            );
            println!(
                "    {}  toggle between light and dark",
                theme::overlay0().paint("toggle")
            );
            println!(
                "    {}   catppuccin latte (light)",
                theme::overlay0().paint("light")
            );
            println!(
                "    {}    catppuccin mocha (dark)",
                theme::overlay0().paint("dark")
            );
            println!(
                "    {}    show current theme",
                theme::overlay0().paint("show")
            );
            println!();
        }
    }

    Ok(())
}

fn cmd_refresh() -> Result<()> {
    // clear screen and show welcome
    print!("\x1B[2J\x1B[1;1H");
    welcome::print_welcome();
    Ok(())
}

fn cmd_picker() -> Result<()> {
    // fuzzy command picker when user types just "/"
    let commands = vec![
        "help         — show all commands",
        "config       — show/edit configuration",
        "templates    — list templates",
        "view         — browse template variables",
        "save         — create new template",
        "vars         — browse variables",
        "theme        — toggle light/dark",
        "e            — edit template",
        "here         — show current directory",
        "home         — go home + refresh",
        "cd           — change directory",
        "refresh      — clear + show welcome",
        "quit         — exit margo",
    ];

    let result = inquire::Select::new("Command:", commands)
        .with_page_size(15)
        .with_help_message("↑↓ navigate, type to filter, Enter select, Esc cancel")
        .prompt_skippable()?;

    match result {
        Some(selected) => {
            // extract command name (first word before spaces/dash)
            let cmd = selected.split_whitespace().next().unwrap_or("");
            // recursively handle the selected command
            handle_slash(cmd)
        }
        None => Ok(()),
    }
}

fn cmd_quick_edit(args: &[&str]) -> Result<()> {
    // /e <name> or /o <name> - quick open template in editor
    if args.is_empty() {
        // no name given - show picker
        let outcomes = Config::list_outcomes();
        let baselines = Config::list_baselines();
        let mut all: Vec<String> = outcomes;
        all.extend(baselines);
        all.sort();
        all.dedup();

        if all.is_empty() {
            println!(
                "  {} no templates found",
                theme::yellow().paint("note:")
            );
            return Ok(());
        }

        match picker::browse_templates("Select template to edit:", &all)? {
            Some(name) => {
                // try outcomes first, then baselines
                let path = if Config::outcomes_dir().join(format!("{}.toml", name)).exists() {
                    Config::outcomes_dir().join(format!("{}.toml", name))
                } else {
                    Config::baselines_dir().join(format!("{}.toml", name))
                };
                open_in_editor(&path.to_string_lossy())?;
            }
            None => {}
        }
        return Ok(());
    }

    let name = args[0];

    // try outcomes first, then baselines
    let outcomes_path = Config::outcomes_dir().join(format!("{}.toml", name));
    let baselines_path = Config::baselines_dir().join(format!("{}.toml", name));

    if outcomes_path.exists() {
        open_in_editor(&outcomes_path.to_string_lossy())?;
    } else if baselines_path.exists() {
        open_in_editor(&baselines_path.to_string_lossy())?;
    } else {
        println!(
            "  {} template '{}' not found",
            theme::yellow().paint("note:"),
            name
        );
        println!(
            "  {} /save outcomes {} or /save baselines {}",
            theme::subtext0().paint("create with:"),
            name,
            name
        );
    }

    Ok(())
}

fn cmd_here() -> Result<()> {
    let cwd = env::current_dir()?;
    let display = shorten_path(&cwd.to_string_lossy());
    println!(
        "  {} {}",
        theme::subtext0().paint("cwd:"),
        theme::text().paint(&display)
    );
    Ok(())
}

fn cmd_home() -> Result<()> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("cannot find home directory"))?;
    env::set_current_dir(&home)?;
    // clear and show welcome with updated cwd
    print!("\x1B[2J\x1B[1;1H");
    welcome::print_welcome();
    Ok(())
}

fn cmd_cd(args: &[&str]) -> Result<()> {
    if args.is_empty() {
        // no args = go home
        return cmd_home();
    }

    let target = args[0];

    // expand ~ to home directory
    let path = if target.starts_with('~') {
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("cannot find home directory"))?;
        if target == "~" {
            home
        } else {
            home.join(&target[2..]) // skip "~/"
        }
    } else {
        std::path::PathBuf::from(target)
    };

    if !path.exists() {
        bail!("directory not found: {}", target);
    }

    if !path.is_dir() {
        bail!("not a directory: {}", target);
    }

    env::set_current_dir(&path)?;
    let display = shorten_path(&path.to_string_lossy());
    println!(
        "  {} {}",
        theme::teal().paint("changed to:"),
        theme::text().paint(&display)
    );
    Ok(())
}

fn shorten_path(path: &str) -> String {
    if let Some(home) = dirs::home_dir() {
        if let Some(home_str) = home.to_str() {
            if path.starts_with(home_str) {
                return format!("~{}", &path[home_str.len()..]);
            }
        }
    }
    path.to_string()
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

fn save_template(path: &std::path::Path, vars: &[String]) -> Result<()> {
    let mut content = String::from("# template variables\n\nvars = [\n");
    for var in vars {
        content.push_str(&format!("    \"{}\",\n", var));
    }
    content.push_str("]\n");
    fs::write(path, content)?;
    Ok(())
}
