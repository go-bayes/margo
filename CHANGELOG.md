# Changelog

Notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2025-12-09

### Added
- Interactive REPL mode (run `margo` with no arguments)
  - Vi mode with visual indicator (pink ● normal, teal ❯ insert)
  - Persistent command history (`~/.config/margo/history`)
  - Catppuccin Mocha colour palette with pink branding
- Slash commands: `/help`, `/config`, `/templates`, `/vars`, `/clear`
- Fuzzy variable picker for init commands
  - 530 bundled NZAVS variable names with fuzzy search
  - Guided flow: model → baseline → exposure → outcomes
- Tab completion for commands, variables, and templates
- Syntax highlighting in REPL input
- Configurable theme support (`[theme]` section in config.toml)
  - `catppuccin` (default) — full RGB colour palette
  - `basic` — 16-colour ANSI fallback for limited terminals
  - `plain` — no colours

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
