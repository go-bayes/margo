// slash command handlers

use anyhow::{bail, Result};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use crate::commands::init;
use crate::config::Config;
use crate::theme;

use super::fuzzy;
use super::pathpicker;
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
        "templates" | "t" => cmd_templates(args),
        "view" => cmd_view(args),
        "save" => cmd_save(args),
        "vars" | "v" => cmd_vars(args),
        "measure" | "measures" | "m" => cmd_measure(args),
        "theme" | "th" => cmd_theme(args),
        "here" | "pwd" => cmd_here(),
        "home" | "~" => cmd_home(),
        "cd" => cmd_cd(args),
        "init" => {
            let mut full = String::from("init");
            if !args.is_empty() {
                full.push(' ');
                full.push_str(&args.join(" "));
            }
            handle_init(&full)
        }
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

fn measure_workspace_cell() -> &'static Mutex<Option<crate::data::MeasureWorkspace>> {
    static CELL: OnceLock<Mutex<Option<crate::data::MeasureWorkspace>>> = OnceLock::new();
    CELL.get_or_init(|| Mutex::new(None))
}

fn format_measure_file_format(format: crate::data::MeasureFileFormat) -> &'static str {
    match format {
        crate::data::MeasureFileFormat::BoilerplateUnifiedJson => "boilerplate_unified.json",
        crate::data::MeasureFileFormat::MeasuresDbJson => "measures_db.json",
        crate::data::MeasureFileFormat::MeasuresDbCsv => "measures_db.csv",
        crate::data::MeasureFileFormat::VariableMetadataTsv => "variable_metadata.tsv",
        crate::data::MeasureFileFormat::VariableMetadataCsv => "variable_metadata.csv",
        crate::data::MeasureFileFormat::Unknown => "unknown",
    }
}

/// handle an init command
pub fn handle_init(_cmd: &str) -> Result<()> {
    // always use guided menu for consistency
    println!();
    let model = match picker::pick_model()? {
        Some(m) => m,
        None => {
            println!("{}", theme::yellow().paint("cancelled"));
            return Ok(());
        }
    };

    match model.as_str() {
        "grf" => handle_init_grf(),
        "grf-event" => handle_init_grf_event(),
        "lmtp" => {
            println!(
                "{} LMTP template not yet implemented",
                theme::yellow().paint("warning:")
            );
            Ok(())
        }
        _ => Ok(()),
    }
}

fn handle_init_grf() -> Result<()> {
    // guided menu flow
    let mut outcomes: Vec<String> = Vec::new();
    let mut templates: Option<Vec<String>> = None;
    let name: Option<String> = None;

    println!();

    // step 1: baseline template
    let (baseline, baseline_vars_override) = loop {
        let available = Config::list_baselines();
        if available.is_empty() {
            println!(
                "{}",
                theme::subtext0().paint("no baseline templates found, using default")
            );
            break ("default".to_string(), None);
        }

        // offer choice: use template as-is, modify, pick custom, or preview
        let methods = vec![
            "template     — use saved baseline template",
            "modify       — edit template variables",
            "custom       — pick individual variables",
            "view         — preview baseline templates",
        ];

        let method = inquire::Select::new("Select baseline from:", methods)
            .with_help_message("↑↓ navigate, Enter select, Esc cancel")
            .prompt_skippable()?;

        match method {
            Some(m) if m.starts_with("template") => {
                let selected = loop {
                    match picker::pick_baseline(&available)? {
                        picker::BaselineSelection::Selected(selected) => break Some(selected),
                        picker::BaselineSelection::View => {
                            if let Some(selected) = view_baseline_picker()? {
                                break Some(selected);
                            }
                        }
                        picker::BaselineSelection::Cancelled => break None,
                    }
                };
                if let Some(selected) = selected {
                    break (selected, None);
                }
                println!("{}", theme::yellow().paint("cancelled"));
                return Ok(());
            }
            Some(m) if m.starts_with("modify") => {
                // pick template then edit its variables
                let tpl_name = loop {
                    match picker::pick_baseline(&available)? {
                        picker::BaselineSelection::Selected(selected) => break Some(selected),
                        picker::BaselineSelection::View => {
                            if let Some(selected) = view_baseline_picker()? {
                                break Some(selected);
                            }
                        }
                        picker::BaselineSelection::Cancelled => break None,
                    }
                };
                let Some(tpl_name) = tpl_name else {
                    println!("{}", theme::yellow().paint("cancelled"));
                    return Ok(());
                };
                // load template vars and let user modify
                let current_vars = Config::load_baselines(&tpl_name)
                    .map(|t| t.vars)
                    .unwrap_or_default();
                match picker::edit_template(&tpl_name, &current_vars)? {
                    Some(vars) => break (tpl_name, Some(vars)),
                    None => {
                        println!("{}", theme::yellow().paint("cancelled"));
                        continue;
                    }
                }
            }
            Some(m) if m.starts_with("custom") => {
                // pick individual variables
                match picker::pick_baseline_vars()? {
                    Some(vars) if !vars.is_empty() => {
                        break ("custom".to_string(), Some(vars));
                    }
                    _ => {
                        println!("{}", theme::yellow().paint("cancelled"));
                        return Ok(());
                    }
                }
            }
            Some(m) if m.starts_with("view") => {
                if let Some(selected) = view_baseline_picker()? {
                    break (selected, None);
                }
            }
            _ => {
                println!("{}", theme::yellow().paint("cancelled"));
                return Ok(());
            }
        }
    };

    // step 2: exposure picker
    let exposure = match picker::pick_exposure()? {
        Some(selected) => selected,
        None => {
            println!("{}", theme::yellow().paint("cancelled"));
            return Ok(());
        }
    };

    // step 3: outcome variables
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
            loop {
                let methods = vec![
                    "templates    — use saved outcome templates",
                    "modify       — edit template variables",
                    "variables    — pick individual variables",
                    "view         — preview outcome templates",
                ];

                let method = inquire::Select::new("Select outcomes from:", methods)
                    .with_help_message("↑↓ navigate, Enter select, Esc cancel")
                    .prompt_skippable()?;

                match method {
                    Some(m) if m.starts_with("templates") => {
                        // pick from templates
                        match picker::browse_templates(
                            "Select outcome template:",
                            &available_templates,
                        )? {
                            Some(tpl_name) => {
                                templates = Some(vec![tpl_name]);
                                break;
                            }
                            None => {
                                println!("{}", theme::yellow().paint("cancelled"));
                                return Ok(());
                            }
                        }
                    }
                    Some(m) if m.starts_with("modify") => {
                        let tpl_name = match picker::browse_templates(
                            "Select outcome template to edit:",
                            &available_templates,
                        )? {
                            Some(selected) => selected,
                            None => {
                                println!("{}", theme::yellow().paint("cancelled"));
                                return Ok(());
                            }
                        };
                        let current_vars = Config::load_outcomes(&tpl_name)
                            .map(|t| t.vars)
                            .unwrap_or_default();
                        match picker::edit_template(&tpl_name, &current_vars)? {
                            Some(vars) => {
                                outcomes = vars;
                                break;
                            }
                            None => {
                                println!("{}", theme::yellow().paint("cancelled"));
                                continue;
                            }
                        }
                    }
                    Some(m) if m.starts_with("variables") => {
                        match picker::pick_outcomes()? {
                            Some(selected) if !selected.is_empty() => {
                                outcomes = selected;
                                break;
                            }
                            _ => {
                                println!("{}", theme::yellow().paint("cancelled"));
                                return Ok(());
                            }
                        }
                    }
                    Some(m) if m.starts_with("view") => {
                        if let Some(selected) = view_outcome_picker()? {
                            templates = Some(vec![selected]);
                            break;
                        }
                    }
                    _ => {
                        println!("{}", theme::yellow().paint("cancelled"));
                        return Ok(());
                    }
                }
            }
        }
    }

    let mut removed_exposure = false;
    outcomes.retain(|outcome| {
        let keep = outcome != &exposure;
        if !keep {
            removed_exposure = true;
        }
        keep
    });
    if removed_exposure {
        println!(
            "  {} exposure '{}' removed from outcome variables",
            theme::yellow().paint("warning:"),
            theme::text().paint(&exposure)
        );
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

    // show project location (scripts go here)
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let scripts_path = cwd.join("src");
    let scripts_display = scripts_path.display().to_string();
    println!(
        "  {} {}",
        theme::subtext0().paint("scripts:"),
        theme::text().paint(shorten_path(&scripts_display))
    );

    // show output directory (model outputs go here)
    let config = Config::load();
    let project_name = grf_project_name(
        &exposure,
        &outcomes,
        templates.as_deref(),
        name.as_deref(),
    );
    let push_mods = config.resolve_push_mods_base(&cwd);
    let push_mods_display = push_mods.display().to_string();
    let output_path = push_mods.join(&project_name);
    println!(
        "  {} {}/{}",
        theme::subtext0().paint("output:"),
        theme::text().paint(shorten_path(&push_mods_display)),
        theme::text().paint(&project_name)
    );
    println!();

    // check for existing project files
    if !check_existing_files()? {
        return Ok(());
    }

    // optional review of selections before creating
    if !maybe_review_grf_details(
        &exposure,
        &baseline,
        baseline_vars_override.as_deref(),
        &outcomes,
        templates.as_deref(),
        &output_path.display().to_string(),
        &scripts_display,
    )? {
        println!("{}", theme::yellow().paint("cancelled"));
        return Ok(());
    }

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

fn grf_project_name(
    exposure: &str,
    outcomes: &[String],
    templates: Option<&[String]>,
    custom_name: Option<&str>,
) -> String {
    if let Some(name) = custom_name {
        return name.to_string();
    }
    if !outcomes.is_empty() {
        return format!("{}-{}", exposure, outcomes[0]);
    }
    if let Some(tpls) = templates {
        if !tpls.is_empty() {
            return format!("{}-{}", exposure, tpls.join("-"));
        }
    }
    exposure.to_string()
}

fn grf_event_project_name(exposure: &str, custom_name: Option<&str>) -> String {
    custom_name
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{}-event-study", exposure))
}

fn maybe_review_grf_details(
    exposure: &str,
    baseline: &str,
    baseline_vars_override: Option<&[String]>,
    outcomes: &[String],
    templates: Option<&[String]>,
    output_path: &str,
    scripts_path: &str,
) -> Result<bool> {
    let review = inquire::Confirm::new("Review template contents?")
        .with_default(false)
        .prompt_skippable()?;

    match review {
        Some(true) => {}
        Some(false) => return Ok(true),
        None => return Ok(false),
    }

    println!();
    println!("  {}", theme::peach().paint("Project Review"));
    println!(
        "  {}",
        theme::overlay0().paint("─────────────────────────────────────────────")
    );
    println!(
        "  {} {}",
        theme::subtext0().paint("scripts:"),
        theme::text().paint(shorten_path(scripts_path))
    );
    println!(
        "  {} {}",
        theme::subtext0().paint("output:"),
        theme::text().paint(shorten_path(output_path))
    );
    println!();

    if let Some(vars) = baseline_vars_override {
        let label = if baseline == "custom" {
            "Baseline variables (custom)".to_string()
        } else {
            format!("Baseline variables (override: {})", baseline)
        };
        print_vars_block(&label, vars);
    } else if let Some(tpl) = Config::load_baselines(baseline) {
        print_vars_block(&format!("Baseline template: {}", baseline), &tpl.vars);
    } else {
        println!(
            "  {} baseline template '{}' not found",
            theme::yellow().paint("warning:"),
            theme::text().paint(baseline)
        );
        println!();
    }

    println!(
        "  {} {}",
        theme::subtext0().paint("exposure:"),
        theme::text().paint(exposure)
    );
    println!();

    if !outcomes.is_empty() {
        print_vars_block("Outcome variables", outcomes);
    }

    if let Some(tpls) = templates {
        for name in tpls {
            if let Some(tpl) = Config::load_outcomes(name) {
                let mut vars = tpl.vars;
                vars.retain(|var| var != exposure);
                print_vars_block(&format!("Outcome template: {}", name), &vars);
            } else {
                println!(
                    "  {} outcome template '{}' not found",
                    theme::yellow().paint("warning:"),
                    theme::text().paint(name)
                );
                println!();
            }
        }
    }

    Ok(true)
}

fn maybe_review_grf_event_details(
    baseline: &str,
    outcome: Option<&str>,
    output_path: &str,
    scripts_path: &str,
) -> Result<bool> {
    let review = inquire::Confirm::new("Review template contents?")
        .with_default(false)
        .prompt_skippable()?;

    match review {
        Some(true) => {}
        Some(false) => return Ok(true),
        None => return Ok(false),
    }

    println!();
    println!("  {}", theme::peach().paint("Project Review"));
    println!(
        "  {}",
        theme::overlay0().paint("─────────────────────────────────────────────")
    );
    println!(
        "  {} {}",
        theme::subtext0().paint("scripts:"),
        theme::text().paint(shorten_path(scripts_path))
    );
    println!(
        "  {} {}",
        theme::subtext0().paint("output:"),
        theme::text().paint(shorten_path(output_path))
    );
    if let Some(value) = outcome {
        println!(
            "  {} {}",
            theme::subtext0().paint("outcome:"),
            theme::text().paint(value)
        );
    }
    println!();

    if let Some(tpl) = Config::load_baselines(baseline) {
        print_vars_block(&format!("Baseline template: {}", baseline), &tpl.vars);
    } else {
        println!(
            "  {} baseline template '{}' not found",
            theme::yellow().paint("warning:"),
            theme::text().paint(baseline)
        );
        println!();
    }

    Ok(true)
}

fn print_vars_block(title: &str, vars: &[String]) {
    println!(
        "  {} ({})",
        theme::sapphire().paint(title),
        theme::text().paint(format!("{} vars", vars.len()))
    );
    println!(
        "  {}",
        theme::overlay0().paint("─────────────────────────────────────────────")
    );

    if vars.is_empty() {
        println!(
            "    {} {}",
            theme::overlay0().paint("•"),
            theme::overlay0().paint("(none)")
        );
    } else {
        for var in vars {
            println!(
                "    {} {}",
                theme::overlay0().paint("•"),
                theme::teal().paint(var)
            );
        }
    }
    println!();
}

fn handle_init_grf_event() -> Result<()> {
    // guided menu flow
    println!();

    // step 1: baseline template
    let baseline = loop {
        let available = Config::list_baselines();
        if available.is_empty() {
            println!(
                "{}",
                theme::subtext0().paint("no baseline templates found, using default")
            );
            break "default".to_string();
        }

        let methods = vec![
            "template     — use saved baseline template",
            "view         — preview baseline templates",
        ];

        let method = inquire::Select::new("Select baseline from:", methods)
            .with_help_message("↑↓ navigate, Enter select, Esc cancel")
            .prompt_skippable()?;

        match method {
            Some(m) if m.starts_with("template") => {
                let selected = loop {
                    match picker::pick_baseline(&available)? {
                        picker::BaselineSelection::Selected(selected) => break Some(selected),
                        picker::BaselineSelection::View => {
                            if let Some(selected) = view_baseline_picker()? {
                                break Some(selected);
                            }
                        }
                        picker::BaselineSelection::Cancelled => break None,
                    }
                };
                if let Some(selected) = selected {
                    break selected;
                }
                println!("{}", theme::yellow().paint("cancelled"));
                return Ok(());
            }
            Some(m) if m.starts_with("view") => {
                if let Some(selected) = view_baseline_picker()? {
                    break selected;
                }
            }
            _ => {
                println!("{}", theme::yellow().paint("cancelled"));
                return Ok(());
            }
        }
    };

    // step 2: exposure picker
    let exposure = match picker::pick_exposure()? {
        Some(selected) => selected,
        None => {
            println!("{}", theme::yellow().paint("cancelled"));
            return Ok(());
        }
    };

    // step 3: outcome variable (optional for event study)
    let outcome = {
        let result = inquire::Confirm::new("Specify outcome variable?")
            .with_default(false)
            .prompt_skippable()?;

        if result == Some(true) {
            picker::pick_variable("Select outcome variable:")?
        } else {
            None
        }
    };

    // waves/reference/name use defaults (could be extended later)
    let waves: Option<Vec<String>> = None;
    let reference: Option<String> = None;
    let name: Option<String> = None;

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

    // show project location (scripts go here)
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let scripts_path = cwd.join("src");
    let scripts_display = scripts_path.display().to_string();
    println!(
        "  {} {}",
        theme::subtext0().paint("scripts:"),
        theme::text().paint(shorten_path(&scripts_display))
    );

    // show output directory
    let config = Config::load();
    let project_name = grf_event_project_name(&exposure, name.as_deref());
    let push_mods = config.resolve_push_mods_base(&cwd);
    let push_mods_display = push_mods.display().to_string();
    let output_path = push_mods.join(&project_name);
    println!(
        "  {} {}/{}",
        theme::subtext0().paint("output:"),
        theme::text().paint(shorten_path(&push_mods_display)),
        theme::text().paint(&project_name)
    );
    println!();

    // check for existing project files
    if !check_existing_files()? {
        return Ok(());
    }

    // optional review of selections before creating
    if !maybe_review_grf_event_details(
        &baseline,
        outcome.as_deref(),
        &output_path.display().to_string(),
        &scripts_display,
    )? {
        println!("{}", theme::yellow().paint("cancelled"));
        return Ok(());
    }

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
    print_help_item("/config data", "set data directory (pull_data)");
    print_help_item("/config output", "set output directory (push_mods)");
    print_help_item("/config setup", "set default paths (pull_data, push_mods)");
    print_help_item("/config reset", "reset config to defaults");
    print_help_item("/config init", "create default config");
    print_help_item("/init", "guided project setup");
    print_help_item("/templates, /t", "list all templates");
    print_help_item("/t outcomes", "list outcome templates");
    print_help_item("/t baselines", "list baseline templates");
    print_help_item("/t edit [name]", "interactive template editor");
    print_help_item("/t open <name>", "open template in $EDITOR");
    print_help_item("/t new <type> <name>", "create new template");
    print_help_item("/vars [pattern]", "fuzzy search variables");
    print_help_item("/measure <subcommand>", "manage measures data sources");
    print_help_item("/measure load [path]", "load measures file into workspace");
    print_help_item("/measure source", "show loaded measures source");
    print_help_item("/measure list [pattern]", "list measures in workspace");
    print_help_item("/measure show <name>", "show full measure details");
    print_help_item("/measure add <name>", "add a new measure record");
    print_help_item("/measure edit <name> <field> <value>", "edit one measure field");
    print_help_item("/measure rename <old> <new>", "rename a measure");
    print_help_item("/measure delete <name>", "delete a measure");
    print_help_item("/measure save [path]", "save workspace (or to a new path)");
    print_help_item("/measure diff", "show added/removed/changed measures");
    print_help_item("/measure validate", "run basic measures validation checks");
    print_help_item("/measure export-missing [field]", "list measures missing a field");
    print_help_item("/view [name]", "browse templates and their variables");
    print_help_item("/save <type> <name>", "create new template from variable picker");
    print_help_item("/theme, /th", "toggle or set theme");
    print_help_item("/e, /o [name]", "quick edit template or config in $EDITOR");
    print_help_item("/here, /pwd", "show current directory");
    print_help_item("/home, /~", "go home + refresh");
    print_help_item("/cd <path>", "change directory");
    print_help_item("/refresh, /r", "clear + show welcome");
    print_help_item("/q, /quit, q", "exit margo");
    println!();

    println!("  {}", theme::subtext1().paint("Init commands"));
    print_help_item("init, /init", "guided project setup");
    print_help_item("init grf", "create grf project");
    print_help_item("init grf-event", "create grf event study");
    println!();

    println!("  {}", theme::subtext1().paint("Keybindings"));
    print_help_item("Esc", "go home (/home)");
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
            println!(
                "  {}: {}",
                theme::subtext0().paint("edit"),
                format!(
                    "{} or {}",
                    theme::sapphire().paint("/config edit"),
                    theme::sapphire().paint("/e config")
                )
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
                "use_rv",
                if config.use_rv.unwrap_or(true) { "true" } else { "false" },
            );
            println!();

            Ok(())
        }
        "edit" => {
            Config::ensure_config_file()?;
            let config_path = Config::config_path();
            open_in_editor(&config_path.to_string_lossy())
        }
        "data" => cmd_config_data(),
        "output" => cmd_config_output(),
        "setup" => cmd_config_setup(),
        "reset" => cmd_config_reset(),
        "init" => cmd_config_init(),
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
            println!("  try: /config, /config edit, /config data, /config output, /config setup, /config reset, /config init, /config path");
            Ok(())
        }
    }
}

fn cmd_config_output() -> Result<()> {
    Config::ensure_config_file()?;

    let methods = vec![
        "choose directory — browse folders",
        "type path        — enter manually",
    ];

    let method = inquire::Select::new("Set output directory (push_mods):", methods)
        .with_help_message("↑↓ navigate, Enter select, Esc cancel")
        .prompt_skippable()?;

    let value = match method {
        Some(m) if m.starts_with("choose") => {
            match pathpicker::pick_output_directory("Select output directory:")? {
                pathpicker::PickerResult::Selected(path) => path.display().to_string(),
                pathpicker::PickerResult::Cancelled => {
                    println!("{}", theme::yellow().paint("cancelled"));
                    return Ok(());
                }
            }
        }
        Some(m) if m.starts_with("type") => {
            let config = Config::load();
            let cwd = std::env::current_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| ".".to_string());
            let default_value = config
                .push_mods
                .unwrap_or_else(|| format!("{}/outputs", cwd));
            let input = inquire::Text::new("Output directory:")
                .with_initial_value(&default_value)
                .with_help_message("absolute or relative path")
                .prompt_skippable()?;
            match input {
                Some(value) if !value.trim().is_empty() => value.trim().to_string(),
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
    };

    Config::set_config_value("paths", "push_mods", &value)?;
    println!(
        "{} output directory set to: {}",
        theme::green().paint("success:"),
        theme::text().paint(&value)
    );
    Ok(())
}

fn cmd_config_data() -> Result<()> {
    Config::ensure_config_file()?;

    let methods = vec![
        "choose directory — browse folders",
        "type path        — enter manually",
    ];

    let method = inquire::Select::new("Set data directory (pull_data):", methods)
        .with_help_message("↑↓ navigate, Enter select, Esc cancel")
        .prompt_skippable()?;

    let value = match method {
        Some(m) if m.starts_with("choose") => {
            let start = std::env::current_dir().ok();
            match pathpicker::pick_directory("Select data directory:", start.as_deref())? {
                pathpicker::PickerResult::Selected(path) => path.display().to_string(),
                pathpicker::PickerResult::Cancelled => {
                    println!("{}", theme::yellow().paint("cancelled"));
                    return Ok(());
                }
            }
        }
        Some(m) if m.starts_with("type") => {
            let config = Config::load();
            let cwd = std::env::current_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| ".".to_string());
            let default_value = config.pull_data.unwrap_or_else(|| cwd);
            let input = inquire::Text::new("Data directory:")
                .with_initial_value(&default_value)
                .with_help_message("absolute or relative path")
                .prompt_skippable()?;
            match input {
                Some(value) if !value.trim().is_empty() => value.trim().to_string(),
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
    };

    Config::set_config_value("paths", "pull_data", &value)?;
    println!(
        "{} data directory set to: {}",
        theme::green().paint("success:"),
        theme::text().paint(&value)
    );
    Ok(())
}

fn cmd_config_setup() -> Result<()> {
    Config::ensure_config_file()?;
    run_config_setup()
}

fn cmd_config_init() -> Result<()> {
    let config_path = Config::config_path();
    if config_path.exists() {
        println!(
            "{} config already exists at: {}",
            theme::yellow().paint("note:"),
            config_path.display()
        );
        println!("  edit with: {}", theme::sapphire().paint("/config edit"));
        return Ok(());
    }

    Config::ensure_config_file()?;
    println!(
        "{} created config at: {}",
        theme::green().paint("success:"),
        config_path.display()
    );

    let setup = inquire::Confirm::new("Set default paths now?")
        .with_default(true)
        .with_help_message("Esc to skip")
        .prompt_skippable()?;

    if matches!(setup, Some(true)) {
        run_config_setup()?;
    }

    Ok(())
}

fn cmd_config_reset() -> Result<()> {
    let config_path = Config::config_path();
    if config_path.exists() {
        let confirm = inquire::Confirm::new("Reset config to margo defaults?")
            .with_default(false)
            .with_help_message("This overwrites config.toml")
            .prompt_skippable()?;
        if !matches!(confirm, Some(true)) {
            println!("{}", theme::yellow().paint("cancelled"));
            return Ok(());
        }
    }

    Config::write_default_config()?;
    println!(
        "{} reset config at: {}",
        theme::green().paint("success:"),
        config_path.display()
    );
    Ok(())
}

fn run_config_setup() -> Result<()> {
    let config = Config::load();
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let cwd_display = cwd.display().to_string();

    println!();
    println!("  {}", theme::peach().paint("Default Paths"));
    println!(
        "  {}",
        theme::overlay0().paint("─────────────────────────────────────────────")
    );

    let pull_default = config.pull_data.unwrap_or_else(|| cwd_display.clone());
    if let Some(value) = prompt_path_value(
        "Default data directory (pull_data):",
        &pull_default,
        "Enter to accept, Esc to skip",
    )? {
        Config::set_config_value("paths", "pull_data", &value)?;
    }

    let push_default = config
        .push_mods
        .unwrap_or_else(|| format!("{}/outputs", cwd_display));
    if let Some(value) = prompt_path_value(
        "Default output directory (push_mods):",
        &push_default,
        "Project subfolders will be created here (Esc to skip)",
    )? {
        Config::set_config_value("paths", "push_mods", &value)?;
    }

    Ok(())
}

fn prompt_path_value(prompt: &str, default_value: &str, help: &str) -> Result<Option<String>> {
    let input = inquire::Text::new(prompt)
        .with_initial_value(default_value)
        .with_help_message(help)
        .prompt_skippable()?;

    match input {
        Some(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else {
                Ok(Some(trimmed.to_string()))
            }
        }
        None => Ok(None),
    }
}

pub(super) fn maybe_first_run_setup() -> Result<()> {
    let config_path = Config::config_path();
    if config_path.exists() {
        return Ok(());
    }

    println!();
    println!(
        "{} no config file found ({}).",
        theme::yellow().paint("note:"),
        theme::text().paint(config_path.display().to_string())
    );
    let setup = inquire::Confirm::new("Set default paths now?")
        .with_default(true)
        .with_help_message("You can run /config setup later")
        .prompt_skippable()?;

    if matches!(setup, Some(true)) {
        Config::ensure_config_file()?;
        run_config_setup()?;
    }

    Ok(())
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
            let (kind, name) = if args.len() < 2 {
                match pick_template_for_edit()? {
                    Some(selection) => selection,
                    None => return Ok(()),
                }
            } else {
                let name = args[1].to_string();
                let outcomes_path = Config::outcomes_dir().join(format!("{}.toml", name));
                let baselines_path = Config::baselines_dir().join(format!("{}.toml", name));

                if outcomes_path.exists() {
                    ("outcomes".to_string(), name)
                } else if baselines_path.exists() {
                    ("baselines".to_string(), name)
                } else {
                    println!(
                        "{} template not found: {}",
                        theme::red().paint("error:"),
                        theme::text().paint(&name)
                    );
                    println!(
                        "  create with: {}",
                        theme::sapphire().paint(format!("/templates new outcomes {}", name))
                    );
                    return Ok(());
                }
            };

            let _ = edit_template_vars(&kind, &name)?;
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
            println!("  try: /templates, /templates outcomes, /templates baselines, /templates edit [name]");
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

fn view_baseline_picker() -> Result<Option<String>> {
    loop {
        let baselines = Config::list_baselines();

        if baselines.is_empty() {
            println!();
            println!(
                "  {} no baseline templates found",
                theme::yellow().paint("warning:")
            );
            println!(
                "  {} use /templates new baselines <name> to create one",
                theme::overlay0().paint("hint:")
            );
            println!();
            return Ok(None);
        }

        let mut items: Vec<String> = Vec::new();
        for name in &baselines {
            let count = Config::load_baselines(name).map(|t| t.vars.len()).unwrap_or(0);
            items.push(format!("{} ({} vars)", name, count));
        }

        let selection = picker::browse_templates("Select baseline to view:", &items)?;
        let Some(selected) = selection else {
            return Ok(None);
        };
        let Some(name) = template_name_from_item(&selected) else {
            continue;
        };
        let name = name.to_string();
        view_baseline_template(&name)?;

        loop {
            let actions = vec![
                "use this template",
                "edit this template",
                "back to list",
            ];

            let action = inquire::Select::new("Template actions:", actions)
                .with_help_message("↑↓ navigate, Enter select, Esc cancel")
                .prompt_skippable()?;

            match action {
                Some(a) if a.starts_with("use") => return Ok(Some(name)),
                Some(a) if a.starts_with("edit") => {
                    if let Some(view_name) = edit_template_vars("baselines", &name)? {
                        view_baseline_template(&view_name)?;
                    }
                }
                Some(a) if a.starts_with("back") => break,
                _ => return Ok(None),
            }
        }
    }
}

fn view_outcome_picker() -> Result<Option<String>> {
    loop {
        let outcomes = Config::list_outcomes();

        if outcomes.is_empty() {
            println!();
            println!(
                "  {} no outcome templates found",
                theme::yellow().paint("warning:")
            );
            println!(
                "  {} use /templates new outcomes <name> to create one",
                theme::overlay0().paint("hint:")
            );
            println!();
            return Ok(None);
        }

        let mut items: Vec<String> = Vec::new();
        for name in &outcomes {
            let count = Config::load_outcomes(name).map(|t| t.vars.len()).unwrap_or(0);
            items.push(format!("{} ({} vars)", name, count));
        }

        let selection = picker::browse_templates("Select outcome template to view:", &items)?;
        let Some(selected) = selection else {
            return Ok(None);
        };
        let Some(name) = template_name_from_item(&selected) else {
            continue;
        };
        let name = name.to_string();
        view_outcome_template(&name)?;

        loop {
            let actions = vec![
                "use this template",
                "edit this template",
                "back to list",
            ];

            let action = inquire::Select::new("Template actions:", actions)
                .with_help_message("↑↓ navigate, Enter select, Esc cancel")
                .prompt_skippable()?;

            match action {
                Some(a) if a.starts_with("use") => return Ok(Some(name)),
                Some(a) if a.starts_with("edit") => {
                    if let Some(view_name) = edit_template_vars("outcomes", &name)? {
                        view_outcome_template(&view_name)?;
                    }
                }
                Some(a) if a.starts_with("back") => break,
                _ => return Ok(None),
            }
        }
    }
}

fn edit_template_vars(kind: &str, name: &str) -> Result<Option<String>> {
    let (path, template) = match kind {
        "outcomes" => (
            Config::outcomes_dir().join(format!("{}.toml", name)),
            Config::load_outcomes(name),
        ),
        "baselines" => (
            Config::baselines_dir().join(format!("{}.toml", name)),
            Config::load_baselines(name),
        ),
        _ => {
            println!(
                "{} unknown template kind: {}",
                theme::red().paint("error:"),
                theme::text().paint(kind)
            );
            return Ok(None);
        }
    };

    if !path.exists() {
        println!(
            "{} template not found: {}",
            theme::red().paint("error:"),
            theme::text().paint(name)
        );
        return Ok(None);
    }

    let current_vars = template.map(|t| t.vars).unwrap_or_default();

    println!();
    match picker::edit_template(name, &current_vars)? {
        Some(new_vars) => {
            let actions = vec![
                format!("overwrite {}", name),
                "save as new template".to_string(),
                "discard changes".to_string(),
            ];

            let action = inquire::Select::new("Save template changes:", actions)
                .with_help_message("↑↓ navigate, Enter select, Esc cancel")
                .prompt_skippable()?;

            match action {
                Some(a) if a.starts_with("overwrite") => {
                    save_template(&path, &new_vars)?;
                    println!(
                        "{} saved {} variables to {}",
                        theme::green().paint("success:"),
                        new_vars.len(),
                        name
                    );
                    Ok(Some(name.to_string()))
                }
                Some(a) if a.starts_with("save as new") => {
                    let new_name = match prompt_new_template_name(kind)? {
                        Some(value) => value,
                        None => {
                            println!("{}", theme::yellow().paint("cancelled"));
                            return Ok(None);
                        }
                    };
                    let new_path = match kind {
                        "outcomes" => Config::outcomes_dir().join(format!("{}.toml", new_name)),
                        _ => Config::baselines_dir().join(format!("{}.toml", new_name)),
                    };
                    save_template(&new_path, &new_vars)?;
                    println!(
                        "{} saved {} variables to {}",
                        theme::green().paint("success:"),
                        new_vars.len(),
                        new_name
                    );
                    Ok(Some(new_name))
                }
                Some(a) if a.starts_with("discard") => {
                    println!("{}", theme::yellow().paint("cancelled"));
                    Ok(None)
                }
                None => {
                    println!("{}", theme::yellow().paint("cancelled"));
                    Ok(None)
                }
                _ => Ok(None),
            }
        }
        None => {
            println!("{}", theme::yellow().paint("cancelled"));
            Ok(None)
        }
    }
}

fn prompt_new_template_name(kind: &str) -> Result<Option<String>> {
    loop {
        let result = inquire::Text::new("New template name:")
            .with_help_message("letters, numbers, underscores")
            .prompt_skippable()?;

        let Some(name) = result else {
            return Ok(None);
        };

        if name.trim().is_empty() {
            println!(
                "{} template name cannot be empty",
                theme::yellow().paint("warning:")
            );
            continue;
        }

        if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            println!(
                "{} use only letters, numbers, and underscores",
                theme::yellow().paint("warning:")
            );
            continue;
        }

        let path = match kind {
            "outcomes" => Config::outcomes_dir().join(format!("{}.toml", name)),
            _ => Config::baselines_dir().join(format!("{}.toml", name)),
        };
        if path.exists() {
            println!(
                "{} template already exists: {}",
                theme::yellow().paint("warning:"),
                theme::text().paint(&name)
            );
            continue;
        }

        return Ok(Some(name));
    }
}

fn view_baseline_template(name: &str) -> Result<()> {
    let template = Config::load_baselines(name);
    let path = Config::baselines_dir().join(format!("{}.toml", name));

    match template {
        Some(t) => {
            print_vars_block(&format!("Baseline template: {}", name), &t.vars);
        }
        None if path.exists() => {
            print_vars_block(&format!("Baseline template: {}", name), &[]);
        }
        None => {
            println!();
            println!(
                "  {} baseline template '{}' not found",
                theme::yellow().paint("warning:"),
                theme::sapphire().paint(name)
            );
            println!(
                "  {} use /templates to list available templates",
                theme::overlay0().paint("hint:")
            );
            println!();
        }
    }

    Ok(())
}

fn view_outcome_template(name: &str) -> Result<()> {
    let template = Config::load_outcomes(name);
    let path = Config::outcomes_dir().join(format!("{}.toml", name));

    match template {
        Some(t) => {
            print_vars_block(&format!("Outcome template: {}", name), &t.vars);
        }
        None if path.exists() => {
            print_vars_block(&format!("Outcome template: {}", name), &[]);
        }
        None => {
            println!();
            println!(
                "  {} outcome template '{}' not found",
                theme::yellow().paint("warning:"),
                theme::sapphire().paint(name)
            );
            println!(
                "  {} use /templates to list available templates",
                theme::overlay0().paint("hint:")
            );
            println!();
        }
    }

    Ok(())
}

fn template_name_from_item(item: &str) -> Option<&str> {
    item.split_whitespace().next()
}

fn pick_template_for_edit() -> Result<Option<(String, String)>> {
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
        return Ok(None);
    }

    let mut items: Vec<String> = Vec::new();
    for name in &outcomes {
        let count = Config::load_outcomes(name).map(|t| t.vars.len()).unwrap_or(0);
        items.push(format!("outcomes/{} ({} vars)", name, count));
    }
    for name in &baselines {
        let count = Config::load_baselines(name).map(|t| t.vars.len()).unwrap_or(0);
        items.push(format!("baselines/{} ({} vars)", name, count));
    }

    let selection = picker::browse_templates("Select template to edit:", &items)?;
    let Some(selected) = selection else {
        return Ok(None);
    };

    let Some(path) = template_name_from_item(&selected) else {
        return Ok(None);
    };
    let Some((kind, name)) = path.split_once('/') else {
        return Ok(None);
    };

    Ok(Some((kind.to_string(), name.to_string())))
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

    let mut selected_vars: Vec<String> = Vec::new();
    let mut browse_prompt = prompt;

    loop {
        match picker::browse_variables(&browse_prompt, &matches)? {
            Some(selected) => {
                if !selected_vars.contains(&selected) {
                    selected_vars.push(selected);
                }
                browse_prompt = "Select another variable (Esc to finish):".to_string();
            }
            None => break,
        }
    }

    if !selected_vars.is_empty() {
        print_variable_details(&selected_vars);
    }

    Ok(())
}

fn cmd_measure(args: &[&str]) -> Result<()> {
    let subcommand = args.first().copied().unwrap_or("");
    match subcommand {
        "" => {
            println!();
            println!("  {}", theme::peach().paint("Measure Commands"));
            println!(
                "  {}",
                theme::overlay0().paint("─────────────────────────────────────────────")
            );
            println!(
                "    {} {}",
                theme::sapphire().paint("/measure load [path]"),
                theme::subtext0().paint("load measures file")
            );
            println!(
                "    {} {}",
                theme::sapphire().paint("/measure source"),
                theme::subtext0().paint("show current workspace source")
            );
            println!(
                "    {} {}",
                theme::sapphire().paint("/measure list [pattern]"),
                theme::subtext0().paint("list loaded measures")
            );
            println!(
                "    {} {}",
                theme::sapphire().paint("/measure show <name>"),
                theme::subtext0().paint("show full details for one measure")
            );
            println!(
                "    {} {}",
                theme::sapphire().paint("/measure add <name>"),
                theme::subtext0().paint("add a new measure record")
            );
            println!(
                "    {} {}",
                theme::sapphire().paint("/measure edit <name> <field> <value>"),
                theme::subtext0().paint("edit one field on a measure")
            );
            println!(
                "    {} {}",
                theme::sapphire().paint("/measure rename <old> <new>"),
                theme::subtext0().paint("rename a measure")
            );
            println!(
                "    {} {}",
                theme::sapphire().paint("/measure delete <name>"),
                theme::subtext0().paint("delete a measure")
            );
            println!(
                "    {} {}",
                theme::sapphire().paint("/measure save [path]"),
                theme::subtext0().paint("save current workspace")
            );
            println!(
                "    {} {}",
                theme::sapphire().paint("/measure diff"),
                theme::subtext0().paint("show in-session changes")
            );
            println!(
                "    {} {}",
                theme::sapphire().paint("/measure validate"),
                theme::subtext0().paint("check duplicates and missing descriptions")
            );
            println!(
                "    {} {}",
                theme::sapphire().paint("/measure export-missing [field]"),
                theme::subtext0().paint("list records missing a field")
            );
            println!();
            Ok(())
        }
        "load" => cmd_measure_load(&args[1..]),
        "source" => cmd_measure_source(),
        "list" => cmd_measure_list(&args[1..]),
        "show" => cmd_measure_show(&args[1..]),
        "add" => cmd_measure_add(&args[1..]),
        "edit" => cmd_measure_edit(&args[1..]),
        "rename" => cmd_measure_rename(&args[1..]),
        "delete" | "rm" | "del" => cmd_measure_delete(&args[1..]),
        "save" => cmd_measure_save(&args[1..]),
        "diff" => cmd_measure_diff(),
        "validate" => cmd_measure_validate(),
        "export-missing" => cmd_measure_export_missing(&args[1..]),
        _ => {
            println!(
                "{} unknown measure subcommand: {}",
                theme::yellow().paint("warning:"),
                theme::text().paint(subcommand)
            );
            println!(
                "  try: {}",
                theme::sapphire().paint("/measure load|source|list|show|add|edit|rename|delete|save|diff|validate|export-missing")
            );
            Ok(())
        }
    }
}

fn cmd_measure_load(args: &[&str]) -> Result<()> {
    let path = if args.is_empty() {
        auto_discover_measure_path().ok_or_else(|| {
            anyhow::anyhow!(
                "no measures file found. pass a path: /measure load <path>"
            )
        })?
    } else {
        PathBuf::from(args.join(" "))
    };

    let workspace = crate::data::MeasureWorkspace::load(&path)?;
    let count = workspace.record_count();
    let source = workspace.source().cloned();

    let mut guard = measure_workspace_cell()
        .lock()
        .map_err(|_| anyhow::anyhow!("measure workspace lock poisoned"))?;
    *guard = Some(workspace);

    println!();
    println!(
        "  {} loaded {} measure{}",
        theme::green().paint("✓"),
        theme::text().paint(count.to_string()),
        if count == 1 { "" } else { "s" }
    );
    if let Some(src) = source {
        println!(
            "  {} {} ({})",
            theme::subtext0().paint("source:"),
            theme::overlay0().paint(src.path.display().to_string()),
            theme::overlay0().paint(format_measure_file_format(src.format))
        );
    }
    println!();

    Ok(())
}

fn cmd_measure_source() -> Result<()> {
    let guard = measure_workspace_cell()
        .lock()
        .map_err(|_| anyhow::anyhow!("measure workspace lock poisoned"))?;
    let Some(workspace) = guard.as_ref() else {
        println!(
            "  {} no measures workspace loaded",
            theme::yellow().paint("note:")
        );
        println!(
            "    {}",
            theme::overlay0().paint("use /measure load <path>")
        );
        return Ok(());
    };

    println!();
    println!("  {}", theme::peach().paint("Measures Source"));
    println!(
        "  {}",
        theme::overlay0().paint("─────────────────────────────────────────────")
    );
    if let Some(source) = workspace.source() {
        println!(
            "  {} {}",
            theme::subtext0().paint("path:"),
            theme::text().paint(source.path.display().to_string())
        );
        println!(
            "  {} {}",
            theme::subtext0().paint("format:"),
            theme::text().paint(format_measure_file_format(source.format))
        );
    } else {
        println!(
            "  {} {}",
            theme::subtext0().paint("path:"),
            theme::overlay0().paint("(none)")
        );
    }
    println!(
        "  {} {}",
        theme::subtext0().paint("records:"),
        theme::text().paint(workspace.record_count().to_string())
    );
    println!(
        "  {} {}",
        theme::subtext0().paint("dirty:"),
        theme::text().paint(if workspace.is_dirty() { "true" } else { "false" })
    );
    println!();

    Ok(())
}

fn cmd_measure_list(args: &[&str]) -> Result<()> {
    let pattern = if args.is_empty() {
        None
    } else {
        Some(args.join(" "))
    };

    let guard = measure_workspace_cell()
        .lock()
        .map_err(|_| anyhow::anyhow!("measure workspace lock poisoned"))?;
    let Some(workspace) = guard.as_ref() else {
        println!(
            "  {} no measures workspace loaded",
            theme::yellow().paint("note:")
        );
        println!(
            "    {}",
            theme::overlay0().paint("use /measure load <path>")
        );
        return Ok(());
    };

    let records = workspace.list(pattern.as_deref());
    println!();
    println!(
        "  {} {} match{}",
        theme::peach().paint("Measures"),
        theme::text().paint(records.len().to_string()),
        if records.len() == 1 { "" } else { "es" }
    );
    println!(
        "  {}",
        theme::overlay0().paint("─────────────────────────────────────────────")
    );
    if records.is_empty() {
        println!("  {}", theme::overlay0().paint("(no matches)"));
        println!();
        return Ok(());
    }

    for record in records {
        let desc = record
            .description
            .as_deref()
            .unwrap_or("description not set");
        println!(
            "  {} {}",
            theme::teal().paint(&record.name),
            theme::overlay0().paint(short_measure_description(desc))
        );
    }
    println!();

    Ok(())
}

fn cmd_measure_show(args: &[&str]) -> Result<()> {
    if args.is_empty() {
        println!(
            "{} missing measure name",
            theme::red().paint("error:")
        );
        println!(
            "  usage: {}",
            theme::sapphire().paint("/measure show <name>")
        );
        return Ok(());
    }

    let name = args.join(" ");
    let guard = measure_workspace_cell()
        .lock()
        .map_err(|_| anyhow::anyhow!("measure workspace lock poisoned"))?;
    let Some(workspace) = guard.as_ref() else {
        println!(
            "  {} no measures workspace loaded",
            theme::yellow().paint("note:")
        );
        println!(
            "    {}",
            theme::overlay0().paint("use /measure load <path>")
        );
        return Ok(());
    };

    let Some(record) = workspace.get(&name) else {
        println!(
            "{} measure not found: {}",
            theme::yellow().paint("warning:"),
            theme::text().paint(&name)
        );
        return Ok(());
    };

    println!();
    println!(
        "  {} {}",
        theme::peach().paint("Measure"),
        theme::teal().paint(&record.name)
    );
    println!(
        "  {}",
        theme::overlay0().paint("─────────────────────────────────────────────")
    );
    print_measure_field("description", record.description.as_deref());
    print_measure_field("reference", record.reference.as_deref());
    print_measure_field("waves", record.waves.as_deref());
    print_measure_field("keywords", record.keywords.as_deref());
    print_measure_field("label", record.label.as_deref());
    print_measure_field("scale", record.scale.as_deref());
    print_measure_field("notes", record.notes.as_deref());
    print_measure_field("standardised_date", record.standardised_date.as_deref());
    println!(
        "  {} {}",
        theme::subtext0().paint("standardised:"),
        theme::text().paint(
            record
                .standardised
                .map(|value| if value { "true" } else { "false" })
                .unwrap_or("(none)")
        )
    );
    println!(
        "  {} {}",
        theme::subtext0().paint("items:"),
        theme::text().paint(record.items.len().to_string())
    );
    for item in &record.items {
        println!(
            "    {} {}",
            theme::overlay0().paint("•"),
            theme::text().paint(item)
        );
    }
    if !record.passthrough.is_empty() {
        let keys = record
            .passthrough
            .keys()
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        println!(
            "  {} {}",
            theme::subtext0().paint("passthrough keys:"),
            theme::overlay0().paint(keys)
        );
    }
    println!();

    Ok(())
}

fn cmd_measure_validate() -> Result<()> {
    let guard = measure_workspace_cell()
        .lock()
        .map_err(|_| anyhow::anyhow!("measure workspace lock poisoned"))?;
    let Some(workspace) = guard.as_ref() else {
        println!(
            "  {} no measures workspace loaded",
            theme::yellow().paint("note:")
        );
        println!(
            "    {}",
            theme::overlay0().paint("use /measure load <path>")
        );
        return Ok(());
    };

    let report = workspace.validate_basic();
    println!();
    println!("  {}", theme::peach().paint("Measure Validation"));
    println!(
        "  {}",
        theme::overlay0().paint("─────────────────────────────────────────────")
    );
    println!(
        "  {} {}",
        theme::subtext0().paint("duplicate names:"),
        theme::text().paint(report.duplicate_names.len().to_string())
    );
    println!(
        "  {} {}",
        theme::subtext0().paint("missing description:"),
        theme::text().paint(report.missing_description.len().to_string())
    );

    if !report.duplicate_names.is_empty() {
        println!(
            "  {}",
            theme::subtext1().paint("duplicates")
        );
        for name in &report.duplicate_names {
            println!(
                "    {} {}",
                theme::overlay0().paint("•"),
                theme::teal().paint(name)
            );
        }
    }

    if !report.missing_description.is_empty() {
        println!(
            "  {}",
            theme::subtext1().paint("missing description")
        );
        for name in &report.missing_description {
            println!(
                "    {} {}",
                theme::overlay0().paint("•"),
                theme::teal().paint(name)
            );
        }
    }
    println!();

    Ok(())
}

fn cmd_measure_add(args: &[&str]) -> Result<()> {
    if args.is_empty() {
        println!(
            "{} missing measure name",
            theme::red().paint("error:")
        );
        println!(
            "  usage: {}",
            theme::sapphire().paint("/measure add <name>")
        );
        return Ok(());
    }
    let name = args.join(" ");

    let mut guard = measure_workspace_cell()
        .lock()
        .map_err(|_| anyhow::anyhow!("measure workspace lock poisoned"))?;
    let Some(workspace) = guard.as_mut() else {
        println!(
            "  {} no measures workspace loaded",
            theme::yellow().paint("note:")
        );
        println!(
            "    {}",
            theme::overlay0().paint("use /measure load <path>")
        );
        return Ok(());
    };

    workspace.add(&name)?;
    println!();
    println!(
        "  {} added {}",
        theme::green().paint("✓"),
        theme::teal().paint(name)
    );
    println!(
        "  {} {}",
        theme::subtext0().paint("records:"),
        theme::text().paint(workspace.record_count().to_string())
    );
    println!();
    Ok(())
}

fn cmd_measure_edit(args: &[&str]) -> Result<()> {
    if args.len() < 3 {
        println!(
            "{} missing arguments",
            theme::red().paint("error:")
        );
        println!(
            "  usage: {}",
            theme::sapphire().paint("/measure edit <name> <field> <value>")
        );
        return Ok(());
    }

    let name = args[0];
    let field = args[1];
    let value = args[2..].join(" ");

    let mut guard = measure_workspace_cell()
        .lock()
        .map_err(|_| anyhow::anyhow!("measure workspace lock poisoned"))?;
    let Some(workspace) = guard.as_mut() else {
        println!(
            "  {} no measures workspace loaded",
            theme::yellow().paint("note:")
        );
        println!(
            "    {}",
            theme::overlay0().paint("use /measure load <path>")
        );
        return Ok(());
    };

    workspace.edit_field(name, field, &value)?;
    println!();
    println!(
        "  {} updated {}.{}",
        theme::green().paint("✓"),
        theme::teal().paint(name),
        theme::teal().paint(field)
    );
    println!();
    Ok(())
}

fn cmd_measure_rename(args: &[&str]) -> Result<()> {
    if args.len() < 2 {
        println!(
            "{} missing arguments",
            theme::red().paint("error:")
        );
        println!(
            "  usage: {}",
            theme::sapphire().paint("/measure rename <old> <new>")
        );
        return Ok(());
    }

    let old_name = args[0];
    let new_name = args[1];

    let mut guard = measure_workspace_cell()
        .lock()
        .map_err(|_| anyhow::anyhow!("measure workspace lock poisoned"))?;
    let Some(workspace) = guard.as_mut() else {
        println!(
            "  {} no measures workspace loaded",
            theme::yellow().paint("note:")
        );
        println!(
            "    {}",
            theme::overlay0().paint("use /measure load <path>")
        );
        return Ok(());
    };

    workspace.rename(old_name, new_name)?;
    println!();
    println!(
        "  {} renamed {} -> {}",
        theme::green().paint("✓"),
        theme::teal().paint(old_name),
        theme::teal().paint(new_name)
    );
    println!();
    Ok(())
}

fn cmd_measure_delete(args: &[&str]) -> Result<()> {
    if args.is_empty() {
        println!(
            "{} missing measure name",
            theme::red().paint("error:")
        );
        println!(
            "  usage: {}",
            theme::sapphire().paint("/measure delete <name>")
        );
        return Ok(());
    }

    let name = args.join(" ");

    let mut guard = measure_workspace_cell()
        .lock()
        .map_err(|_| anyhow::anyhow!("measure workspace lock poisoned"))?;
    let Some(workspace) = guard.as_mut() else {
        println!(
            "  {} no measures workspace loaded",
            theme::yellow().paint("note:")
        );
        println!(
            "    {}",
            theme::overlay0().paint("use /measure load <path>")
        );
        return Ok(());
    };

    let removed = workspace.delete(&name);
    println!();
    if removed {
        println!(
            "  {} deleted {}",
            theme::green().paint("✓"),
            theme::teal().paint(name)
        );
        println!(
            "  {} {}",
            theme::subtext0().paint("records:"),
            theme::text().paint(workspace.record_count().to_string())
        );
    } else {
        println!(
            "  {} not found: {}",
            theme::yellow().paint("warning:"),
            theme::text().paint(name)
        );
    }
    println!();
    Ok(())
}

fn cmd_measure_save(args: &[&str]) -> Result<()> {
    let target_path = if args.is_empty() {
        None
    } else {
        Some(PathBuf::from(args.join(" ")))
    };

    let mut guard = measure_workspace_cell()
        .lock()
        .map_err(|_| anyhow::anyhow!("measure workspace lock poisoned"))?;
    let Some(workspace) = guard.as_mut() else {
        println!(
            "  {} no measures workspace loaded",
            theme::yellow().paint("note:")
        );
        println!(
            "    {}",
            theme::overlay0().paint("use /measure load <path>")
        );
        return Ok(());
    };

    let source = workspace.save(target_path.as_deref())?;
    println!();
    println!(
        "  {} saved measures workspace",
        theme::green().paint("✓")
    );
    println!(
        "  {} {} ({})",
        theme::subtext0().paint("path:"),
        theme::text().paint(source.path.display().to_string()),
        theme::overlay0().paint(format_measure_file_format(source.format))
    );
    println!(
        "  {} {}",
        theme::subtext0().paint("dirty:"),
        theme::text().paint("false")
    );
    println!();

    Ok(())
}

fn cmd_measure_diff() -> Result<()> {
    let guard = measure_workspace_cell()
        .lock()
        .map_err(|_| anyhow::anyhow!("measure workspace lock poisoned"))?;
    let Some(workspace) = guard.as_ref() else {
        println!(
            "  {} no measures workspace loaded",
            theme::yellow().paint("note:")
        );
        println!(
            "    {}",
            theme::overlay0().paint("use /measure load <path>")
        );
        return Ok(());
    };

    let diff = workspace.diff_summary();
    println!();
    println!("  {}", theme::peach().paint("Measure Diff"));
    println!(
        "  {}",
        theme::overlay0().paint("─────────────────────────────────────────────")
    );
    println!(
        "  {} {}",
        theme::subtext0().paint("added:"),
        theme::text().paint(diff.added.len().to_string())
    );
    println!(
        "  {} {}",
        theme::subtext0().paint("removed:"),
        theme::text().paint(diff.removed.len().to_string())
    );
    println!(
        "  {} {}",
        theme::subtext0().paint("changed:"),
        theme::text().paint(diff.changed.len().to_string())
    );

    if diff.added.is_empty() && diff.removed.is_empty() && diff.changed.is_empty() {
        println!("  {}", theme::overlay0().paint("(no changes)"));
        println!();
        return Ok(());
    }

    if !diff.added.is_empty() {
        println!("  {}", theme::subtext1().paint("added"));
        for name in &diff.added {
            println!(
                "    {} {}",
                theme::overlay0().paint("•"),
                theme::teal().paint(name)
            );
        }
    }
    if !diff.removed.is_empty() {
        println!("  {}", theme::subtext1().paint("removed"));
        for name in &diff.removed {
            println!(
                "    {} {}",
                theme::overlay0().paint("•"),
                theme::teal().paint(name)
            );
        }
    }
    if !diff.changed.is_empty() {
        println!("  {}", theme::subtext1().paint("changed"));
        for name in &diff.changed {
            println!(
                "    {} {}",
                theme::overlay0().paint("•"),
                theme::teal().paint(name)
            );
        }
    }
    println!();
    Ok(())
}

fn cmd_measure_export_missing(args: &[&str]) -> Result<()> {
    let field = args.first().copied().unwrap_or("description");
    let guard = measure_workspace_cell()
        .lock()
        .map_err(|_| anyhow::anyhow!("measure workspace lock poisoned"))?;
    let Some(workspace) = guard.as_ref() else {
        println!(
            "  {} no measures workspace loaded",
            theme::yellow().paint("note:")
        );
        println!(
            "    {}",
            theme::overlay0().paint("use /measure load <path>")
        );
        return Ok(());
    };

    let missing = workspace.export_missing(field);
    println!();
    println!(
        "  {} {} {}",
        theme::peach().paint("Missing"),
        theme::teal().paint(field),
        theme::peach().paint("values")
    );
    println!(
        "  {}",
        theme::overlay0().paint("─────────────────────────────────────────────")
    );
    println!(
        "  {} {}",
        theme::subtext0().paint("count:"),
        theme::text().paint(missing.len().to_string())
    );

    for name in &missing {
        println!(
            "    {} {}",
            theme::overlay0().paint("•"),
            theme::teal().paint(name)
        );
    }
    println!();

    Ok(())
}

fn print_measure_field(label: &str, value: Option<&str>) {
    println!(
        "  {} {}",
        theme::subtext0().paint(format!("{label}:")),
        match value {
            Some(text) if !text.trim().is_empty() => theme::text().paint(text.to_string()),
            _ => theme::overlay0().paint("(none)"),
        }
    );
}

fn short_measure_description(value: &str) -> String {
    const LIMIT: usize = 72;
    let trimmed = value.trim();
    if trimmed.chars().count() <= LIMIT {
        return trimmed.to_string();
    }
    let mut out = String::new();
    for (idx, ch) in trimmed.chars().enumerate() {
        if idx >= LIMIT {
            break;
        }
        out.push(ch);
    }
    out.push('…');
    out
}

fn auto_discover_measure_path() -> Option<PathBuf> {
    let cwd = env::current_dir().ok()?;
    let candidates = vec![
        cwd.join("boilerplate_unified.json"),
        cwd.join("measures_db.json"),
        cwd.join("storage/variable_metadata.tsv"),
        cwd.join("storage/variable_metadata.csv"),
        cwd.join("storage/variables.tsv"),
        cwd.join("storage/variables.csv"),
        cwd.join("../bptui/boilerplate_unified.json"),
        cwd.join("../boilerplate/boilerplate_unified.json"),
    ];
    candidates.into_iter().find(|path| path.exists())
}

fn print_variable_details(vars: &[String]) {
    let metadata_source = crate::data::variable_metadata_source();

    println!();
    println!(
        "  {} selected {} variable{}",
        theme::green().paint("✓"),
        theme::text().paint(vars.len().to_string()),
        if vars.len() == 1 { "" } else { "s" }
    );
    println!();

    if let Some(ref source) = metadata_source {
        println!(
            "  {} {}",
            theme::subtext0().paint("metadata:"),
            theme::overlay0().paint(source)
        );
        println!();
    }

    for (idx, var) in vars.iter().enumerate() {
        let mut tokens = var.split('_');
        let group = tokens.next().unwrap_or(var.as_str());
        let token_count = 1 + tokens.count();
        let description = crate::data::lookup_variable_description(var);

        println!(
            "  {} {}",
            theme::overlay0().paint(format!("{:>3}.", idx + 1)),
            theme::teal().paint(var.as_str())
        );
        println!(
            "      {} {}   {} {}",
            theme::subtext0().paint("group:"),
            theme::text().paint(group),
            theme::subtext0().paint("tokens:"),
            theme::text().paint(token_count.to_string())
        );
        println!(
            "      {} {}",
            theme::subtext0().paint("description:"),
            match description {
                Some(text) => theme::text().paint(text),
                None => theme::overlay0().paint("not available"),
            }
        );
    }

    if metadata_source.is_none() {
        println!(
            "  {} {}",
            theme::subtext0().paint("tip:"),
            theme::overlay0().paint(
                "add storage/variable_metadata.tsv (name<TAB>description) or set MARGO_VAR_METADATA, MARGO_BOILERPLATE_METADATA, or MARGO_BPTUI_METADATA"
            )
        );
        println!();
    }

    println!();
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
        "init (/init) — guided project setup",
        "templates    — list templates",
        "view         — browse template variables",
        "save         — create new template",
        "vars         — browse and inspect variables",
        "measure      — load/list/edit/save/diff/validate measures",
        "theme        — toggle light/dark",
        "e            — edit template or config",
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

        let mut choices = Vec::with_capacity(all.len() + 1);
        choices.push("/config".to_string());
        choices.extend(all);

        match picker::browse_templates("Select template or config to edit:", &choices)? {
            Some(name) => {
                if name == "/config" {
                    Config::ensure_config_file()?;
                    let config_path = Config::config_path();
                    open_in_editor(&config_path.to_string_lossy())?;
                    return Ok(());
                }
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
    if name == "config" || name == "config.toml" {
        Config::ensure_config_file()?;
        let config_path = Config::config_path();
        open_in_editor(&config_path.to_string_lossy())?;
        return Ok(());
    }

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

/// check for existing project files and warn user
/// returns true if ok to proceed, false if cancelled
fn check_existing_files() -> Result<bool> {
    use std::path::Path;

    let project_files = [
        "study.toml",
        "src/00-setup.R",
        "src/01-data-prep.R",
        "src/02-wide-format.R",
        "src/03-causal-forest.R",
        "src/04-heterogeneity.R",
        "src/04-trajectory-plot.R",
        "src/05-policy-tree.R",
        "src/05-heterogeneity.R",
        "src/06-positivity.R",
        "src/07-tables.R",
        "src/08-plots.R",
    ];

    let study_toml_exists = Path::new("study.toml").exists();
    let existing_files: Vec<&str> = project_files
        .iter()
        .filter(|f| Path::new(f).exists())
        .copied()
        .collect();

    if study_toml_exists {
        // full project exists
        println!(
            "  {} Project already exists in this directory",
            theme::yellow().paint("⚠")
        );
        println!(
            "    {} found",
            theme::overlay0().paint("study.toml")
        );
        println!();

        let result = inquire::Confirm::new("Overwrite existing project?")
            .with_default(false)
            .prompt_skippable()?;

        if result != Some(true) {
            println!("{}", theme::yellow().paint("cancelled"));
            return Ok(false);
        }
    } else if !existing_files.is_empty() {
        // some files exist but no study.toml - unusual state
        println!(
            "  {} Found project files but no study.toml",
            theme::yellow().paint("⚠")
        );
        for file in &existing_files {
            println!(
                "    {} {}",
                theme::overlay0().paint("•"),
                theme::text().paint(*file)
            );
        }
        println!();

        let result = inquire::Confirm::new("Continue anyway? (files will be overwritten)")
            .with_default(false)
            .prompt_skippable()?;

        if result != Some(true) {
            println!("{}", theme::yellow().paint("cancelled"));
            return Ok(false);
        }
    }

    Ok(true)
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
    let config = Config::load();
    if !crate::commands::utils::open_in_editor(path, &config)? {
        let editor = crate::commands::utils::resolve_editor(&config);
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
