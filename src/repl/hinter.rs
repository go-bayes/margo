// ghost text hints

use nu_ansi_term::Style;
use reedline::{Hinter, History};

use crate::theme;

pub struct MargoHinter {
    hints: Vec<(&'static str, &'static str)>,
}

impl MargoHinter {
    pub fn new() -> Self {
        Self {
            hints: vec![
                ("init g", "rf <exposure> [outcomes...]"),
                ("init grf ", "<exposure> [outcomes...] [-t templates]"),
                ("init grf-", "event <exposure> [-o outcome]"),
                ("/h", "elp"),
                ("/he", "lp"),
                ("/hel", "p"),
                ("/c", "onfig"),
                ("/co", "nfig"),
                ("/con", "fig"),
                ("/conf", "ig"),
                ("/confi", "g"),
                ("/config ", "edit"),
                ("/t", "emplates"),
                ("/te", "mplates"),
                ("/tem", "plates"),
                ("/temp", "lates"),
                ("/templ", "ates"),
                ("/templa", "tes"),
                ("/templat", "es"),
                ("/template", "s"),
                ("/templates ", "edit <name>"),
                ("/v", "ars"),
                ("/va", "rs"),
                ("/var", "s"),
                ("/vars ", "<pattern>"),
                ("/q", "uit"),
                ("/qu", "it"),
                ("/qui", "t"),
                ("q", "uit"),
                ("qu", "it"),
                ("qui", "t"),
            ],
        }
    }
}

impl Default for MargoHinter {
    fn default() -> Self {
        Self::new()
    }
}

impl Hinter for MargoHinter {
    fn handle(
        &mut self,
        line: &str,
        _pos: usize,
        _history: &dyn History,
        _use_ansi_coloring: bool,
        _cwd: &str,
    ) -> String {
        // check static hints
        for (prefix, suffix) in &self.hints {
            if line == *prefix {
                return format!(
                    "{}",
                    Style::new().fg(theme::color_overlay0()).paint(*suffix)
                );
            }
        }

        String::new()
    }

    fn complete_hint(&self) -> String {
        String::new()
    }

    fn next_hint_token(&self) -> String {
        String::new()
    }
}
