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
| `grf` | Generalised Random Forests (heterogeneous treatment effects) | ✓ Available |
| `lmtp` | Longitudinal Modified Treatment Policies | Planned |

## Licence

MIT
