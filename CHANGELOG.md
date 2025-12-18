# Changelog

Notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.4] - 2025-12-14

### Added
- Hash-aware template manifest to track shipped defaults and user state
- `margo templates refresh` command with `--force`, `--sidecar`, and `--dry-run`
- Auto-initialize bundled templates on first run if none exist

### Fixed
- Project name year now uses real calendar time (no early rollover)

## [0.3.3] - 2025-12-13

### Added
- `margo templates` command with subcommands: `list`, `examples`, `copy`, `init`
- Bundled example templates in `baselines/examples/` and `outcomes/examples/`
- `00-setup.R` script with renv::init() for reproducible R environments
- `use_renv` config option (defaults to true) to control renv integration

### Changed
- Removed `who_mode` option (users should define variables in their own templates)
- Templates served as defaults in `examples/` subdirectory, never overwriting user templates

### Refactored
- Extracted `format_var_array()` to shared `templates/mod.rs`
- Extracted editor spawning to `commands/utils.rs`

## [0.3.2] - 2025-12-09

### Changed
- `/t` shortcut now maps to `/templates` (was `/theme`)
- `/th` shortcut added for `/theme`
- Init commands always use guided menu (no CLI args required)
- Updated hints to show guided flow descriptions
- Increased fuzzy match results from 10 to 50

### Added
- Project summary now shows "scripts:" location
- Warning before overwriting existing project files (study.toml, R scripts)

### Fixed
- Tab completion hints now reflect new command structure

## [0.3.1] - 2025-12-09

### Added
- `/theme` command for light/dark theme toggle
  - Catppuccin Latte (light) and Mocha (dark) palettes
  - `/theme toggle`, `/theme light`, `/theme dark`, `/theme show`
- `/view` command to browse templates with variable preview
- `/save <type> <name>` command to create templates on-the-fly
- Navigation commands: `/home`, `/cd`, `/here`, `/refresh` (`/r`)
- `/e` and `/o` quick edit aliases for templates
- `/` command picker (fuzzy find commands)
- Baseline selection options: template, modify (edit vars), custom
- Outcome selection: choice between templates or individual variables
- Confirmation step showing selected variables before proceeding

### Changed
- Init flow validates exposure variable before showing other pickers
- Escape at any picker cancels entire init flow
- Project summary shown before creating files with y/n confirmation
- Welcome screen shows config (data, output, baselines, cwd) + tips
- Prompt hints now show `/help  /home  /q`
- Clearer help messages: `↑↓ move, Space toggle, type to filter, Enter done`

### Removed
- `/clear` command (use `/r` or `/refresh` instead)

### Fixed
- Tab autocomplete now displays completion menu
- `/vars` now scrollable through all 530 variables
- Vim mode disabled in variable pickers so j/k can filter

## [0.3.0] - 2025-12-09

### Added
- Interactive REPL mode (run `margo` with no arguments)
  - Vi mode with visual indicator (pink ● normal, teal ❯ insert)
  - Right-prompt hints: `NORMAL • i insert` or `/help • :q quit`
  - Vim-style quit commands (`:q`, `:q!`, `:wq`)
  - Persistent command history (`~/.config/margo/history`)
  - Catppuccin Mocha colour palette with pink branding
- Slash commands: `/help`, `/config`, `/templates`, `/vars`, `/clear`
- Fuzzy variable picker for init commands
  - 530 bundled NZAVS variable names with fuzzy search
  - Guided flow: model → baseline → exposure → outcomes
  - Subtle background highlight on selected row
- Tab completion for commands, variables, and templates
- Syntax highlighting in REPL input
- Configurable theme support (`[theme]` section in config.toml)
  - `catppuccin` (default) — full RGB colour palette
  - `basic` — 16-colour ANSI fallback for limited terminals
  - `plain` — no colours
- Interactive template editor (`/templates edit <name>`)
  - Toggle variables on/off with fuzzy search
  - Pre-selects existing template variables (e.g., `wellbeing` shows `kessler_latent_depression`, `life_satisfaction`, etc.)
- `/templates open <name>` to edit raw TOML in $EDITOR

### Changed
- Default entry point is now REPL (previously required subcommand)
- `[editor]` section in config.toml for configurable editor

### Removed
- TUI mode (`margo new`) - replaced by REPL
- `ratatui` and `colored` dependencies

### Notes
- TUI code preserved in `storage/tui/` for reference

## [0.2.1] - 2025-12-07

### Added
- `margo init grf-event` command for longitudinal event studies
  - Multi-outcome wave design (e.g., earthquake effects over 12 years)
  - Configurable wave column (`time_factor` or `wave`)
  - 7 R scripts: data-prep, wide-format, causal-forest, trajectory-plot, heterogeneity, positivity, tables
  - ATE trajectory visualisation with confidence intervals and sample size panel

## [0.2.0] - 2025-12-07

### Added
- Template-based configuration system (`~/.config/margo/`)
  - `config.toml` for paths (`pull_data`, `push_mods`) and defaults
  - `baselines/` directory for baseline variable templates
  - `outcomes/` directory for outcome variable templates
- New CLI syntax: `margo init grf <exposure> [outcomes...] [-t templates]`
  - Direct outcome variables as positional args
  - `-t` flag for loading outcomes from templates
  - `-n` flag for custom project names
- `margo config` command to manage configuration
  - `margo config` / `margo config init` - create config file
  - `margo config path` - show config path
  - `margo config edit` - open in $EDITOR
- TUI scaffolding with ratatui (not yet functional)

### Changed
- Projects now created in current directory (scripts are git-friendly)
- Output folder created at `{push_mods}/{project-name}/`
- Config location changed to `~/.config/margo/` (XDG style)

### Note
- TUI (`margo new`) is scaffolded but not yet connected to new config system

## [0.1.1] - 2025-12-07

### Added
- Standard NZAVS baseline variables (39 vars: demographics, Big Six personality, health, social)
- `who_mode` field in `[baseline]` for BMI/exercise variable selection (default/cat/num)
- `[confounders]` section with time-varying confounders and `include_outcomes` option
- Integration test suite (8 tests covering project creation, TOML validity, variable sets)

### Changed
- Wave defaults now Time 10, 11, 12 (was Time 11, 12, 13)
- Ordinal vars: education, eth, rural (religion_identification_level is not ordinal)

## [0.1.0] - 2025-12-07

### Added
- Initial release
- `margo init grf <name>` command for GRF project scaffolding
- `study.toml` configuration template
- 8 R scripts (01-data-prep through 08-plots)
- README and .gitignore generation
