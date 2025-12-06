use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;

use crate::templates::grf;

/// initialise a new GRF project
pub fn grf(name: &str) -> Result<()> {
    let project_path = Path::new(name);

    // check if directory already exists
    if project_path.exists() {
        anyhow::bail!(
            "directory '{}' already exists. choose a different name or remove the existing directory.",
            name
        );
    }

    println!(
        "{} GRF project '{}'",
        "Creating".green().bold(),
        name.cyan()
    );

    // create project directory
    fs::create_dir_all(project_path)
        .with_context(|| format!("failed to create directory '{}'", name))?;

    // write all template files
    let files = grf::get_template_files(name);

    for (filename, content) in files {
        let file_path = project_path.join(&filename);
        fs::write(&file_path, content)
            .with_context(|| format!("failed to write '{}'", filename))?;
        println!("  {} {}", "wrote".green(), filename);
    }

    println!();
    println!("{}", "Project created successfully!".green().bold());
    println!();
    println!("Next steps:");
    println!("  1. cd {}", name.cyan());
    println!("  2. Edit {} with your study configuration", "study.toml".cyan());
    println!("  3. Run scripts in order: 01, 02, 03...");
    println!();

    Ok(())
}
