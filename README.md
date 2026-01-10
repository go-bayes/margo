# agreeableness-wellbeing

GRF (Generalised Random Forests) causal inference workflow.

## Getting started

1. Install rv: https://github.com/A2-ai/rv
2. Open R in this project directory
3. Run `source("00-setup.R")` to initialise rv and install dependencies
4. Edit `study.toml` with your study-specific settings
5. Run scripts in order: 01, 02, 03...

## Script order

| Script | Purpose |
|--------|---------|
| 00-setup.R | project setup (rv, dependencies) |
| 01-data-prep.R | data prep, saves `dat_long_final`, weights |
| 02-wide-format.R | wide data + two-stage IPCW weights, saves `df_grf` |
| 03-causal-forest.R | causal forest estimation + ATE plot + diagnostics |
| 04-heterogeneity.R | heterogeneity tests + qini plots |
| 05-policy-tree.R | policy tree stability + policy workflow |
| 06-positivity.R | positivity transition tables |
| 07-tables.R | baseline/exposure/outcome tables |
| 08-plots.R | timeline + individual plots |

## Configuration

Edit `study.toml` with your study-specific settings before running scripts.

## Requirements

- R >= 4.0
- rv package manager: https://github.com/A2-ai/rv
- margot package (installed via rv in `00-setup.R`)
