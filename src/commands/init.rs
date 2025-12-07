use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;

use crate::config::Config;
use crate::templates::grf;

/// initialise a GRF project from config and templates
pub fn grf_from_config(
    exposure: &str,
    direct_outcomes: Option<&[String]>,
    outcome_templates: Option<&[String]>,
    baselines_name: &str,
    custom_name: Option<&str>,
    who_mode: &str,
) -> Result<()> {
    // load user config
    let config = Config::load();

    // check required paths are configured
    let pull_data = config.pull_data.ok_or_else(|| {
        anyhow::anyhow!(
            "pull_data not configured. run: {} and set paths",
            "margo config".cyan()
        )
    })?;

    let push_mods_base = config.push_mods.ok_or_else(|| {
        anyhow::anyhow!(
            "push_mods not configured. run: {} and set paths",
            "margo config".cyan()
        )
    })?;

    // collect outcome variables from direct args and/or templates
    let mut outcome_vars: Vec<String> = Vec::new();

    // add direct outcomes first
    if let Some(direct) = direct_outcomes {
        outcome_vars.extend(direct.iter().cloned());
    }

    // add outcomes from templates
    if let Some(templates) = outcome_templates {
        for name in templates {
            if let Some(template) = Config::load_outcomes(name) {
                outcome_vars.extend(template.vars);
            } else {
                println!(
                    "{} outcome template '{}' not found, skipping",
                    "warning:".yellow().bold(),
                    name
                );
            }
        }
    }

    // generate project name from exposure + first outcome (or template name)
    let project_name = custom_name.map(|s| s.to_string()).unwrap_or_else(|| {
        if let Some(direct) = direct_outcomes {
            if !direct.is_empty() {
                // use first direct outcome in name
                return format!("{}-{}", exposure, direct[0]);
            }
        }
        if let Some(templates) = outcome_templates {
            if !templates.is_empty() {
                // use template names
                return format!("{}-{}", exposure, templates.join("-"));
            }
        }
        exposure.to_string()
    });

    // load baselines template
    let baseline_vars = Config::load_baselines(baselines_name)
        .map(|t| t.vars)
        .unwrap_or_else(|| {
            println!(
                "{} baseline template '{}' not found, using empty",
                "warning:".yellow().bold(),
                baselines_name
            );
            Vec::new()
        });

    // create push_mods project subfolder
    let push_mods_path = format!("{}/{}", push_mods_base, project_name);
    fs::create_dir_all(&push_mods_path)
        .with_context(|| format!("failed to create output directory '{}'", push_mods_path))?;

    println!(
        "{} GRF project '{}'",
        "Creating".green().bold(),
        project_name.cyan()
    );

    // write scripts to current directory
    let files = grf::get_template_files_with_config(
        &project_name,
        &pull_data,
        &push_mods_path,
        exposure,
        &baseline_vars,
        &outcome_vars,
        who_mode,
    );

    for (filename, content) in files {
        fs::write(&filename, content)
            .with_context(|| format!("failed to write '{}'", filename))?;
        println!("  {} {}", "wrote".green(), filename);
    }

    println!();
    println!("{}", "Project created successfully!".green().bold());
    println!();
    println!("Scripts created in current directory");
    println!("Outputs will be written to: {}", push_mods_path.cyan());
    println!();
    println!("Next steps:");
    println!("  1. Review {} and adjust as needed", "study.toml".cyan());
    println!("  2. Run scripts in order: 01, 02, 03...");
    println!();

    Ok(())
}

/// initialise a new GRF project (quiet mode for TUI)
pub fn grf_quiet(name: &str) -> Result<()> {
    grf_full(name, None, true)
}

/// full grf init with all options
fn grf_full(name: &str, from_path: Option<&str>, quiet: bool) -> Result<()> {
    let project_path = Path::new(name);

    // check if directory already exists
    if project_path.exists() {
        anyhow::bail!(
            "directory '{}' already exists. choose a different name or remove the existing directory.",
            name
        );
    }

    // if --from specified, check it exists
    let from_content = if let Some(path) = from_path {
        let from = Path::new(path);
        if !from.exists() {
            anyhow::bail!("source config not found: {}", path);
        }
        if !quiet {
            println!(
                "{} GRF project '{}' from '{}'",
                "Creating".green().bold(),
                name.cyan(),
                path.cyan()
            );
        }
        Some(fs::read_to_string(from).with_context(|| format!("failed to read '{}'", path))?)
    } else {
        if !quiet {
            println!(
                "{} GRF project '{}'",
                "Creating".green().bold(),
                name.cyan()
            );
        }
        None
    };

    // create project directory
    fs::create_dir_all(project_path)
        .with_context(|| format!("failed to create directory '{}'", name))?;

    // write all template files
    let files = grf::get_template_files(name);

    for (filename, content) in files {
        let file_path = project_path.join(&filename);

        // use source config if --from was specified and this is study.toml
        let final_content = if filename == "study.toml" {
            if let Some(ref src) = from_content {
                // update project name in cloned config
                src.replace(
                    &extract_old_project_name(src),
                    name,
                )
            } else {
                content
            }
        } else {
            content
        };

        fs::write(&file_path, final_content)
            .with_context(|| format!("failed to write '{}'", filename))?;
        if !quiet {
            println!("  {} {}", "wrote".green(), filename);
        }
    }

    if !quiet {
        println!();
        println!("{}", "Project created successfully!".green().bold());
        println!();
        println!("Next steps:");
        println!("  1. cd {}", name.cyan());
        println!("  2. Edit {} with your study configuration", "study.toml".cyan());
        println!("  3. Run scripts in order: 01, 02, 03...");
        println!();
    }

    Ok(())
}

/// extract project name from existing study.toml (first line comment)
fn extract_old_project_name(content: &str) -> String {
    // look for "# project-name - GRF study configuration" pattern
    if let Some(first_line) = content.lines().next() {
        if first_line.starts_with("# ") && first_line.contains(" - ") {
            if let Some(name) = first_line.strip_prefix("# ") {
                if let Some((name, _)) = name.split_once(" - ") {
                    return name.to_string();
                }
            }
        }
    }
    String::new()
}
