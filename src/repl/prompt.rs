// custom prompt with vi mode indicator

use nu_ansi_term::Style;
use reedline::{Prompt, PromptEditMode, PromptHistorySearch, PromptHistorySearchStatus, PromptViMode};
use std::borrow::Cow;
use std::cell::Cell;

use crate::theme;

pub struct MargoPrompt {
    // track vi mode for right prompt hint
    vi_normal: Cell<bool>,
}

impl MargoPrompt {
    pub fn new() -> Self {
        Self {
            vi_normal: Cell::new(false),
        }
    }
}

impl Default for MargoPrompt {
    fn default() -> Self {
        Self::new()
    }
}

impl Prompt for MargoPrompt {
    fn render_prompt_left(&self) -> Cow<'_, str> {
        Cow::Owned(format!("{}", theme::pink().paint("margo")))
    }

    fn render_prompt_right(&self) -> Cow<'_, str> {
        if self.vi_normal.get() {
            // show hint in normal mode
            Cow::Owned(format!(
                "{} {} {} {}",
                theme::overlay0().paint("NORMAL"),
                theme::overlay0().paint("•"),
                theme::subtext0().paint("i"),
                theme::overlay0().paint("insert")
            ))
        } else {
            // show hints in insert mode
            Cow::Owned(format!(
                "{}  {}  {}",
                theme::overlay0().paint("/help"),
                theme::overlay0().paint("/home"),
                theme::overlay0().paint("/q"),
            ))
        }
    }

    fn render_prompt_indicator(&self, edit_mode: PromptEditMode) -> Cow<'_, str> {
        match edit_mode {
            PromptEditMode::Default | PromptEditMode::Emacs => {
                self.vi_normal.set(false);
                Cow::Owned(format!(" {} ", theme::teal().paint("❯")))
            }
            PromptEditMode::Vi(vi_mode) => match vi_mode {
                PromptViMode::Insert => {
                    self.vi_normal.set(false);
                    Cow::Owned(format!(" {} ", theme::teal().paint("❯")))
                }
                PromptViMode::Normal => {
                    self.vi_normal.set(true);
                    Cow::Owned(format!(" {} ", theme::pink().paint("●")))
                }
            },
            PromptEditMode::Custom(_) => {
                self.vi_normal.set(false);
                Cow::Owned(format!(" {} ", theme::peach().paint("▸")))
            }
        }
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<'_, str> {
        Cow::Owned(format!("{} ", theme::overlay0().paint("...")))
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: PromptHistorySearch,
    ) -> Cow<'_, str> {
        let prefix = match history_search.status {
            PromptHistorySearchStatus::Passing => "",
            PromptHistorySearchStatus::Failing => "failing ",
        };

        Cow::Owned(format!(
            "({}reverse-search: {}) ",
            Style::new().fg(theme::color_overlay0()).paint(prefix),
            Style::new().fg(theme::color_sapphire()).paint(&history_search.term),
        ))
    }
}
