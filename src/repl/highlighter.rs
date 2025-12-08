// syntax highlighting for the REPL

use nu_ansi_term::Style;
use reedline::{Highlighter, StyledText};

use crate::theme;

pub struct MargoHighlighter;

impl MargoHighlighter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MargoHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl Highlighter for MargoHighlighter {
    fn highlight(&self, line: &str, _cursor: usize) -> StyledText {
        let mut styled = StyledText::new();

        if line.is_empty() {
            return styled;
        }

        // slash commands
        if line.starts_with('/') {
            highlight_slash_command(line, &mut styled);
            return styled;
        }

        // init commands
        if line.starts_with("init ") {
            highlight_init_command(line, &mut styled);
            return styled;
        }

        // quit/exit
        if line == "quit" || line == "exit" || line == "q" {
            styled.push((Style::new().fg(theme::color_sapphire()), line.to_string()));
            return styled;
        }

        // default: plain text
        styled.push((Style::new().fg(theme::color_text()), line.to_string()));
        styled
    }
}

fn highlight_slash_command(line: &str, styled: &mut StyledText) {
    let parts: Vec<&str> = line.splitn(2, ' ').collect();
    let cmd = parts[0];
    let rest = parts.get(1).copied().unwrap_or("");

    // command in sapphire
    styled.push((Style::new().fg(theme::color_sapphire()), cmd.to_string()));

    if !rest.is_empty() {
        styled.push((Style::new().fg(theme::color_text()), " ".to_string()));
        // subcommands/args in teal
        styled.push((Style::new().fg(theme::color_teal()), rest.to_string()));
    }
}

fn highlight_init_command(line: &str, styled: &mut StyledText) {
    let mut chars = line.chars().peekable();
    let mut in_flag = false;
    let mut after_flag = false;

    // "init" keyword
    styled.push((Style::new().fg(theme::color_mauve()), "init".to_string()));

    // skip "init"
    for _ in 0..4 {
        chars.next();
    }

    // space after init
    if chars.peek() == Some(&' ') {
        styled.push((Style::new().fg(theme::color_text()), " ".to_string()));
        chars.next();
    }

    // collect rest
    let rest: String = chars.collect();
    let parts: Vec<&str> = rest.split_whitespace().collect();

    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            styled.push((Style::new().fg(theme::color_text()), " ".to_string()));
        }

        if part.starts_with('-') {
            // flag in peach
            styled.push((Style::new().fg(theme::color_peach()), part.to_string()));
            in_flag = true;
            after_flag = false;
        } else if in_flag && !after_flag {
            // value after flag in yellow
            styled.push((Style::new().fg(theme::color_yellow()), part.to_string()));
            after_flag = true;
        } else if i == 0 {
            // template type (grf, grf-event) in mauve
            styled.push((Style::new().fg(theme::color_mauve()), part.to_string()));
            in_flag = false;
            after_flag = false;
        } else {
            // variables in teal
            styled.push((Style::new().fg(theme::color_teal()), part.to_string()));
            in_flag = false;
            after_flag = false;
        }
    }
}
