// custom widgets for margo tui

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Stylize},
    symbols::Marker,
    text::{Line, Span},
    widgets::{
        canvas::{Canvas, Map, MapResolution},
        Block, Borders, Paragraph, Widget,
    },
};

use super::theme;

/// welcome screen widget with logo
pub struct WelcomeScreen {
    pub tick: u64,
}

impl WelcomeScreen {
    pub fn new(tick: u64) -> Self {
        Self { tick }
    }
}

impl Default for WelcomeScreen {
    fn default() -> Self {
        Self { tick: 0 }
    }
}

impl Widget for WelcomeScreen {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let logo_width = 50;

        // render logo
        let logo_lines: Vec<Line> = theme::LOGO
            .lines()
            .map(|line| Line::from(Span::styled(line, Style::new().fg(theme::TEAL))))
            .collect();

        let logo = Paragraph::new(logo_lines);
        let logo_area = Rect {
            x: area.x + 2,
            y: area.y + 1,
            width: logo_width.min(area.width.saturating_sub(4)),
            height: 10.min(area.height),
        };
        logo.render(logo_area, buf);

        // render tagline below logo
        let tagline = Paragraph::new(Line::from(vec![Span::styled(
            theme::TAGLINE,
            Style::new().fg(theme::SUBTEXT0).italic(),
        )]));
        let tagline_area = Rect {
            x: area.x + 4,
            y: logo_area.y + logo_area.height,
            width: 50.min(area.width.saturating_sub(6)),
            height: 1,
        };
        tagline.render(tagline_area, buf);

        // blinking "press enter" prompt
        let blink = (self.tick / 10) % 2 == 0;
        if blink && area.height > 14 {
            let prompt = Paragraph::new(Line::from(vec![
                Span::styled("▶ ", Style::new().fg(theme::GREEN)),
                Span::styled("Press ", Style::new().fg(theme::SUBTEXT0)),
                Span::styled("Enter", Style::new().fg(theme::MAUVE).bold()),
                Span::styled(" to begin", Style::new().fg(theme::SUBTEXT0)),
            ]));
            let prompt_area = Rect {
                x: area.x + 4,
                y: logo_area.y + logo_area.height + 2,
                width: 30,
                height: 1,
            };
            prompt.render(prompt_area, buf);
        }
    }
}

/// nz map canvas widget with animation
pub struct NzMapCanvas {
    pub tick: u64,
}

impl NzMapCanvas {
    pub fn new(tick: u64) -> Self {
        Self { tick }
    }
}

impl Widget for NzMapCanvas {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // animate colour slightly
        let phase = (self.tick % 60) as f64 / 60.0;
        let pulse = ((phase * std::f64::consts::PI * 2.0).sin() * 20.0) as u8;
        let map_color = ratatui::style::Color::Rgb(
            (148_u8).saturating_add(pulse / 2),
            (226_u8).saturating_sub(pulse / 3),
            (213_u8).saturating_add(pulse / 4),
        );

        let canvas = Canvas::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::new().fg(theme::SURFACE1))
                    .title(" Aotearoa ")
                    .title_style(Style::new().fg(theme::MAUVE)),
            )
            .marker(Marker::Braille)
            .paint(move |ctx| {
                ctx.draw(&Map {
                    color: map_color,
                    resolution: MapResolution::High,
                });
            })
            .x_bounds([theme::NZ_LON_MIN, theme::NZ_LON_MAX])
            .y_bounds([theme::NZ_LAT_MIN, theme::NZ_LAT_MAX]);

        canvas.render(area, buf);
    }
}

/// menu item for template selection
#[derive(Clone)]
pub struct MenuItem {
    pub label: String,
    pub description: String,
    pub key: char,
}

impl MenuItem {
    pub fn new(label: &str, description: &str, key: char) -> Self {
        Self {
            label: label.to_string(),
            description: description.to_string(),
            key,
        }
    }
}

/// template selection menu
pub struct TemplateMenu<'a> {
    items: &'a [MenuItem],
    selected: usize,
}

impl<'a> TemplateMenu<'a> {
    pub fn new(items: &'a [MenuItem], selected: usize) -> Self {
        Self { items, selected }
    }
}

impl Widget for TemplateMenu<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Select Template ")
            .title_style(Style::new().fg(theme::MAUVE).bold())
            .borders(Borders::ALL)
            .border_style(Style::new().fg(theme::SURFACE1));

        let inner = block.inner(area);
        block.render(area, buf);

        for (i, item) in self.items.iter().enumerate() {
            let y = inner.y + i as u16 * 2;
            if y >= inner.y + inner.height {
                break;
            }

            let (key_style, label_style, desc_style) = if i == self.selected {
                (
                    Style::new().fg(theme::CRUST).bg(theme::MAUVE).bold(),
                    Style::new().fg(theme::MAUVE).bold(),
                    Style::new().fg(theme::TEXT),
                )
            } else {
                (
                    Style::new().fg(theme::PEACH),
                    Style::new().fg(theme::TEXT),
                    Style::new().fg(theme::SUBTEXT0),
                )
            };

            // key hint
            let key_span = Span::styled(format!(" {} ", item.key), key_style);
            buf.set_span(inner.x + 1, y, &key_span, 3);

            // label
            let label_span = Span::styled(&item.label, label_style);
            buf.set_span(inner.x + 5, y, &label_span, inner.width.saturating_sub(6));

            // description
            let desc_span = Span::styled(&item.description, desc_style);
            buf.set_span(inner.x + 5, y + 1, &desc_span, inner.width.saturating_sub(6));
        }
    }
}

/// input field widget
pub struct InputField<'a> {
    label: &'a str,
    value: &'a str,
    focused: bool,
    cursor_pos: usize,
}

impl<'a> InputField<'a> {
    pub fn new(label: &'a str, value: &'a str, focused: bool, cursor_pos: usize) -> Self {
        Self {
            label,
            value,
            focused,
            cursor_pos,
        }
    }
}

impl Widget for InputField<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border_color = if self.focused {
            theme::MAUVE
        } else {
            theme::SURFACE1
        };

        let block = Block::default()
            .title(format!(" {} ", self.label))
            .title_style(Style::new().fg(theme::TEXT))
            .borders(Borders::ALL)
            .border_style(Style::new().fg(border_color));

        let inner = block.inner(area);
        block.render(area, buf);

        // render value with cursor
        let value_span = Span::styled(self.value, Style::new().fg(theme::TEXT));
        buf.set_span(inner.x + 1, inner.y, &value_span, inner.width.saturating_sub(2));

        // render cursor if focused
        if self.focused && inner.width > 2 {
            let cursor_x = inner.x + 1 + self.cursor_pos as u16;
            if cursor_x < inner.x + inner.width - 1 {
                buf.set_string(
                    cursor_x,
                    inner.y,
                    "▌",
                    Style::new().fg(theme::MAUVE),
                );
            }
        }
    }
}

/// status bar at bottom
pub struct StatusBar<'a> {
    message: &'a str,
    is_error: bool,
    keybinds: &'a str,
}

impl<'a> StatusBar<'a> {
    pub fn new(message: &'a str, is_error: bool) -> Self {
        Self {
            message,
            is_error,
            keybinds: " q:quit  j/k:navigate  enter:select ",
        }
    }

    pub fn with_keybinds(mut self, keybinds: &'a str) -> Self {
        self.keybinds = keybinds;
        self
    }
}

impl Widget for StatusBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let bg = if self.is_error {
            theme::RED
        } else {
            theme::SURFACE0
        };

        let fg = if self.is_error {
            theme::CRUST
        } else {
            theme::SUBTEXT0
        };

        // fill background
        for x in area.x..area.x + area.width {
            buf.set_string(x, area.y, " ", Style::new().bg(bg));
        }

        // render message
        let msg = Span::styled(self.message, Style::new().fg(fg).bg(bg));
        buf.set_span(area.x + 1, area.y, &msg, area.width.saturating_sub(2));

        // render keybinds on right
        let kb_span = Span::styled(self.keybinds, Style::new().fg(theme::OVERLAY0).bg(bg));
        let kb_x = area.x + area.width.saturating_sub(self.keybinds.len() as u16 + 1);
        buf.set_span(kb_x, area.y, &kb_span, self.keybinds.len() as u16);
    }
}

