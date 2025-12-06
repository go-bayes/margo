use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

mod commands;
mod templates;

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
}

#[derive(Subcommand)]
enum InitTemplate {
    /// Create a GRF (Generalised Random Forests) project
    Grf {
        /// Project name (creates directory with this name)
        name: String,
    },
    /// Create an LMTP (Longitudinal Modified Treatment Policies) project
    Lmtp {
        /// Project name (creates directory with this name)
        name: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { template } => match template {
            InitTemplate::Grf { name } => {
                commands::init::grf(&name)?;
            }
            InitTemplate::Lmtp { name: _ } => {
                println!(
                    "{} LMTP template not yet implemented",
                    "warning:".yellow().bold()
                );
                println!("  use {} for now", "margo init grf".cyan());
                std::process::exit(1);
            }
        },
    }

    Ok(())
}
