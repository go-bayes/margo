// interactive directory picker for selecting paths
// uses crossterm for terminal control
// (currently unused - planned for future path selection feature)

#![allow(dead_code)]

use anyhow::Result;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{stdout, Write};
use std::path::{Path, PathBuf};

use crate::theme;

/// result of directory picker
pub enum PickerResult {
    Selected(PathBuf),
    Cancelled,
}

/// pick a directory path interactively
pub fn pick_directory(prompt: &str, start_path: Option<&Path>) -> Result<PickerResult> {
    let start = start_path
        .map(PathBuf::from)
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")));

    let mut picker = DirectoryPicker::new(prompt, start)?;
    picker.run()
}

/// pick a directory with common options shown first
pub fn pick_output_directory(prompt: &str) -> Result<PickerResult> {
    // offer common choices first
    let current = std::env::current_dir().ok();
    let home = dirs::home_dir();

    let mut options: Vec<(&str, PathBuf)> = Vec::new();

    if let Some(ref cwd) = current {
        options.push(("./outputs/ (project subdirectory)", cwd.join("outputs")));
        options.push(("Current directory", cwd.clone()));
    }
    if let Some(ref h) = home {
        options.push(("Home directory", h.clone()));
        let documents = h.join("Documents");
        if documents.exists() {
            options.push(("Documents", documents));
        }
    }
    options.push(("Browse...", current.clone().unwrap_or_else(|| PathBuf::from("/"))));

    // use inquire for initial selection
    let labels: Vec<&str> = options.iter().map(|(l, _)| *l).collect();

    let result = inquire::Select::new(prompt, labels)
        .with_page_size(6)
        .with_help_message("↑↓ navigate, Enter select")
        .prompt_skippable()?;

    match result {
        Some(label) => {
            if label == "Browse..." {
                // launch full directory picker
                let start = current.unwrap_or_else(|| PathBuf::from("/"));
                pick_directory("Select directory:", Some(&start))
            } else {
                // find the path for this label
                let path = options
                    .iter()
                    .find(|(l, _)| *l == label)
                    .map(|(_, p)| p.clone())
                    .unwrap_or_else(|| PathBuf::from("."));
                Ok(PickerResult::Selected(path))
            }
        }
        None => Ok(PickerResult::Cancelled),
    }
}

/// directory picker state
struct DirectoryPicker {
    prompt: String,
    current_dir: PathBuf,
    entries: Vec<DirEntry>,
    selected: usize,
    scroll_offset: usize,
    visible_rows: usize,
    filter: String,
}

#[derive(Clone)]
struct DirEntry {
    name: String,
    path: PathBuf,
    is_dir: bool,
}

impl DirectoryPicker {
    fn new(prompt: &str, start: PathBuf) -> Result<Self> {
        let mut picker = Self {
            prompt: prompt.to_string(),
            current_dir: start,
            entries: Vec::new(),
            selected: 0,
            scroll_offset: 0,
            visible_rows: 15,
            filter: String::new(),
        };
        picker.refresh_entries()?;
        Ok(picker)
    }

    fn refresh_entries(&mut self) -> Result<()> {
        self.entries.clear();

        // add parent directory entry if not at root
        if let Some(parent) = self.current_dir.parent() {
            self.entries.push(DirEntry {
                name: "..".to_string(),
                path: parent.to_path_buf(),
                is_dir: true,
            });
        }

        // read directory contents
        if let Ok(read_dir) = std::fs::read_dir(&self.current_dir) {
            let mut dirs: Vec<DirEntry> = Vec::new();
            let mut files: Vec<DirEntry> = Vec::new();

            for entry in read_dir.flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();

                // skip hidden files unless filter starts with .
                if name.starts_with('.') && !self.filter.starts_with('.') {
                    continue;
                }

                // apply filter if set
                if !self.filter.is_empty() {
                    let filter_lower = self.filter.to_lowercase();
                    let name_lower = name.to_lowercase();
                    if !name_lower.contains(&filter_lower) {
                        continue;
                    }
                }

                let is_dir = path.is_dir();
                let entry = DirEntry { name, path, is_dir };

                if is_dir {
                    dirs.push(entry);
                } else {
                    files.push(entry);
                }
            }

            // sort alphabetically
            dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
            files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

            // directories first, then files
            self.entries.extend(dirs);
            self.entries.extend(files);
        }

        // reset selection if out of bounds
        if self.selected >= self.entries.len() {
            self.selected = 0;
        }

        Ok(())
    }

    fn run(&mut self) -> Result<PickerResult> {
        // enter raw mode and alternate screen
        terminal::enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, Hide)?;

        let result = self.event_loop();

        // restore terminal
        execute!(stdout, Show, LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;

        result
    }

    fn event_loop(&mut self) -> Result<PickerResult> {
        loop {
            self.render()?;

            if let Event::Key(key) = event::read()? {
                match (key.code, key.modifiers) {
                    // quit/cancel
                    (KeyCode::Esc, _) | (KeyCode::Char('q'), KeyModifiers::NONE) => {
                        return Ok(PickerResult::Cancelled);
                    }
                    (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                        return Ok(PickerResult::Cancelled);
                    }

                    // navigation
                    (KeyCode::Up, _) | (KeyCode::Char('k'), KeyModifiers::NONE) => {
                        self.move_up();
                    }
                    (KeyCode::Down, _) | (KeyCode::Char('j'), KeyModifiers::NONE) => {
                        self.move_down();
                    }
                    (KeyCode::PageUp, _) => {
                        for _ in 0..self.visible_rows {
                            self.move_up();
                        }
                    }
                    (KeyCode::PageDown, _) => {
                        for _ in 0..self.visible_rows {
                            self.move_down();
                        }
                    }
                    (KeyCode::Home, _) | (KeyCode::Char('g'), KeyModifiers::NONE) => {
                        self.selected = 0;
                        self.scroll_offset = 0;
                    }
                    (KeyCode::End, _) | (KeyCode::Char('G'), KeyModifiers::SHIFT) => {
                        if !self.entries.is_empty() {
                            self.selected = self.entries.len() - 1;
                            self.adjust_scroll();
                        }
                    }

                    // enter directory or go up
                    (KeyCode::Enter, _) | (KeyCode::Char('l'), KeyModifiers::NONE) => {
                        if let Some(entry) = self.entries.get(self.selected) {
                            if entry.is_dir {
                                self.current_dir = entry.path.clone();
                                self.selected = 0;
                                self.scroll_offset = 0;
                                self.filter.clear();
                                self.refresh_entries()?;
                            }
                        }
                    }

                    // go up (parent directory)
                    (KeyCode::Backspace, _) | (KeyCode::Char('h'), KeyModifiers::NONE) => {
                        if let Some(parent) = self.current_dir.parent() {
                            self.current_dir = parent.to_path_buf();
                            self.selected = 0;
                            self.scroll_offset = 0;
                            self.filter.clear();
                            self.refresh_entries()?;
                        }
                    }

                    // select current directory
                    (KeyCode::Tab, _) | (KeyCode::Char(' '), KeyModifiers::NONE) => {
                        return Ok(PickerResult::Selected(self.current_dir.clone()));
                    }

                    // filter: start typing
                    (KeyCode::Char('/'), KeyModifiers::NONE) => {
                        self.filter.clear();
                        // visual feedback that filter mode is active
                    }

                    // filter input (if not a navigation key)
                    (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                        // add to filter for alphanumeric
                        if c.is_alphanumeric() || c == '_' || c == '-' || c == '.' {
                            self.filter.push(c);
                            self.selected = 0;
                            self.scroll_offset = 0;
                            self.refresh_entries()?;
                        }
                    }

                    // clear filter
                    (KeyCode::Delete, _) => {
                        self.filter.clear();
                        self.refresh_entries()?;
                    }

                    _ => {}
                }
            }
        }
    }

    fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.adjust_scroll();
        }
    }

    fn move_down(&mut self) {
        if self.selected < self.entries.len().saturating_sub(1) {
            self.selected += 1;
            self.adjust_scroll();
        }
    }

    fn adjust_scroll(&mut self) {
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        } else if self.selected >= self.scroll_offset + self.visible_rows {
            self.scroll_offset = self.selected - self.visible_rows + 1;
        }
    }

    fn render(&self) -> Result<()> {
        let mut stdout = stdout();

        // clear screen and move to top
        execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;

        // header with prompt
        let pink = theme::pink();
        let teal = theme::teal();
        let text = theme::text();
        let overlay = theme::overlay0();
        let surface = theme::color_surface0();

        // prompt line
        println!(
            " {} {}",
            pink.paint("?"),
            text.bold().paint(&self.prompt)
        );
        println!();

        // current path
        let path_display = self.current_dir.display().to_string();
        println!(
            " {} {}",
            overlay.paint("Location:"),
            teal.paint(&path_display)
        );

        // filter indicator
        if !self.filter.is_empty() {
            println!(
                " {} {}",
                overlay.paint("Filter:"),
                theme::peach().paint(&self.filter)
            );
        }

        println!();

        // directory listing
        let visible_entries: Vec<_> = self
            .entries
            .iter()
            .enumerate()
            .skip(self.scroll_offset)
            .take(self.visible_rows)
            .collect();

        for (idx, entry) in &visible_entries {
            let is_selected = *idx == self.selected;

            let prefix = if is_selected { "❯" } else { " " };
            let prefix_style = if is_selected { pink } else { overlay };

            let name = if entry.is_dir {
                format!("{}/", entry.name)
            } else {
                entry.name.clone()
            };

            let name_style = if is_selected {
                if entry.is_dir {
                    teal.on(surface)
                } else {
                    text.on(surface)
                }
            } else if entry.is_dir {
                teal
            } else {
                text
            };

            println!(
                " {} {}",
                prefix_style.paint(prefix),
                name_style.paint(&name)
            );
        }

        // padding if fewer entries than visible rows
        let displayed = visible_entries.len();
        for _ in displayed..self.visible_rows {
            println!();
        }

        // scroll indicators
        if self.scroll_offset > 0 {
            println!(" {} more above", overlay.paint("▲"));
        } else {
            println!();
        }

        if self.scroll_offset + self.visible_rows < self.entries.len() {
            let remaining = self.entries.len() - self.scroll_offset - self.visible_rows;
            println!(" {} {} more below", overlay.paint("▼"), remaining);
        } else {
            println!();
        }

        println!();

        // help line
        println!(
            " {} navigate  {} open  {} up  {} select this dir  {} cancel",
            overlay.paint("↑↓"),
            overlay.paint("Enter"),
            overlay.paint("Backspace"),
            overlay.paint("Tab"),
            overlay.paint("q")
        );

        stdout.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dir_entry_creation() {
        let entry = DirEntry {
            name: "test".to_string(),
            path: PathBuf::from("/tmp/test"),
            is_dir: true,
        };
        assert!(entry.is_dir);
        assert_eq!(entry.name, "test");
    }
}
