// tui application state and main loop

use std::io;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame, Terminal,
};

use super::{theme, widgets::*};
use crate::config::Defaults;

/// application state
#[derive(Debug, Clone, PartialEq)]
enum Screen {
    Welcome,
    TemplateSelect,
    ProjectName,
    ConfigureDataSource,  // text input + fzf option
    ConfigureOutputRoot,  // text input + fzf option
    ConfigureExposure,
    ConfigureWhoMode,
    Review,
    Creating,
    Done,
}

/// which field is being edited
#[derive(Debug, Clone, PartialEq)]
enum EditingField {
    None,
    ExposureName,
}

/// project configuration being built
#[derive(Debug, Clone)]
struct ProjectConfig {
    template: String,
    name: String,
    pull_data: String,      // where data lives
    output_root: String,    // root for projects
    exposure: String,
    who_mode: usize,        // 0=default, 1=cat, 2=num
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            template: String::new(),
            name: String::new(),
            pull_data: String::new(),
            output_root: String::new(),
            exposure: String::from("exposure_var"),
            who_mode: 0,
        }
    }
}

impl ProjectConfig {
    /// generate suggested project folder name
    fn suggested_folder(&self) -> String {
        let year = chrono_year();
        let exp = if self.exposure.is_empty() { "study" } else { &self.exposure };
        format!("{}-{}-{}", year, exp, self.name)
    }

    /// full path where project will be created
    fn full_project_path(&self) -> String {
        if self.output_root.is_empty() {
            self.suggested_folder()
        } else {
            format!("{}/{}", self.output_root, self.suggested_folder())
        }
    }
}

/// get current year
fn chrono_year() -> u32 {
    // simple approach: parse from system
    use std::process::Command;
    if let Ok(output) = Command::new("date").arg("+%Y").output() {
        if let Ok(s) = String::from_utf8(output.stdout) {
            if let Ok(y) = s.trim().parse() {
                return y;
            }
        }
    }
    2025 // fallback
}

/// main app state
struct App {
    screen: Screen,
    config: ProjectConfig,
    menu_items: Vec<MenuItem>,
    selected_menu: usize,
    input_buffer: String,
    cursor_pos: usize,
    status_message: String,
    status_is_error: bool,
    should_quit: bool,
    tick: u64,
    editing_field: EditingField,
}

impl App {
    fn new() -> Self {
        let menu_items = vec![
            MenuItem::new(
                "GRF  - Generalised Random Forests",
                "Heterogeneous treatment effects with causal forests",
                'g',
            ),
            MenuItem::new(
                "LMTP - Longitudinal Modified Treatment Policies",
                "Coming soon...",
                'l',
            ),
        ];

        // load user config from ~/.config/margo/config.toml
        let user_config = Defaults::load();

        // pre-fill config from user settings
        let who_mode = match user_config.who_mode.as_deref() {
            Some("cat") => 1,
            Some("num") => 2,
            _ => 0,
        };

        let config = ProjectConfig {
            template: String::new(),
            name: String::new(),
            pull_data: user_config.pull_data.clone().unwrap_or_default(),
            output_root: user_config.push_mods.clone().unwrap_or_default(),
            exposure: String::from("exposure_var"),
            who_mode,
        };

        Self {
            screen: Screen::Welcome,
            config,
            menu_items,
            selected_menu: 0,
            input_buffer: String::new(),
            cursor_pos: 0,
            status_message: String::from("press enter or space to continue"),
            status_is_error: false,
            should_quit: false,
            tick: 0,
            editing_field: EditingField::None,
        }
    }

    fn handle_key(&mut self, key: KeyCode) {
        match &self.screen {
            Screen::Welcome => match key {
                KeyCode::Enter | KeyCode::Char(' ') => {
                    self.screen = Screen::TemplateSelect;
                    self.status_message = String::from("select a template with arrow keys, press enter");
                }
                KeyCode::Char('q') | KeyCode::Esc => {
                    self.should_quit = true;
                }
                _ => {}
            }
            Screen::TemplateSelect => match key {
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.selected_menu > 0 {
                        self.selected_menu -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.selected_menu < self.menu_items.len() - 1 {
                        self.selected_menu += 1;
                    }
                }
                KeyCode::Enter | KeyCode::Char('g') => {
                    if self.selected_menu == 0 {
                        self.config.template = String::from("grf");
                        self.screen = Screen::ProjectName;
                        self.status_message = String::from("enter project name");
                    }
                }
                KeyCode::Char('l') => {
                    self.status_message = String::from("LMTP template coming soon");
                    self.status_is_error = true;
                }
                KeyCode::Char('q') | KeyCode::Esc => {
                    self.should_quit = true;
                }
                _ => {}
            },
            Screen::ProjectName => match key {
                KeyCode::Char(c) => {
                    // only allow valid project name chars
                    if c.is_alphanumeric() || c == '-' || c == '_' {
                        self.input_buffer.insert(self.cursor_pos, c);
                        self.cursor_pos += 1;
                        self.status_is_error = false;
                    }
                }
                KeyCode::Backspace => {
                    if self.cursor_pos > 0 {
                        self.cursor_pos -= 1;
                        self.input_buffer.remove(self.cursor_pos);
                    }
                }
                KeyCode::Left => {
                    if self.cursor_pos > 0 {
                        self.cursor_pos -= 1;
                    }
                }
                KeyCode::Right => {
                    if self.cursor_pos < self.input_buffer.len() {
                        self.cursor_pos += 1;
                    }
                }
                KeyCode::Enter => {
                    if self.input_buffer.is_empty() {
                        self.status_message = String::from("project name cannot be empty");
                        self.status_is_error = true;
                    } else {
                        self.config.name = self.input_buffer.clone();
                        // go to data source config - use saved default or home
                        self.input_buffer = if self.config.pull_data.is_empty() {
                            dirs::home_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_default()
                        } else {
                            self.config.pull_data.clone()
                        };
                        self.cursor_pos = self.input_buffer.len();
                        self.screen = Screen::ConfigureDataSource;
                        self.status_message = String::from("enter data source path");
                    }
                }
                KeyCode::Esc => {
                    self.screen = Screen::TemplateSelect;
                    self.input_buffer.clear();
                    self.cursor_pos = 0;
                    self.status_message = String::from("select a template type");
                }
                _ => {}
            },
            Screen::ConfigureDataSource | Screen::ConfigureOutputRoot => {
                match key {
                    KeyCode::Char(c) => {
                        self.input_buffer.insert(self.cursor_pos, c);
                        self.cursor_pos += 1;
                    }
                    KeyCode::Backspace => {
                        if self.cursor_pos > 0 {
                            self.cursor_pos -= 1;
                            self.input_buffer.remove(self.cursor_pos);
                        }
                    }
                    KeyCode::Left => {
                        if self.cursor_pos > 0 {
                            self.cursor_pos -= 1;
                        }
                    }
                    KeyCode::Right => {
                        if self.cursor_pos < self.input_buffer.len() {
                            self.cursor_pos += 1;
                        }
                    }
                    KeyCode::Tab | KeyCode::Enter => {
                        let path = self.input_buffer.clone();
                        if self.screen == Screen::ConfigureDataSource {
                            self.config.pull_data = path;
                            // next: output root - use saved default or home
                            self.input_buffer = if self.config.output_root.is_empty() {
                                dirs::home_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_default()
                            } else {
                                self.config.output_root.clone()
                            };
                            self.cursor_pos = self.input_buffer.len();
                            self.screen = Screen::ConfigureOutputRoot;
                            self.status_message = String::from("enter output root path");
                        } else {
                            self.config.output_root = path;
                            // next: exposure
                            self.screen = Screen::ConfigureExposure;
                            self.input_buffer = self.config.exposure.clone();
                            self.cursor_pos = self.input_buffer.len();
                            self.status_message = String::from("enter exposure variable name");
                        }
                    }
                    KeyCode::Esc => {
                        if self.screen == Screen::ConfigureDataSource {
                            self.screen = Screen::ProjectName;
                            self.input_buffer = self.config.name.clone();
                            self.cursor_pos = self.input_buffer.len();
                            self.status_message = String::from("enter project name");
                        } else {
                            self.screen = Screen::ConfigureDataSource;
                            self.input_buffer = self.config.pull_data.clone();
                            self.cursor_pos = self.input_buffer.len();
                            self.status_message = String::from("enter data source path");
                        }
                    }
                    _ => {}
                }
            }
            Screen::ConfigureExposure => {
                match key {
                    KeyCode::Char(c) => {
                        if c.is_alphanumeric() || c == '_' {
                            // start editing if not already
                            if self.editing_field != EditingField::ExposureName {
                                self.editing_field = EditingField::ExposureName;
                                self.input_buffer = self.config.exposure.clone();
                                self.cursor_pos = self.input_buffer.len();
                            }
                            self.input_buffer.insert(self.cursor_pos, c);
                            self.cursor_pos += 1;
                        }
                    }
                    KeyCode::Backspace => {
                        if self.editing_field == EditingField::ExposureName && self.cursor_pos > 0 {
                            self.cursor_pos -= 1;
                            self.input_buffer.remove(self.cursor_pos);
                        }
                    }
                    KeyCode::Left => {
                        if self.cursor_pos > 0 {
                            self.cursor_pos -= 1;
                        }
                    }
                    KeyCode::Right => {
                        if self.cursor_pos < self.input_buffer.len() {
                            self.cursor_pos += 1;
                        }
                    }
                    KeyCode::Tab | KeyCode::Enter => {
                        // save and continue
                        if self.editing_field == EditingField::ExposureName {
                            self.config.exposure = self.input_buffer.clone();
                            self.editing_field = EditingField::None;
                            self.input_buffer.clear();
                        }
                        self.screen = Screen::ConfigureWhoMode;
                        self.status_message = String::from("select BMI/exercise variable mode");
                    }
                    KeyCode::Esc => {
                        self.editing_field = EditingField::None;
                        self.input_buffer = self.config.output_root.clone();
                        self.cursor_pos = self.input_buffer.len();
                        self.screen = Screen::ConfigureOutputRoot;
                        self.status_message = String::from("enter output root path");
                    }
                    _ => {}
                }
            }
            Screen::ConfigureWhoMode => match key {
                KeyCode::Char('q') => {
                    self.should_quit = true;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.config.who_mode > 0 {
                        self.config.who_mode -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.config.who_mode < 2 {
                        self.config.who_mode += 1;
                    }
                }
                KeyCode::Char('1') => self.config.who_mode = 0,
                KeyCode::Char('2') => self.config.who_mode = 1,
                KeyCode::Char('3') => self.config.who_mode = 2,
                KeyCode::Tab | KeyCode::Enter => {
                    self.screen = Screen::Review;
                    self.status_message = String::from("review configuration");
                }
                KeyCode::Esc => {
                    self.screen = Screen::ConfigureExposure;
                    self.status_message = String::from("configure exposure variable");
                }
                _ => {}
            },
            Screen::Review => match key {
                KeyCode::Char('q') => {
                    self.should_quit = true;
                }
                KeyCode::Enter => {
                    self.screen = Screen::Creating;
                    self.status_message = String::from("creating project...");
                }
                KeyCode::Esc => {
                    self.screen = Screen::ConfigureWhoMode;
                    self.status_message = String::from("select BMI/exercise variable mode");
                }
                _ => {}
            },
            Screen::Creating => {
                // handled in main loop
            }
            Screen::Done => {
                self.should_quit = true;
            }
        }
    }
}

fn ui(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // clear with background colour
    frame.render_widget(
        Block::default().style(Style::new().bg(theme::BASE)),
        area,
    );

    // layout: main area + status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(1)])
        .split(area);

    let main_area = chunks[0];
    let status_area = chunks[1];

    match &app.screen {
        Screen::Welcome => {
            // split: logo left, map right (map takes more space)
            let h_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(55), Constraint::Min(35)])
                .split(main_area);

            frame.render_widget(WelcomeScreen::new(app.tick), h_chunks[0]);

            // nz map on right - full height
            if h_chunks[1].width >= 25 {
                let map_area = Rect {
                    x: h_chunks[1].x,
                    y: h_chunks[1].y,
                    width: h_chunks[1].width,
                    height: h_chunks[1].height,
                };
                frame.render_widget(NzMapCanvas::new(app.tick), map_area);
            }
        }
        Screen::TemplateSelect => {
            // split: left content, right map
            let h_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                .split(main_area);

            // welcome header (smaller, no blink)
            let header_area = Rect {
                x: h_chunks[0].x,
                y: h_chunks[0].y,
                width: h_chunks[0].width,
                height: 11,
            };
            frame.render_widget(WelcomeScreen::default(), header_area);

            // menu below - make it visible!
            let menu_area = Rect {
                x: h_chunks[0].x + 2,
                y: h_chunks[0].y + 12,
                width: h_chunks[0].width.saturating_sub(4),
                height: 10,
            };
            frame.render_widget(
                TemplateMenu::new(&app.menu_items, app.selected_menu),
                menu_area,
            );

            // nz map on right - full height
            if h_chunks[1].width >= 20 {
                frame.render_widget(NzMapCanvas::new(app.tick), h_chunks[1]);
            }
        }
        Screen::ProjectName => {
            // header
            let header = Paragraph::new(vec![
                Line::from(Span::styled(
                    "New GRF Project",
                    Style::new().fg(theme::TEAL).bold(),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Enter a name for your project directory.",
                    Style::new().fg(theme::SUBTEXT0),
                )),
                Line::from(Span::styled(
                    "Use lowercase letters, numbers, and hyphens.",
                    Style::new().fg(theme::SUBTEXT0),
                )),
            ]);

            let header_area = Rect {
                x: main_area.x + 4,
                y: main_area.y + 2,
                width: main_area.width.saturating_sub(8),
                height: 5,
            };
            frame.render_widget(header, header_area);

            // input field
            let input_area = Rect {
                x: main_area.x + 4,
                y: main_area.y + 8,
                width: main_area.width.saturating_sub(8).min(50),
                height: 3,
            };
            frame.render_widget(
                InputField::new("Project Name", &app.input_buffer, true, app.cursor_pos),
                input_area,
            );
        }
        Screen::ConfigureDataSource | Screen::ConfigureOutputRoot => {
            let (title, subtitle, label) = if app.screen == Screen::ConfigureDataSource {
                ("Data Source Directory", "Where your .qs data files are stored", "Path")
            } else {
                ("Output Root Directory", "Where project folders will be created", "Path")
            };

            let content_area = Rect {
                x: main_area.x + 4,
                y: main_area.y + 2,
                width: main_area.width.saturating_sub(8),
                height: main_area.height.saturating_sub(4),
            };

            // header
            let header = Paragraph::new(vec![
                Line::from(Span::styled(title, Style::new().fg(theme::TEAL).bold())),
                Line::from(""),
                Line::from(Span::styled(subtitle, Style::new().fg(theme::SUBTEXT0))),
            ]);
            frame.render_widget(header, content_area);

            // path input
            let input_area = Rect {
                x: content_area.x,
                y: content_area.y + 5,
                width: content_area.width.min(70),
                height: 3,
            };
            frame.render_widget(
                InputField::new(label, &app.input_buffer, true, app.cursor_pos),
                input_area,
            );

            // hint
            let hint = "Type path directly | Tab/Enter to continue | Esc to go back | q to quit";
            let hint_para = Paragraph::new(Line::from(Span::styled(hint, Style::new().fg(theme::OVERLAY0).italic())));
            let hint_area = Rect {
                x: content_area.x,
                y: content_area.y + 9,
                width: 72,
                height: 1,
            };
            frame.render_widget(hint_para, hint_area);
        }
        Screen::ConfigureExposure => {
            let content_area = Rect {
                x: main_area.x + 4,
                y: main_area.y + 2,
                width: main_area.width.saturating_sub(8),
                height: main_area.height.saturating_sub(4),
            };

            let header = Paragraph::new(vec![
                Line::from(Span::styled(
                    "Configure Exposure Variable",
                    Style::new().fg(theme::TEAL).bold(),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Enter your exposure variable name (without wave prefix).",
                    Style::new().fg(theme::SUBTEXT0),
                )),
            ]);
            frame.render_widget(header, content_area);

            let editing = app.editing_field == EditingField::ExposureName;
            let value = if editing { &app.input_buffer } else { &app.config.exposure };
            let input_area = Rect {
                x: content_area.x,
                y: content_area.y + 5,
                width: content_area.width.min(50),
                height: 3,
            };
            frame.render_widget(
                InputField::new("Exposure Variable", value, true, if editing { app.cursor_pos } else { 0 }),
                input_area,
            );

            let hint = if editing { "Type variable name, Enter/Tab to continue" } else { "Start typing to edit, Tab to continue" };
            let hint_para = Paragraph::new(Line::from(Span::styled(hint, Style::new().fg(theme::OVERLAY0).italic())));
            let hint_area = Rect {
                x: content_area.x,
                y: content_area.y + 9,
                width: 50,
                height: 1,
            };
            frame.render_widget(hint_para, hint_area);
        }
        Screen::ConfigureWhoMode => {
            let content_area = Rect {
                x: main_area.x + 4,
                y: main_area.y + 2,
                width: main_area.width.saturating_sub(8),
                height: main_area.height.saturating_sub(4),
            };

            let header = Paragraph::new(vec![
                Line::from(Span::styled(
                    "BMI / Exercise Variable Mode",
                    Style::new().fg(theme::TEAL).bold(),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Select how BMI and exercise hours are represented.",
                    Style::new().fg(theme::SUBTEXT0),
                )),
            ]);
            frame.render_widget(header, content_area);

            let modes = [
                ("default", "hlth_bmi, log_hours_exercise (continuous)"),
                ("cat", "bmi_cat, who_hours_exercise_cat (categorical)"),
                ("num", "bmi_cat_num, who_hours_exercise_num (ordinal numeric)"),
            ];

            for (i, (mode, desc)) in modes.iter().enumerate() {
                let selected = app.config.who_mode == i;
                let style = if selected {
                    Style::new().fg(theme::MAUVE).bold()
                } else {
                    Style::new().fg(theme::TEXT)
                };
                let prefix = if selected { "▶ " } else { "  " };
                let line = Paragraph::new(Line::from(vec![
                    Span::styled(prefix, Style::new().fg(theme::GREEN)),
                    Span::styled(format!("{}: ", mode), style),
                    Span::styled(*desc, Style::new().fg(theme::SUBTEXT0)),
                ]));
                let line_area = Rect {
                    x: content_area.x,
                    y: content_area.y + 5 + i as u16 * 2,
                    width: content_area.width,
                    height: 1,
                };
                frame.render_widget(line, line_area);
            }
        }
        Screen::Review => {
            let content_area = Rect {
                x: main_area.x + 4,
                y: main_area.y + 2,
                width: main_area.width.saturating_sub(8),
                height: main_area.height.saturating_sub(4),
            };

            let who_mode_str = match app.config.who_mode {
                0 => "default",
                1 => "cat",
                2 => "num",
                _ => "default",
            };

            let project_path = app.config.full_project_path();

            let review = Paragraph::new(vec![
                Line::from(Span::styled(
                    "Review Configuration",
                    Style::new().fg(theme::TEAL).bold(),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Template:     ", Style::new().fg(theme::SUBTEXT0)),
                    Span::styled(&app.config.template, Style::new().fg(theme::TEXT)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Data source:  ", Style::new().fg(theme::SUBTEXT0)),
                    Span::styled(&app.config.pull_data, Style::new().fg(theme::TEXT)),
                ]),
                Line::from(vec![
                    Span::styled("Output root:  ", Style::new().fg(theme::SUBTEXT0)),
                    Span::styled(&app.config.output_root, Style::new().fg(theme::TEXT)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Exposure:     ", Style::new().fg(theme::SUBTEXT0)),
                    Span::styled(&app.config.exposure, Style::new().fg(theme::TEXT)),
                ]),
                Line::from(vec![
                    Span::styled("WHO mode:     ", Style::new().fg(theme::SUBTEXT0)),
                    Span::styled(who_mode_str, Style::new().fg(theme::TEXT)),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    "Project will be created at:",
                    Style::new().fg(theme::PEACH),
                )),
                Line::from(Span::styled(
                    format!("  {}", project_path),
                    Style::new().fg(theme::MAUVE).bold(),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Press Enter to create, Esc to go back",
                    Style::new().fg(theme::GREEN).italic(),
                )),
            ]);
            frame.render_widget(review, content_area);
        }
        Screen::Creating => {
            let msg = Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled(
                    "Creating project...",
                    Style::new().fg(theme::TEAL).bold(),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    &app.config.name,
                    Style::new().fg(theme::MAUVE),
                )),
            ])
            .centered();

            frame.render_widget(msg, main_area);
        }
        Screen::Done => {
            let content_area = Rect {
                x: main_area.x + 4,
                y: main_area.y + 2,
                width: main_area.width.saturating_sub(8),
                height: main_area.height.saturating_sub(4),
            };

            let project_path = app.config.full_project_path();

            let msg = Paragraph::new(vec![
                Line::from(Span::styled(
                    "Project created successfully!",
                    Style::new().fg(theme::GREEN).bold(),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Location: ", Style::new().fg(theme::SUBTEXT0)),
                    Span::styled(&project_path, Style::new().fg(theme::TEXT)),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    "Files created:",
                    Style::new().fg(theme::PEACH),
                )),
                Line::from(Span::styled(
                    "  study.toml         - configuration (edit this first)",
                    Style::new().fg(theme::LAVENDER),
                )),
                Line::from(Span::styled(
                    "  01-data-prep.R     - data wrangling",
                    Style::new().fg(theme::SUBTEXT0),
                )),
                Line::from(Span::styled(
                    "  02-wide-format.R   - wide format + IPCW weights",
                    Style::new().fg(theme::SUBTEXT0),
                )),
                Line::from(Span::styled(
                    "  03-causal-forest.R - causal forest estimation",
                    Style::new().fg(theme::SUBTEXT0),
                )),
                Line::from(Span::styled(
                    "  04-08              - heterogeneity, policy, tables, plots",
                    Style::new().fg(theme::OVERLAY0),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Next steps:",
                    Style::new().fg(theme::PEACH),
                )),
                Line::from(Span::styled(
                    format!("  cd {}", app.config.name),
                    Style::new().fg(theme::MAUVE),
                )),
                Line::from(Span::styled(
                    "  # edit study.toml with your exposure, outcomes, paths",
                    Style::new().fg(theme::OVERLAY0),
                )),
                Line::from(Span::styled(
                    "  # run scripts in order: 01, 02, 03...",
                    Style::new().fg(theme::OVERLAY0),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Press any key to exit",
                    Style::new().fg(theme::OVERLAY0).italic(),
                )),
            ]);

            frame.render_widget(msg, content_area);
        }
    }

    // status bar with screen-specific keybinds
    let keybinds = match &app.screen {
        Screen::Welcome => " enter:continue  q:quit ",
        Screen::TemplateSelect => " j/k:navigate  enter:select  q:quit ",
        Screen::ProjectName => " enter:continue  esc:back ",
        Screen::ConfigureDataSource | Screen::ConfigureOutputRoot => " type/paste path  tab:continue  esc:back ",
        Screen::ConfigureExposure => " type to edit  tab:next  esc:back ",
        Screen::ConfigureWhoMode => " j/k:select  enter:next  esc:back  q:quit ",
        Screen::Review => " enter:create  esc:back  q:quit ",
        Screen::Done => " any key:exit ",
        _ => " q:quit ",
    };

    frame.render_widget(
        StatusBar::new(&app.status_message, app.status_is_error).with_keybinds(keybinds),
        status_area,
    );
}

/// run the tui application
pub fn run() -> Result<String> {
    use std::time::Duration;

    // setup terminal
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    // main loop with animation
    loop {
        terminal.draw(|frame| ui(frame, &app))?;

        // poll for events with timeout for animation
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.handle_key(key.code);
                }
            }
        }

        // increment tick for animations
        app.tick = app.tick.wrapping_add(1);

        if app.should_quit {
            break;
        }

        // if we're in creating state, actually create the project
        if app.screen == Screen::Creating {
            let project_path = app.config.full_project_path();

            // do the actual creation (quiet mode - no stdout)
            let result = crate::commands::init::grf_quiet(&project_path);
            match result {
                Ok(_) => {
                    app.screen = Screen::Done;
                    app.status_message = format!("created: {}", project_path);
                }
                Err(e) => {
                    app.status_message = format!("error: {}", e);
                    app.status_is_error = true;
                    app.screen = Screen::Review;
                }
            }
        }
    }

    // restore terminal
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    Ok(app.config.name)
}
