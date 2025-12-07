use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::fs;

mod commands;
mod config;
mod templates;
mod tui;

#[derive(Parser)]
#[command(name = "margo")]
#[command(version)]
#[command(about = "Scaffold margot causal inference projects", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialise a new project from a template
    Init {
        #[command(subcommand)]
        template: InitTemplate,
    },
    /// Launch interactive TUI for project creation
    New,
    /// Manage user configuration (~/.config/margo/config.toml)
    Config {
        #[command(subcommand)]
        action: Option<ConfigAction>,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Create default config file
    Init,
    /// Show config file path
    Path,
    /// Edit config file (opens in $EDITOR)
    Edit,
}

#[derive(Subcommand)]
enum InitTemplate {
    /// Create a GRF (Generalised Random Forests) project
    Grf {
        /// Exposure variable name
        exposure: String,

        /// Outcome variable(s) - specify directly or use -t for templates
        #[arg(trailing_var_arg = true)]
        outcomes: Vec<String>,

        /// Load outcomes from template(s) instead (comma-separated, e.g., "wellbeing,health")
        #[arg(long, short = 't', value_delimiter = ',')]
        templates: Option<Vec<String>>,

        /// Baseline template to use (default: "default")
        #[arg(long, short = 'b', default_value = "default")]
        baselines: String,

        /// Custom project name (default: auto-generated from exposure-outcomes)
        #[arg(long, short = 'n')]
        name: Option<String>,

        /// WHO mode for BMI/exercise variables: default, cat, or num
        #[arg(long, short = 'w', default_value = "default")]
        who_mode: String,
    },
    /// Create a GRF Event Study project (multi-outcome waves)
    GrfEvent {
        /// Exposure variable name
        exposure: String,

        /// Outcome variable name (single variable measured across waves)
        #[arg(long, short = 'o')]
        outcome: Option<String>,

        /// Outcome waves (comma-separated, e.g., "2011,2012,2013,2014")
        #[arg(long, short = 'w', value_delimiter = ',')]
        waves: Option<Vec<String>>,

        /// Reference wave for t=0 (default: first outcome wave)
        #[arg(long, short = 'r')]
        reference: Option<String>,

        /// Baseline template to use (default: "default")
        #[arg(long, short = 'b', default_value = "default")]
        baselines: String,

        /// Custom project name
        #[arg(long, short = 'n')]
        name: Option<String>,
    },
    /// Create an LMTP (Longitudinal Modified Treatment Policies) project
    Lmtp {
        /// Exposure variable name
        exposure: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { template } => match template {
            InitTemplate::Grf { exposure, outcomes, templates, baselines, name, who_mode } => {
                commands::init::grf_from_config(
                    &exposure,
                    if outcomes.is_empty() { None } else { Some(&outcomes) },
                    templates.as_deref(),
                    &baselines,
                    name.as_deref(),
                    &who_mode,
                )?;
            }
            InitTemplate::GrfEvent { exposure, outcome, waves, reference, baselines, name } => {
                commands::init::grf_event_from_config(
                    &exposure,
                    outcome.as_deref(),
                    waves.as_deref(),
                    reference.as_deref(),
                    &baselines,
                    name.as_deref(),
                )?;
            }
            InitTemplate::Lmtp { exposure: _ } => {
                println!(
                    "{} LMTP template not yet implemented",
                    "warning:".yellow().bold()
                );
                println!("  use {} for now", "margo init grf".cyan());
                std::process::exit(1);
            }
        },
        Commands::New => {
            tui::run()?;
        }
        Commands::Config { action } => {
            let config_path = config::Defaults::config_path();
            let config_dir = config::Defaults::config_dir();

            match action {
                Some(ConfigAction::Init) | None => {
                    // create config file with defaults
                    if config_path.exists() {
                        println!(
                            "{} config already exists at: {}",
                            "note:".cyan().bold(),
                            config_path.display()
                        );
                        println!("  edit with: {}", "margo config edit".cyan());
                    } else {
                        fs::create_dir_all(&config_dir)?;
                        fs::write(&config_path, config::Defaults::default_config_content())?;
                        println!(
                            "{} config file at: {}",
                            "Created".green().bold(),
                            config_path.display()
                        );
                        println!();
                        println!("Edit this file to set your default paths:");
                        println!("  {}", "margo config edit".cyan());
                    }
                }
                Some(ConfigAction::Path) => {
                    println!("{}", config_path.display());
                }
                Some(ConfigAction::Edit) => {
                    // create if doesn't exist
                    if !config_path.exists() {
                        fs::create_dir_all(&config_dir)?;
                        fs::write(&config_path, config::Defaults::default_config_content())?;
                    }

                    // open in editor
                    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
                    let status = std::process::Command::new(&editor)
                        .arg(&config_path)
                        .status()?;

                    if !status.success() {
                        println!(
                            "{} failed to open editor '{}'",
                            "error:".red().bold(),
                            editor
                        );
                        println!("  set $EDITOR or edit manually: {}", config_path.display());
                    }
                }
            }
        }
    }

    Ok(())
}
