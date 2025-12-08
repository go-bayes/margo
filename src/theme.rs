// theme support with catppuccin mocha as default
// https://github.com/catppuccin/catppuccin

use nu_ansi_term::{Color, Style};
use std::sync::atomic::{AtomicU8, Ordering};

// theme modes
const THEME_CATPPUCCIN: u8 = 0;
const THEME_BASIC: u8 = 1;
const THEME_PLAIN: u8 = 2;

static THEME_MODE: AtomicU8 = AtomicU8::new(THEME_CATPPUCCIN);

/// set the active theme mode
pub fn set_theme(name: &str) {
    let mode = match name.to_lowercase().as_str() {
        "basic" | "16" => THEME_BASIC,
        "plain" | "none" | "off" => THEME_PLAIN,
        _ => THEME_CATPPUCCIN, // default
    };
    THEME_MODE.store(mode, Ordering::Relaxed);
}

/// get the current theme mode
fn mode() -> u8 {
    THEME_MODE.load(Ordering::Relaxed)
}

// catppuccin mocha palette (rgb)
pub const ROSEWATER: Color = Color::Rgb(245, 224, 220);
pub const FLAMINGO: Color = Color::Rgb(242, 205, 205);
pub const PINK: Color = Color::Rgb(245, 194, 231);
pub const MAUVE: Color = Color::Rgb(203, 166, 247);
pub const RED: Color = Color::Rgb(243, 139, 168);
pub const MAROON: Color = Color::Rgb(235, 160, 172);
pub const PEACH: Color = Color::Rgb(250, 179, 135);
pub const YELLOW: Color = Color::Rgb(249, 226, 175);
pub const GREEN: Color = Color::Rgb(166, 227, 161);
pub const TEAL: Color = Color::Rgb(148, 226, 213);
pub const SKY: Color = Color::Rgb(137, 220, 235);
pub const SAPPHIRE: Color = Color::Rgb(116, 199, 236);
pub const BLUE: Color = Color::Rgb(137, 180, 250);
pub const LAVENDER: Color = Color::Rgb(180, 190, 254);

// catppuccin mocha base colours
pub const TEXT: Color = Color::Rgb(205, 214, 244);
pub const SUBTEXT1: Color = Color::Rgb(186, 194, 222);
pub const SUBTEXT0: Color = Color::Rgb(166, 173, 200);
pub const OVERLAY2: Color = Color::Rgb(147, 153, 178);
pub const OVERLAY1: Color = Color::Rgb(127, 132, 156);
pub const OVERLAY0: Color = Color::Rgb(108, 112, 134);
pub const SURFACE2: Color = Color::Rgb(88, 91, 112);
pub const SURFACE1: Color = Color::Rgb(69, 71, 90);
pub const SURFACE0: Color = Color::Rgb(49, 50, 68);
pub const BASE: Color = Color::Rgb(30, 30, 46);
pub const MANTLE: Color = Color::Rgb(24, 24, 37);
pub const CRUST: Color = Color::Rgb(17, 17, 27);

// basic 16-colour fallbacks (ansi)
mod basic {
    use nu_ansi_term::Color;
    pub const PINK: Color = Color::LightMagenta;
    pub const MAUVE: Color = Color::Magenta;
    pub const RED: Color = Color::Red;
    pub const PEACH: Color = Color::Yellow;
    pub const YELLOW: Color = Color::LightYellow;
    pub const GREEN: Color = Color::Green;
    pub const TEAL: Color = Color::Cyan;
    pub const SAPPHIRE: Color = Color::LightBlue;
    pub const TEXT: Color = Color::White;
    pub const SUBTEXT0: Color = Color::LightGray;
    pub const OVERLAY0: Color = Color::DarkGray;
}

// style helpers - theme-aware
pub fn pink() -> Style {
    match mode() {
        THEME_PLAIN => Style::new(),
        THEME_BASIC => Style::new().fg(basic::PINK),
        _ => Style::new().fg(PINK),
    }
}

pub fn mauve() -> Style {
    match mode() {
        THEME_PLAIN => Style::new(),
        THEME_BASIC => Style::new().fg(basic::MAUVE),
        _ => Style::new().fg(MAUVE),
    }
}

pub fn teal() -> Style {
    match mode() {
        THEME_PLAIN => Style::new(),
        THEME_BASIC => Style::new().fg(basic::TEAL),
        _ => Style::new().fg(TEAL),
    }
}

pub fn peach() -> Style {
    match mode() {
        THEME_PLAIN => Style::new(),
        THEME_BASIC => Style::new().fg(basic::PEACH),
        _ => Style::new().fg(PEACH),
    }
}

pub fn green() -> Style {
    match mode() {
        THEME_PLAIN => Style::new(),
        THEME_BASIC => Style::new().fg(basic::GREEN),
        _ => Style::new().fg(GREEN),
    }
}

pub fn red() -> Style {
    match mode() {
        THEME_PLAIN => Style::new(),
        THEME_BASIC => Style::new().fg(basic::RED),
        _ => Style::new().fg(RED),
    }
}

pub fn yellow() -> Style {
    match mode() {
        THEME_PLAIN => Style::new(),
        THEME_BASIC => Style::new().fg(basic::YELLOW),
        _ => Style::new().fg(YELLOW),
    }
}

pub fn sapphire() -> Style {
    match mode() {
        THEME_PLAIN => Style::new(),
        THEME_BASIC => Style::new().fg(basic::SAPPHIRE),
        _ => Style::new().fg(SAPPHIRE),
    }
}

pub fn text() -> Style {
    match mode() {
        THEME_PLAIN => Style::new(),
        THEME_BASIC => Style::new().fg(basic::TEXT),
        _ => Style::new().fg(TEXT),
    }
}

pub fn subtext0() -> Style {
    match mode() {
        THEME_PLAIN => Style::new(),
        THEME_BASIC => Style::new().fg(basic::SUBTEXT0),
        _ => Style::new().fg(SUBTEXT0),
    }
}

pub fn subtext1() -> Style {
    match mode() {
        THEME_PLAIN => Style::new(),
        THEME_BASIC => Style::new().fg(basic::SUBTEXT0), // fallback
        _ => Style::new().fg(SUBTEXT1),
    }
}

pub fn overlay0() -> Style {
    match mode() {
        THEME_PLAIN => Style::new(),
        THEME_BASIC => Style::new().fg(basic::OVERLAY0),
        _ => Style::new().fg(OVERLAY0),
    }
}

pub fn surface1() -> Style {
    match mode() {
        THEME_PLAIN => Style::new(),
        THEME_BASIC => Style::new().fg(basic::OVERLAY0), // fallback
        _ => Style::new().fg(SURFACE1),
    }
}

// semantic aliases - theme-aware
pub fn error() -> Style {
    red()
}

pub fn success() -> Style {
    green()
}

pub fn warning() -> Style {
    yellow()
}

pub fn highlight() -> Style {
    mauve()
}

pub fn accent() -> Style {
    teal()
}

pub fn dim() -> Style {
    overlay0()
}

// margo ascii logo
pub const LOGO: &str = r#"
  ███╗   ███╗ █████╗ ██████╗  ██████╗  ██████╗
  ████╗ ████║██╔══██╗██╔══██╗██╔════╝ ██╔═══██╗
  ██╔████╔██║███████║██████╔╝██║  ███╗██║   ██║
  ██║╚██╔╝██║██╔══██║██╔══██╗██║   ██║██║   ██║
  ██║ ╚═╝ ██║██║  ██║██║  ██║╚██████╔╝╚██████╔╝
  ╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝  ╚═════╝
"#;

// tagline
pub const TAGLINE: &str = "scaffolding for margot causal inference";
