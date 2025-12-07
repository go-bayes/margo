# margo

A CLI tool for scaffolding [margot](https://github.com/go-bayes/margot) causal inference projects.

## Installation

### Install via cargo

If you have Rust installed:

```bash
cargo install --git https://github.com/go-bayes/margo
```

### Installing Rust

If you don't have Rust, install it first:

```bash
# macOS / Linux
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# then restart your terminal, or run:
source ~/.cargo/env
```

For Windows, download the installer from [rustup.rs](https://rustup.rs).

Once Rust is installed, run the `cargo install` command above.

## Usage

```bash
# create a new GRF (Generalised Random Forests) project
margo init grf my-study

# show help
margo --help
```

## What it does

`margo init grf <project-name>` creates a complete project scaffold:

```
my-study/
├── study.toml          # configuration (edit this first)
├── README.md           # workflow documentation
├── .gitignore          # excludes data files
├── 01-data-prep.R      # data wrangling, binary exposure
├── 02-wide-format.R    # long→wide, two-stage IPCW weights
├── 03-causal-forest.R  # GRF estimation, ATE plots
├── 04-heterogeneity.R  # RATE/AUTOC tests, qini plots
├── 05-policy-tree.R    # policy tree stability
├── 06-positivity.R     # transition tables
├── 07-tables.R         # summary tables
└── 08-plots.R          # visualisation
```

### Configuration

Edit `study.toml` with your study-specific settings:

```toml
[paths]
pull_data = "/path/to/your/source/data"
push_mods = "/path/to/your/output/directory"

[waves]
baseline = "Time 11"
exposure = ["Time 12"]
outcome  = "Time 13"

[exposure]
name = "your_exposure_variable"
binary_cutpoints = [0, 5]

[outcomes]
vars = ["outcome_1", "outcome_2"]

[baseline]
vars = ["age", "male_binary", "education_level_coarsen"]
```

Then run scripts in order: `01-data-prep.R`, `02-wide-format.R`, etc.

## Requirements

- R ≥ 4.0
- margot package: `devtools::install_github("go-bayes/margot")`

## Templates

| Template | Description | Status |
|----------|-------------|--------|
| `grf` | Generalised Random Forests (3-wave heterogeneous treatment effects) | ✓ Available |
| `grf-event` | GRF Event Study (multi-outcome waves for effect trajectories) | ✓ Available |
| `lmtp` | Longitudinal Modified Treatment Policies | Planned |

## CLI Examples

### Basic usage

```bash
# create a GRF project with exposure and outcomes specified directly
margo init grf church_attendance wellbeing life_satisfaction

# multiple outcomes
margo init grf hours_exercise kessler_6 self_esteem meaning_purpose
```

### Using templates

Templates let you reuse predefined sets of outcomes and baselines:

```bash
# load outcomes from a template
margo init grf church_attendance -t wellbeing

# combine multiple outcome templates
margo init grf exercise -t wellbeing,health

# use a custom baseline template
margo init grf meditation life_satisfaction -b extended
```

Templates are stored in `~/.config/margo/outcomes/` and `~/.config/margo/baselines/`.

### Custom project names

```bash
# auto-generated name: church_attendance-wellbeing-life_satisfaction
margo init grf church_attendance wellbeing life_satisfaction

# custom name
margo init grf church_attendance wellbeing -n "nzavs-religion-study"
```

### WHO mode (BMI/exercise variables)

```bash
# default: continuous (hlth_bmi, log_hours_exercise)
margo init grf sleep quality -w default

# categorical (bmi_cat, who_hours_exercise_cat)
margo init grf diet weight -w cat

# ordinal numeric (bmi_cat_num, who_hours_exercise_num)
margo init grf stress anxiety -w num
```

### Configuration management

```bash
# create user config file
margo config init

# show config path
margo config path
# ~/.config/margo/config.toml

# edit config in your $EDITOR
margo config edit
```

### GRF Event Study (multi-outcome waves)

For longitudinal event studies where a single exposure is followed by multiple outcome waves:

```bash
# basic event study
margo init grf-event earthquake_affected -o religion_religious

# specify outcome waves and reference wave (t=0)
margo init grf-event earthquake_affected \
  -o religion_religious \
  -w 2011,2012,2013,2014,2015,2016,2017,2018,2019,2020,2021,2022,2023 \
  -r 2011 \
  -n chch-earthquake-faith

# use custom baseline template
margo init grf-event flood_exposure -o mental_health -b extended
```

This generates scripts that:
1. Fit causal forests for each outcome wave
2. Collect ATEs across waves
3. Plot effect trajectory over time
4. Run heterogeneity tests on significant waves

### Interactive mode

```bash
# launch the terminal UI for guided project setup
margo new
```

### Getting help

```bash
margo --help
margo init --help
margo init grf --help
margo init grf-event --help
margo config --help
```

## Licence

MIT
