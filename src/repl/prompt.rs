// custom prompt with vi mode indicator

use nu_ansi_term::Style;
use reedline::{Prompt, PromptEditMode, PromptHistorySearch, PromptHistorySearchStatus, PromptViMode};
use std::borrow::Cow;

use crate::theme;

pub struct MargoPrompt;

impl MargoPrompt {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MargoPrompt {
    fn default() -> Self {
        Self::new()
    }
}

impl Prompt for MargoPrompt {
    fn render_prompt_left(&self) -> Cow<str> {
        Cow::Owned(format!("{}", theme::pink().paint("margo")))
    }

    fn render_prompt_right(&self) -> Cow<str> {
        Cow::Borrowed("")
    }

    fn render_prompt_indicator(&self, edit_mode: PromptEditMode) -> Cow<str> {
        match edit_mode {
            PromptEditMode::Default | PromptEditMode::Emacs => {
                Cow::Owned(format!(" {} ", theme::teal().paint("❯")))
            }
            PromptEditMode::Vi(vi_mode) => match vi_mode {
                PromptViMode::Insert => {
                    Cow::Owned(format!(" {} ", theme::teal().paint("❯")))
                }
                PromptViMode::Normal => {
                    Cow::Owned(format!(" {} ", theme::pink().paint("●")))
                }
            },
            PromptEditMode::Custom(_) => {
                Cow::Owned(format!(" {} ", theme::peach().paint("▸")))
            }
        }
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<str> {
        Cow::Owned(format!("{} ", theme::overlay0().paint("...")))
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: PromptHistorySearch,
    ) -> Cow<str> {
        let prefix = match history_search.status {
            PromptHistorySearchStatus::Passing => "",
            PromptHistorySearchStatus::Failing => "failing ",
        };

        Cow::Owned(format!(
            "({}reverse-search: {}) ",
            Style::new().fg(theme::OVERLAY0).paint(prefix),
            Style::new().fg(theme::SAPPHIRE).paint(&history_search.term),
        ))
    }
}
