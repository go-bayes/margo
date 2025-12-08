// welcome screen with logo and config

use crate::config::Config;
use crate::theme;
use std::env;

pub fn print_welcome() {
    print_logo();
    print_config();
    print_tips();
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

fn print_config() {
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

    let cwd = env::current_dir()
        .map(|p| shorten_path(&p.to_string_lossy()))
        .unwrap_or_else(|_| "unknown".to_string());

    println!("  {}", theme::peach().paint("Config"));
    println!(
        "  {}",
        theme::overlay0().paint("─────────────────────────────────────────────")
    );
    print_config_line("data", &data);
    print_config_line("output", &output);
    print_config_line("baselines", &baselines);
    print_config_line("cwd", &cwd);
    println!();
}

fn print_config_line(label: &str, value: &str) {
    let style = if value == "not set" {
        theme::overlay0()
    } else {
        theme::text()
    };
    println!(
        "  {}  {:<10} {}",
        theme::overlay0().paint("•"),
        theme::subtext0().paint(label),
        style.paint(value)
    );
}

fn print_tips() {
    println!("  {}", theme::peach().paint("Tips"));
    println!(
        "  {}",
        theme::overlay0().paint("─────────────────────────────────────────────")
    );
    print_tip("/help", "commands");
    print_tip("/vars", "browse variables");
    print_tip("/e", "edit template");
    print_tip("init", "scaffold project");
    print_tip("/r", "refresh screen");
    print_tip("q", "quit");
    println!();
}

fn print_tip(cmd: &str, desc: &str) {
    println!(
        "  {}  {:<12} {}",
        theme::overlay0().paint("•"),
        theme::sapphire().paint(cmd),
        theme::subtext0().paint(desc)
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
