// welcome screen with logo and tips

use crate::config::Config;
use crate::theme;

pub fn print_welcome() {
    print_logo();
    print_tips();
    print_defaults();
    println!();
}

fn print_logo() {
    // print logo in pink
    for line in theme::LOGO.lines() {
        if !line.is_empty() {
            println!("{}", theme::pink().paint(line));
        }
    }
    println!(
        "  {}",
        theme::subtext0().paint(theme::TAGLINE)
    );
    println!();
}

fn print_tips() {
    println!("  {}", theme::peach().paint("Tips"));
    println!(
        "  {}",
        theme::overlay0().paint("─────────────────────────────────────────────")
    );
    print_tip("/help", "show available commands");
    print_tip("/templates", "list outcome & baseline templates");
    print_tip("/vars [pattern]", "fuzzy search variables");
    print_tip("init grf", "scaffold a grf project");
    print_tip("Tab", "autocomplete commands & variables");
    print_tip("Ctrl+R", "search command history");
    println!();
}

fn print_tip(cmd: &str, desc: &str) {
    println!(
        "  {}  {:<18} {}",
        theme::overlay0().paint("•"),
        theme::sapphire().paint(cmd),
        theme::subtext0().paint(desc)
    );
}

fn print_defaults() {
    let config = Config::load();

    let data = config
        .pull_data
        .as_ref()
        .map(|p| shorten_path(p))
        .unwrap_or_else(|| "not set".to_string());

    let output = config
        .push_mods
        .as_ref()
        .map(|p| shorten_path(p))
        .unwrap_or_else(|| "not set".to_string());

    let baselines = config
        .baselines
        .as_ref()
        .cloned()
        .unwrap_or_else(|| "default".to_string());

    println!(
        "  {}",
        theme::overlay0().paint("─────────────────────────────────────────────")
    );
    println!(
        "  {}: {}   {}: {}   {}: {}",
        theme::subtext0().paint("data"),
        theme::text().paint(&data),
        theme::subtext0().paint("output"),
        theme::text().paint(&output),
        theme::subtext0().paint("baselines"),
        theme::text().paint(&baselines),
    );
}

fn shorten_path(path: &str) -> String {
    // replace home dir with ~
    if let Some(home) = dirs::home_dir() {
        if let Some(home_str) = home.to_str() {
            if path.starts_with(home_str) {
                return format!("~{}", &path[home_str.len()..]);
            }
        }
    }
    path.to_string()
}
