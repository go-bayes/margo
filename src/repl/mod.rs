// REPL (Read-Eval-Print Loop) for interactive margo sessions

mod commands;
mod completer;
mod fuzzy;
mod highlighter;
mod hinter;
mod picker;
mod prompt;
mod welcome;

use anyhow::Result;
use reedline::{FileBackedHistory, Reedline, Signal, Vi};
use std::path::PathBuf;

use crate::config::Config;
use completer::MargoCompleter;
use highlighter::MargoHighlighter;
use hinter::MargoHinter;
use prompt::MargoPrompt;

/// run the interactive REPL
pub fn run() -> Result<()> {
    welcome::print_welcome();

    let mut editor = create_editor()?;
    let prompt = MargoPrompt::new();

    loop {
        match editor.read_line(&prompt) {
            Ok(Signal::Success(line)) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                match parse_input(line) {
                    Input::Quit => break,
                    Input::Slash(cmd) => {
                        if let Err(e) = commands::handle_slash(&cmd) {
                            eprintln!("{}", crate::theme::red().paint(format!("error: {}", e)));
                        }
                    }
                    Input::Init(args) => {
                        if let Err(e) = commands::handle_init(&args) {
                            eprintln!("{}", crate::theme::red().paint(format!("error: {}", e)));
                        }
                    }
                    Input::Unknown(s) => {
                        print_unknown(&s);
                    }
                }
            }
            Ok(Signal::CtrlC) => {
                // interrupt - just continue
                continue;
            }
            Ok(Signal::CtrlD) => {
                // eof - exit
                break;
            }
            Err(e) => {
                eprintln!("{}", crate::theme::red().paint(format!("error: {}", e)));
                break;
            }
        }
    }

    println!();
    Ok(())
}

fn create_editor() -> Result<Reedline> {
    let history_path = history_path();

    // ensure parent directory exists
    if let Some(parent) = history_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let history = Box::new(FileBackedHistory::with_file(1000, history_path)?);
    let completer = Box::new(MargoCompleter::new());
    let highlighter = Box::new(MargoHighlighter::new());
    let hinter = Box::new(MargoHinter::new());

    let editor = Reedline::create()
        .with_history(history)
        .with_completer(completer)
        .with_highlighter(highlighter)
        .with_hinter(hinter)
        .with_edit_mode(Box::new(Vi::default()));

    Ok(editor)
}

fn history_path() -> PathBuf {
    Config::config_dir().join("history")
}

#[derive(Debug)]
enum Input {
    Quit,
    Slash(String),
    Init(String),
    Unknown(String),
}

fn parse_input(line: &str) -> Input {
    let line = line.trim();

    // slash commands
    if line.starts_with('/') {
        let cmd = line.strip_prefix('/').unwrap_or(line);
        if cmd == "q" || cmd == "quit" || cmd == "exit" {
            return Input::Quit;
        }
        return Input::Slash(cmd.to_string());
    }

    // init commands
    if line.starts_with("init ") || line == "init" {
        return Input::Init(line.to_string());
    }

    // quit without slash (including vim-style :q)
    if line == "quit" || line == "exit" || line == "q" || line == ":q" || line == ":q!" || line == ":wq" {
        return Input::Quit;
    }

    Input::Unknown(line.to_string())
}

fn print_unknown(cmd: &str) {
    use crate::theme;
    println!(
        "{} unknown command: {}",
        theme::yellow().paint("warning:"),
        theme::text().paint(cmd)
    );
    println!(
        "  type {} for available commands",
        theme::sapphire().paint("/help")
    );
}
