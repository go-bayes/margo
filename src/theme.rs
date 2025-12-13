// theme support with catppuccin mocha (dark) and latte (light)
// https://github.com/catppuccin/catppuccin

use nu_ansi_term::{Color, Style};
use std::sync::atomic::{AtomicU8, Ordering};

// theme modes
const THEME_DARK: u8 = 0;   // catppuccin mocha
const THEME_LIGHT: u8 = 1;  // catppuccin latte
const THEME_BASIC: u8 = 2;  // 16-colour ansi
const THEME_PLAIN: u8 = 3;  // no colours

static THEME_MODE: AtomicU8 = AtomicU8::new(THEME_DARK);

/// set the active theme mode
pub fn set_theme(name: &str) {
    let mode = match name.to_lowercase().as_str() {
        "light" | "latte" => THEME_LIGHT,
        "dark" | "mocha" => THEME_DARK,
        "basic" | "16" => THEME_BASIC,
        "plain" | "none" | "off" => THEME_PLAIN,
        // legacy: "catppuccin" defaults to dark
        "catppuccin" => THEME_DARK,
        _ => THEME_DARK,
    };
    THEME_MODE.store(mode, Ordering::Relaxed);
}

/// get current theme name
pub fn current_theme() -> &'static str {
    match mode() {
        THEME_LIGHT => "light",
        THEME_DARK => "dark",
        THEME_BASIC => "basic",
        THEME_PLAIN => "plain",
        _ => "dark",
    }
}

/// toggle between light and dark themes
pub fn toggle_theme() {
    let current = mode();
    let new_mode = match current {
        THEME_DARK => THEME_LIGHT,
        THEME_LIGHT => THEME_DARK,
        _ => THEME_DARK, // reset to dark from basic/plain
    };
    THEME_MODE.store(new_mode, Ordering::Relaxed);
}

/// check if current theme is light
#[allow(dead_code)]
pub fn is_light() -> bool {
    mode() == THEME_LIGHT
}

fn mode() -> u8 {
    THEME_MODE.load(Ordering::Relaxed)
}

// catppuccin mocha palette (dark)
#[allow(dead_code)]
mod mocha {
    use nu_ansi_term::Color;
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
}

// catppuccin latte palette (light)
#[allow(dead_code)]
mod latte {
    use nu_ansi_term::Color;
    pub const ROSEWATER: Color = Color::Rgb(220, 138, 120);
    pub const FLAMINGO: Color = Color::Rgb(221, 120, 120);
    pub const PINK: Color = Color::Rgb(234, 118, 203);
    pub const MAUVE: Color = Color::Rgb(136, 57, 239);
    pub const RED: Color = Color::Rgb(210, 15, 57);
    pub const MAROON: Color = Color::Rgb(230, 69, 83);
    pub const PEACH: Color = Color::Rgb(254, 100, 11);
    pub const YELLOW: Color = Color::Rgb(223, 142, 29);
    pub const GREEN: Color = Color::Rgb(64, 160, 43);
    pub const TEAL: Color = Color::Rgb(23, 146, 153);
    pub const SKY: Color = Color::Rgb(4, 165, 229);
    pub const SAPPHIRE: Color = Color::Rgb(32, 159, 181);
    pub const BLUE: Color = Color::Rgb(30, 102, 245);
    pub const LAVENDER: Color = Color::Rgb(114, 135, 253);
    pub const TEXT: Color = Color::Rgb(76, 79, 105);
    pub const SUBTEXT1: Color = Color::Rgb(92, 95, 119);
    pub const SUBTEXT0: Color = Color::Rgb(108, 111, 133);
    pub const OVERLAY2: Color = Color::Rgb(124, 127, 147);
    pub const OVERLAY1: Color = Color::Rgb(140, 143, 161);
    pub const OVERLAY0: Color = Color::Rgb(156, 160, 176);
    pub const SURFACE2: Color = Color::Rgb(172, 176, 190);
    pub const SURFACE1: Color = Color::Rgb(188, 192, 204);
    pub const SURFACE0: Color = Color::Rgb(204, 208, 218);
    pub const BASE: Color = Color::Rgb(239, 241, 245);
    pub const MANTLE: Color = Color::Rgb(230, 233, 239);
    pub const CRUST: Color = Color::Rgb(220, 224, 232);
}

// basic 16-colour fallbacks (ansi)
#[allow(dead_code)]
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
        THEME_LIGHT => Style::new().fg(latte::PINK),
        _ => Style::new().fg(mocha::PINK),
    }
}

#[allow(dead_code)]
pub fn mauve() -> Style {
    match mode() {
        THEME_PLAIN => Style::new(),
        THEME_BASIC => Style::new().fg(basic::MAUVE),
        THEME_LIGHT => Style::new().fg(latte::MAUVE),
        _ => Style::new().fg(mocha::MAUVE),
    }
}

pub fn teal() -> Style {
    match mode() {
        THEME_PLAIN => Style::new(),
        THEME_BASIC => Style::new().fg(basic::TEAL),
        THEME_LIGHT => Style::new().fg(latte::TEAL),
        _ => Style::new().fg(mocha::TEAL),
    }
}

pub fn peach() -> Style {
    match mode() {
        THEME_PLAIN => Style::new(),
        THEME_BASIC => Style::new().fg(basic::PEACH),
        THEME_LIGHT => Style::new().fg(latte::PEACH),
        _ => Style::new().fg(mocha::PEACH),
    }
}

pub fn green() -> Style {
    match mode() {
        THEME_PLAIN => Style::new(),
        THEME_BASIC => Style::new().fg(basic::GREEN),
        THEME_LIGHT => Style::new().fg(latte::GREEN),
        _ => Style::new().fg(mocha::GREEN),
    }
}

pub fn red() -> Style {
    match mode() {
        THEME_PLAIN => Style::new(),
        THEME_BASIC => Style::new().fg(basic::RED),
        THEME_LIGHT => Style::new().fg(latte::RED),
        _ => Style::new().fg(mocha::RED),
    }
}

pub fn yellow() -> Style {
    match mode() {
        THEME_PLAIN => Style::new(),
        THEME_BASIC => Style::new().fg(basic::YELLOW),
        THEME_LIGHT => Style::new().fg(latte::YELLOW),
        _ => Style::new().fg(mocha::YELLOW),
    }
}

pub fn sapphire() -> Style {
    match mode() {
        THEME_PLAIN => Style::new(),
        THEME_BASIC => Style::new().fg(basic::SAPPHIRE),
        THEME_LIGHT => Style::new().fg(latte::SAPPHIRE),
        _ => Style::new().fg(mocha::SAPPHIRE),
    }
}

pub fn text() -> Style {
    match mode() {
        THEME_PLAIN => Style::new(),
        THEME_BASIC => Style::new().fg(basic::TEXT),
        THEME_LIGHT => Style::new().fg(latte::TEXT),
        _ => Style::new().fg(mocha::TEXT),
    }
}

pub fn subtext0() -> Style {
    match mode() {
        THEME_PLAIN => Style::new(),
        THEME_BASIC => Style::new().fg(basic::SUBTEXT0),
        THEME_LIGHT => Style::new().fg(latte::SUBTEXT0),
        _ => Style::new().fg(mocha::SUBTEXT0),
    }
}

pub fn subtext1() -> Style {
    match mode() {
        THEME_PLAIN => Style::new(),
        THEME_BASIC => Style::new().fg(basic::SUBTEXT0),
        THEME_LIGHT => Style::new().fg(latte::SUBTEXT1),
        _ => Style::new().fg(mocha::SUBTEXT1),
    }
}

pub fn overlay0() -> Style {
    match mode() {
        THEME_PLAIN => Style::new(),
        THEME_BASIC => Style::new().fg(basic::OVERLAY0),
        THEME_LIGHT => Style::new().fg(latte::OVERLAY0),
        _ => Style::new().fg(mocha::OVERLAY0),
    }
}

#[allow(dead_code)]
pub fn surface0() -> Style {
    match mode() {
        THEME_PLAIN => Style::new(),
        THEME_BASIC => Style::new().fg(basic::OVERLAY0),
        THEME_LIGHT => Style::new().fg(latte::SURFACE0),
        _ => Style::new().fg(mocha::SURFACE0),
    }
}

#[allow(dead_code)]
pub fn surface1() -> Style {
    match mode() {
        THEME_PLAIN => Style::new(),
        THEME_BASIC => Style::new().fg(basic::OVERLAY0),
        THEME_LIGHT => Style::new().fg(latte::SURFACE1),
        _ => Style::new().fg(mocha::SURFACE1),
    }
}

// semantic aliases
#[allow(dead_code)]
pub fn error() -> Style {
    red()
}

#[allow(dead_code)]
pub fn success() -> Style {
    green()
}

#[allow(dead_code)]
pub fn warning() -> Style {
    yellow()
}

#[allow(dead_code)]
pub fn highlight() -> Style {
    mauve()
}

#[allow(dead_code)]
pub fn accent() -> Style {
    teal()
}

#[allow(dead_code)]
pub fn dim() -> Style {
    overlay0()
}

// colour helpers - return Color for use in custom Style construction
pub fn color_sapphire() -> Color {
    match mode() {
        THEME_LIGHT => latte::SAPPHIRE,
        _ => mocha::SAPPHIRE,
    }
}

pub fn color_teal() -> Color {
    match mode() {
        THEME_LIGHT => latte::TEAL,
        _ => mocha::TEAL,
    }
}

pub fn color_mauve() -> Color {
    match mode() {
        THEME_LIGHT => latte::MAUVE,
        _ => mocha::MAUVE,
    }
}

pub fn color_peach() -> Color {
    match mode() {
        THEME_LIGHT => latte::PEACH,
        _ => mocha::PEACH,
    }
}

pub fn color_yellow() -> Color {
    match mode() {
        THEME_LIGHT => latte::YELLOW,
        _ => mocha::YELLOW,
    }
}

pub fn color_text() -> Color {
    match mode() {
        THEME_LIGHT => latte::TEXT,
        _ => mocha::TEXT,
    }
}

pub fn color_overlay0() -> Color {
    match mode() {
        THEME_LIGHT => latte::OVERLAY0,
        _ => mocha::OVERLAY0,
    }
}

#[allow(dead_code)]
pub fn color_surface0() -> Color {
    match mode() {
        THEME_LIGHT => latte::SURFACE0,
        _ => mocha::SURFACE0,
    }
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
