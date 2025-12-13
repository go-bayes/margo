use anyhow::Result;
use clap::{Parser, Subcommand};
use nu_ansi_term::Color;
use std::fs;

mod commands;
mod config;
mod data;
mod repl;
mod templates;
mod theme;

#[derive(Parser)]
#[command(name = "margo")]
#[command(version)]
#[command(about = "Scaffold margot causal inference projects", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialise a new project from a template
    Init {
        #[command(subcommand)]
        template: InitTemplate,
    },
    /// Manage user configuration (~/.config/margo/config.toml)
    Config {
        #[command(subcommand)]
        action: Option<ConfigAction>,
    },
    /// Manage templates (baselines and outcomes)
    Templates {
        #[command(subcommand)]
        action: Option<TemplatesAction>,
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
enum TemplatesAction {
    /// List available templates
    List,
    /// List example templates (bundled with margo)
    Examples,
    /// Copy an example template to your config
    Copy {
        /// Template kind: "baselines" or "outcomes"
        kind: String,
        /// Template name (e.g., "default", "wellbeing")
        name: String,
    },
    /// Initialise example templates (creates examples/ directories)
    Init,
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
    // load config and initialise theme
    let cfg = config::Config::load();
    if let Some(theme_name) = &cfg.theme {
        theme::set_theme(theme_name);
    }

    let cli = Cli::parse();

    match cli.command {
        // no subcommand → launch REPL
        None => {
            repl::run()?;
        }
        Some(Commands::Init { template }) => match template {
            InitTemplate::Grf {
                exposure,
                outcomes,
                templates,
                baselines,
                name,
            } => {
                commands::init::grf_from_config(
                    &exposure,
                    if outcomes.is_empty() {
                        None
                    } else {
                        Some(&outcomes)
                    },
                    templates.as_deref(),
                    &baselines,
                    None, // no baseline override from CLI
                    name.as_deref(),
                )?;
            }
            InitTemplate::GrfEvent {
                exposure,
                outcome,
                waves,
                reference,
                baselines,
                name,
            } => {
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
                    Color::Yellow.bold().paint("warning:")
                );
                println!(
                    "  use {} for now",
                    Color::Cyan.paint("margo init grf")
                );
                std::process::exit(1);
            }
        },
        Some(Commands::Config { action }) => {
            let config_path = config::Defaults::config_path();
            let config_dir = config::Defaults::config_dir();

            match action {
                Some(ConfigAction::Init) | None => {
                    // create config file with defaults
                    if config_path.exists() {
                        println!(
                            "{} config already exists at: {}",
                            Color::Cyan.bold().paint("note:"),
                            config_path.display()
                        );
                        println!(
                            "  edit with: {}",
                            Color::Cyan.paint("margo config edit")
                        );
                    } else {
                        fs::create_dir_all(&config_dir)?;
                        fs::write(&config_path, config::Defaults::default_config_content())?;
                        println!(
                            "{} config file at: {}",
                            Color::Green.bold().paint("Created"),
                            config_path.display()
                        );
                        println!();
                        println!("Edit this file to set your default paths:");
                        println!("  {}", Color::Cyan.paint("margo config edit"));
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
                    let cfg = config::Config::load();
                    if !commands::utils::open_in_editor(&config_path.to_string_lossy(), &cfg)? {
                        let editor = commands::utils::resolve_editor(&cfg);
                        println!(
                            "{} failed to open editor '{}'",
                            Color::Red.bold().paint("error:"),
                            editor
                        );
                        println!(
                            "  set $EDITOR or edit manually: {}",
                            config_path.display()
                        );
                    }
                }
            }
        }
        Some(Commands::Templates { action }) => {
            match action {
                Some(TemplatesAction::List) | None => {
                    // list user templates
                    let baselines = config::Config::list_baselines();
                    let outcomes = config::Config::list_outcomes();

                    println!("{}", Color::Cyan.bold().paint("Your templates:"));
                    println!();

                    if baselines.is_empty() {
                        println!("  baselines: (none)");
                    } else {
                        println!("  {}:", Color::Green.paint("baselines"));
                        for name in &baselines {
                            println!("    - {}", name);
                        }
                    }
                    println!();

                    if outcomes.is_empty() {
                        println!("  outcomes: (none)");
                    } else {
                        println!("  {}:", Color::Green.paint("outcomes"));
                        for name in &outcomes {
                            println!("    - {}", name);
                        }
                    }
                    println!();
                    println!(
                        "Templates stored in: {}",
                        Color::Cyan.paint(config::Config::config_dir().display().to_string())
                    );
                    println!();
                    println!(
                        "See examples with: {}",
                        Color::Cyan.paint("margo templates examples")
                    );
                }
                Some(TemplatesAction::Examples) => {
                    // first ensure examples are initialised
                    if let Err(e) = config::Config::init_examples() {
                        println!(
                            "{} {}",
                            Color::Red.bold().paint("error:"),
                            e
                        );
                        std::process::exit(1);
                    }

                    let baselines = config::Config::list_baselines_examples();
                    let outcomes = config::Config::list_outcomes_examples();

                    println!("{}", Color::Cyan.bold().paint("Example templates:"));
                    println!();

                    if baselines.is_empty() {
                        println!("  baselines/examples: (none)");
                    } else {
                        println!("  {}:", Color::Green.paint("baselines/examples"));
                        for name in &baselines {
                            println!("    - {}", name);
                        }
                    }
                    println!();

                    if outcomes.is_empty() {
                        println!("  outcomes/examples: (none)");
                    } else {
                        println!("  {}:", Color::Green.paint("outcomes/examples"));
                        for name in &outcomes {
                            println!("    - {}", name);
                        }
                    }
                    println!();
                    println!(
                        "Copy an example to your templates with:"
                    );
                    println!(
                        "  {}",
                        Color::Cyan.paint("margo templates copy baselines default")
                    );
                    println!(
                        "  {}",
                        Color::Cyan.paint("margo templates copy outcomes wellbeing")
                    );
                }
                Some(TemplatesAction::Copy { kind, name }) => {
                    // ensure examples exist first
                    if let Err(e) = config::Config::init_examples() {
                        println!(
                            "{} {}",
                            Color::Red.bold().paint("error:"),
                            e
                        );
                        std::process::exit(1);
                    }

                    match config::Config::copy_example(&kind, &name) {
                        Ok(dest) => {
                            println!(
                                "{} {} template '{}' to {}",
                                Color::Green.bold().paint("Copied"),
                                kind,
                                name,
                                dest.display()
                            );
                            println!();
                            println!("You can now use it with:");
                            if kind == "baselines" || kind == "baseline" {
                                println!(
                                    "  {}",
                                    Color::Cyan.paint(format!("margo init grf exposure -b {}", name))
                                );
                            } else {
                                println!(
                                    "  {}",
                                    Color::Cyan.paint(format!("margo init grf exposure -t {}", name))
                                );
                            }
                        }
                        Err(e) => {
                            println!(
                                "{} {}",
                                Color::Red.bold().paint("error:"),
                                e
                            );
                            std::process::exit(1);
                        }
                    }
                }
                Some(TemplatesAction::Init) => {
                    match config::Config::init_examples() {
                        Ok(created) => {
                            if created.is_empty() {
                                println!(
                                    "{} example templates already initialised",
                                    Color::Cyan.bold().paint("note:")
                                );
                            } else {
                                println!(
                                    "{} example templates",
                                    Color::Green.bold().paint("Initialised")
                                );
                                for path in &created {
                                    println!("  {} {}", Color::Green.paint("created"), path);
                                }
                            }
                            println!();
                            println!(
                                "View examples with: {}",
                                Color::Cyan.paint("margo templates examples")
                            );
                        }
                        Err(e) => {
                            println!(
                                "{} {}",
                                Color::Red.bold().paint("error:"),
                                e
                            );
                            std::process::exit(1);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
