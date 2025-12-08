// interactive variable picker using inquire

use anyhow::Result;
use inquire::ui::{Color, RenderConfig, StyleSheet, Styled};
use inquire::{MultiSelect, Select};

use crate::data::VARIABLES;

/// catppuccin-styled render config
fn catppuccin_config() -> RenderConfig<'static> {
    let pink = Color::Rgb {
        r: 245,
        g: 194,
        b: 231,
    };
    let teal = Color::Rgb {
        r: 148,
        g: 226,
        b: 213,
    };
    let peach = Color::Rgb {
        r: 250,
        g: 179,
        b: 135,
    };
    let subtext0 = Color::Rgb {
        r: 166,
        g: 173,
        b: 200,
    };
    let overlay0 = Color::Rgb {
        r: 108,
        g: 112,
        b: 134,
    };
    let text = Color::Rgb {
        r: 205,
        g: 214,
        b: 244,
    };

    // subtle background for highlighted row
    let surface0 = Color::Rgb {
        r: 49,
        g: 50,
        b: 68,
    };

    RenderConfig {
        prompt_prefix: Styled::new("?").with_fg(pink),
        answered_prompt_prefix: Styled::new("✓").with_fg(teal),
        prompt: StyleSheet::new().with_fg(text),
        default_value: StyleSheet::new().with_fg(overlay0),
        placeholder: StyleSheet::new().with_fg(overlay0),
        help_message: StyleSheet::new().with_fg(subtext0),
        text_input: StyleSheet::new().with_fg(text),
        answer: StyleSheet::new().with_fg(teal),
        canceled_prompt_indicator: Styled::new("✗").with_fg(peach),
        highlighted_option_prefix: Styled::new("❯").with_fg(pink),
        selected_option: Some(StyleSheet::new().with_fg(teal).with_bg(surface0)),
        scroll_up_prefix: Styled::new("▲").with_fg(overlay0),
        scroll_down_prefix: Styled::new("▼").with_fg(overlay0),
        option: StyleSheet::new().with_fg(text),
        selected_checkbox: Styled::new("◉").with_fg(teal),
        unselected_checkbox: Styled::new("○").with_fg(overlay0),
        option_index_prefix: inquire::ui::IndexPrefix::None,
        ..RenderConfig::default()
    }
}

/// pick a single variable (for exposure)
pub fn pick_exposure() -> Result<Option<String>> {
    let variables: Vec<&str> = VARIABLES.iter().copied().collect();

    let result = Select::new("Select exposure variable:", variables)
        .with_vim_mode(true)
        .with_page_size(15)
        .with_help_message("↑↓ navigate, type to filter, Enter select, Esc cancel")
        .with_render_config(catppuccin_config())
        .prompt_skippable()?;

    Ok(result.map(|s| s.to_string()))
}

/// pick multiple variables (for outcomes)
pub fn pick_outcomes() -> Result<Option<Vec<String>>> {
    let variables: Vec<&str> = VARIABLES.iter().copied().collect();

    let result = MultiSelect::new("Select outcome variables:", variables)
        .with_vim_mode(true)
        .with_page_size(15)
        .with_help_message("↑↓ navigate, Space select, Enter confirm, Esc cancel")
        .with_render_config(catppuccin_config())
        .prompt_skippable()?;

    Ok(result.map(|v| v.into_iter().map(|s| s.to_string()).collect()))
}

/// pick a single variable with custom prompt
pub fn pick_variable(prompt: &str) -> Result<Option<String>> {
    let variables: Vec<&str> = VARIABLES.iter().copied().collect();

    let result = Select::new(prompt, variables)
        .with_vim_mode(true)
        .with_page_size(15)
        .with_help_message("↑↓ navigate, type to filter, Enter select, Esc cancel")
        .with_render_config(catppuccin_config())
        .prompt_skippable()?;

    Ok(result.map(|s| s.to_string()))
}

/// pick a baseline template
pub fn pick_baseline(available: &[String]) -> Result<Option<String>> {
    if available.is_empty() {
        return Ok(Some("default".to_string()));
    }

    let mut options: Vec<&str> = available.iter().map(|s| s.as_str()).collect();
    // ensure "default" is first if it exists
    if let Some(pos) = options.iter().position(|&s| s == "default") {
        options.remove(pos);
        options.insert(0, "default");
    }

    let result = Select::new("Select baseline template:", options)
        .with_vim_mode(true)
        .with_page_size(10)
        .with_help_message("↑↓ navigate, Enter select, Esc for default")
        .with_render_config(catppuccin_config())
        .prompt_skippable()?;

    Ok(result.map(|s| s.to_string()).or(Some("default".to_string())))
}

/// pick model type (grf, grf-event, lmtp)
pub fn pick_model() -> Result<Option<String>> {
    let models = vec![
        "grf        — generalised random forests",
        "grf-event  — grf event study (multi-wave)",
        "lmtp       — longitudinal modified treatment policies (coming soon)",
    ];

    let result = Select::new("Select model type:", models)
        .with_vim_mode(true)
        .with_page_size(5)
        .with_help_message("↑↓ navigate, Enter select")
        .with_render_config(catppuccin_config())
        .prompt_skippable()?;

    Ok(result.map(|s| s.split_whitespace().next().unwrap_or("grf").to_string()))
}

/// edit template variables with pre-selected items
/// returns None if cancelled, Some(vec) with updated selection
pub fn edit_template(name: &str, current_vars: &[String]) -> Result<Option<Vec<String>>> {
    let variables: Vec<&str> = VARIABLES.iter().copied().collect();

    // find indices of currently selected vars
    let defaults: Vec<usize> = current_vars
        .iter()
        .filter_map(|v| variables.iter().position(|&var| var == v.as_str()))
        .collect();

    let prompt = format!("Edit template '{}' ({} vars):", name, current_vars.len());

    let result = MultiSelect::new(&prompt, variables)
        .with_vim_mode(true)
        .with_page_size(15)
        .with_default(&defaults)
        .with_help_message("↑↓ navigate, Space toggle, / filter, Enter save, Esc cancel")
        .with_render_config(catppuccin_config())
        .prompt_skippable()?;

    Ok(result.map(|v| v.into_iter().map(|s| s.to_string()).collect()))
}
