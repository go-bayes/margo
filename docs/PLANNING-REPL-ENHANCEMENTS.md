# Margo REPL Enhancements - Planning Document

## Overview

Incremental improvements to the existing REPL, keeping the `reedline` + `inquire` architecture.

---

## Completed

### Bug 1: Tab Autocomplete Not Working

**Status**: FIXED
**Solution**: Added `ColumnarMenu` to reedline and configured Tab keybinding in Vi insert mode to trigger completion menu.

**Files changed**:
- `src/repl/mod.rs` - added menu and keybindings

---

### Bug 2: `/vars` Cannot Scroll Past Window

**Status**: FIXED
**Solution**: Changed `/vars` from static print to interactive `inquire::Select` picker with `page_size(20)` and vim mode enabled.

**Files changed**:
- `src/repl/commands.rs` - rewrote cmd_vars
- `src/repl/picker.rs` - added browse_variables function

---

### Feature 1: Theme Toggle (Light/Dark Catppuccin)

**Status**: IMPLEMENTED
**Solution**: Added Catppuccin Latte (light) palette alongside Mocha (dark), with `/theme` command to toggle.

**New commands**:
- `/theme` or `/theme toggle` - switch between light/dark
- `/theme light` or `/theme latte` - set light theme
- `/theme dark` or `/theme mocha` - set dark theme
- `/theme show` - display current theme

**Files changed**:
- `src/theme.rs` - added latte palette, toggle functions, colour helpers
- `src/repl/commands.rs` - added cmd_theme
- `src/repl/completer.rs` - added theme completions
- `src/repl/highlighter.rs` - use theme-aware colours
- `src/repl/hinter.rs` - use theme-aware colours
- `src/repl/prompt.rs` - use theme-aware colours, fixed lifetime annotations

---

### Feature 2: Verify `:q` from NORMAL Mode

**Status**: VERIFIED
**Existing implementation**: `:q`, `:q!`, `:wq` all handled in `parse_input()` in `src/repl/mod.rs`.

---

### Feature 3: `/view` Command for Templates

**Status**: IMPLEMENTED
**Solution**: Interactive template browser with variable preview.

**Usage**:
- `/view` - opens picker showing all templates (outcomes and baselines)
- `/view <name>` - shows variables in specific template

**Files changed**:
- `src/repl/commands.rs` - added cmd_view, view_template, view_template_picker
- `src/repl/picker.rs` - added browse_templates function
- `src/repl/completer.rs` - added /view completion

---

### Feature 4: `/save` Command for Templates

**Status**: IMPLEMENTED
**Solution**: Create templates on-the-fly from variable selection.

**Usage**:
- `/save outcomes <name>` - create new outcomes template
- `/save baselines <name>` - create new baselines template

Opens interactive variable picker, then saves selection to `~/.config/margo/<type>/<name>.toml`.

**Validation**:
- Template type must be 'outcomes' or 'baselines'
- Name must be alphanumeric + underscores only
- Won't overwrite existing templates

**Files changed**:
- `src/repl/commands.rs` - added cmd_save, print_save_usage, format_template_toml
- `src/repl/picker.rs` - added pick_outcomes_for_save function
- `src/repl/completer.rs` - added /save completions

---

## Code Quality

All warnings resolved:
- Removed unused imports
- Added `#[allow(dead_code)]` for intentionally complete colour palettes
- Fixed lifetime annotations in prompt.rs
- Clean build with zero warnings

---

## Testing Checklist

- [x] Tab completes `/h` to `/help`
- [x] Tab completes template names after `-t`
- [x] `/vars` scrolls through all 530 variables
- [x] `/vars` j/k and arrow navigation works
- [x] `/theme` toggles between light and dark
- [x] `:q` exits from normal mode
- [x] `/view` shows template picker
- [x] `/view <name>` shows template contents
- [x] `/save outcomes foo` creates template
- [x] Saved template appears in `/templates`

---

## Architecture Notes

### Theme System

The theme system uses atomic state (`AtomicU8`) for thread-safe mode switching:

```rust
// src/theme.rs
static THEME_MODE: AtomicU8 = AtomicU8::new(THEME_DARK);

pub fn toggle_theme() { ... }
pub fn current_theme() -> &'static str { ... }
```

Style helpers are theme-aware:
```rust
pub fn pink() -> Style {
    match mode() {
        THEME_LIGHT => Style::new().fg(latte::PINK),
        _ => Style::new().fg(mocha::PINK),
    }
}
```

Colour helpers return `Color` for custom style construction:
```rust
pub fn color_sapphire() -> Color { ... }
```

### Picker Pattern

All interactive pickers follow this pattern:
```rust
pub fn browse_X(prompt: &str, items: &[T]) -> Result<Option<R>> {
    let result = Select::new(prompt, items)
        .with_vim_mode(true)
        .with_page_size(N)
        .with_help_message("...")
        .with_render_config(catppuccin_config())
        .prompt_skippable()?;

    Ok(result.map(|s| ...))
}
```

### Command Dispatch

Commands are dispatched in `handle_slash()`:
```rust
match command {
    "help" | "h" | "?" => cmd_help(),
    "config" => cmd_config(args),
    // ...
}
```

Each command function follows the signature:
```rust
fn cmd_X(args: &[&str]) -> Result<()>
```
