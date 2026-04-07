# Changelog

Notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.15] - 2026-04-05

### Changed
- replaced renv with rv for reproducible environment management in generated projects
- generated projects now include `rproject.toml` for rv dependency tracking
- `use_renv` config option renamed to `use_rv` (still accepts `use_renv` for backward compatibility)

## [0.3.14] - 2026-03-03

### Added
- New measures workspace in the REPL via `/measure` commands for load, source, list, show, add, edit, rename, delete, save, diff, validate, and export-missing
- Measures data adapters and IO utilities for `boilerplate_unified.json`, `measures_db.json`, and variable metadata TSV/CSV sources
- Measures documentation: quickstart and command reference under `docs/measures/`

### Changed
c- README now links to measures workspace documentation
- Measures workspace now supports deterministic serialisation and source-aware format handling

### Fixed
- Removed unused measures-module scaffolding and re-exports to eliminate compile-time warnings in normal builds

## [0.3.13] - 2026-03-02

### Fixed
- Remove unused `time` dependency to eliminate `RUSTSEC-2026-0009` / `CVE-2026-25727` exposure

## [0.3.12] - 2026-03-02

### Added
- `/vars` metadata provider fallbacks in priority order: local files, boilerplate sources, then bptui sources
- JSON metadata parsing for measure databases (including `boilerplate_unified.json` and `measures_db.json`)
- Provider-specific environment overrides: `MARGO_BOILERPLATE_METADATA` and `MARGO_BPTUI_METADATA`

### Changed
- `/vars` metadata source output now includes provider context (`local`, `boilerplate`, `bptui`)
- `/vars` missing-metadata hint now documents all supported metadata environment variables

### Fixed
- Metadata source borrow handling in `/vars` output to avoid partial move compile errors
- `/vars` picker flow now supports Enter-to-add with repeated selection, then Esc to finish

## [0.3.11] - 2026-01-16

### Fixed
- Ensure renv settings are passed through GRF event scaffold generation

## [0.3.10] - 2026-01-16

### Added
- `/init` slash command alias for guided project setup
- `/e` picker includes `/config` for quick config editing
- Alert when exposure is present in outcomes and remove it

### Changed
- Tips copy now emphasises reviewing default paths and editing templates or config, and shows `/q` on startup
- `/config` output now shows edit hints for `/config edit` and `/e config`
- Project scaffolds now use renv by default (`use_renv`, with `use_rv` accepted for compatibility)
- GRF event study scaffolds now include `src/00-setup.R` for renv setup

### Fixed
- Remove exposure variables from outcome lists during REPL init and warn the user

## [0.3.9] - 2026-01-12

### Fixed
- Baseline custom selection no longer reuses the outcome picker in the REPL

## [0.3.8] - 2026-01-11

### Added
- `/config output` to set the output root (`push_mods`) from the REPL
- `/config data` to set the default data directory (`pull_data`)
- `/config setup` to set default paths in one flow
- `/config reset` to restore config defaults with confirmation
- First-run prompt in the REPL to configure default paths
- `margo config reset` CLI command

### Changed
- `margo config init` now offers an interactive default-path setup when run in a TTY
- Generated R scripts now live under `src/` instead of the project root

### Fixed
- Output paths now use OS-aware joins and expand `~` in config paths

## [0.3.7] - 2026-01-11

### Added
- Version line under the welcome tagline
- Template review actions to view, edit, and save as a new template during init
- Esc binding to return home in the REPL

### Changed
- Project review now includes exposure between baseline and outcomes
- Init summary output path matches the actual project name
- `/templates edit` opens a picker when no name is provided

### Fixed
- Empty `push_mods` config falls back to `./outputs` instead of writing to `/`
- Edit-template confirmation no longer triggers project creation

## [0.3.6] - 2026-01-11

### Fixed
1. Restored shipped baseline and outcome templates on startup when the user copy is missing or still unmodified.

## [0.3.5] - 2025-12-27

### Changed
- GRF templates now default to rv setup (`00-setup.R`, README instructions, and `.gitignore`)
- Config option renamed to `use_rv` (still accepts `use_renv`)

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
- `00-setup.R` script with rv init/sync for reproducible R environments
- `use_rv` config option (defaults to true) to control rv integration

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




