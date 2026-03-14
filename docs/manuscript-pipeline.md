# Manuscript pipeline for margo

Design document for `margo init --manuscript`, a scaffolding command that
standardises Quarto manuscript production for causal inference projects.

## Problems to solve

### Duplicated boilerplate

Every manuscript in `epic-pubs/` reinvents the same infrastructure: YAML
headers (PDF engine, geometry, CSL path, bibliography paths, execute
options), `setup.R` (package loading, utility functions, result import),
build scripts (`render-docx.sh`, `sync-to-submission.sh`, `Makefile`), and
author metadata (name, ORCID, affiliation, corresponding flag). The trust-
science-env manuscript, for instance, carries 68 lines of YAML, a 17 KB
`setup.R`, and a 300-line submission script. These are copied and edited
from project to project.

### Scattered shared assets

Templates live in three locations with no central registry:

- `GIT/templates/bib/references.bib` (348 KB shared bibliography)
- `GIT/templates/csl/` (apa-7, camb-a, nature, pnas)
- `GIT/latex/latex-for-quarto.tex` (352 KB shared preamble)

Manuscripts reference these via fragile relative paths
(`../../../../templates/bib/references.bib`). Moving a project breaks
every path.

### Monolithic LaTeX preamble

`latex-for-quarto.tex` is 7,418 lines. Most manuscripts need core packages
(booktabs, hyperref, amsmath, graphicx, authblk) and causal-notation
commands. The remaining ~80% is bespoke TikZ diagram infrastructure that
most papers never use. Loading the full file adds compile time and
namespace pollution.

### Missing corresponding-author handling

The current `title.tex` partial renders author names, ORCID links, and
affiliations, but has no logic for the `corresponding: true` flag in
Quarto's author metadata. Corresponding-author footnotes must be added
manually.

### Main + SI reference splitting

When main text and supplementary materials live in one QMD, Pandoc produces
a single merged bibliography. Splitting into separate reference lists
requires post-hoc TeX surgery (`sync-to-submission.sh` uses awk to extract
the `CSLReferences` environment and append it to both halves). A two-QMD
pattern with separate `::: {#refs}` divs avoids this entirely.

### No measures integration

Margo's `/measure` workspace manages variable metadata (descriptions,
scales, items, references) but this metadata is not wired into manuscript
production. Supplementary variable-description tables are still built by
hand.

## Proposed: what `margo init --manuscript` generates

### Project structure

```
my-manuscript/
├── study.toml              # extended config (see below)
├── manuscript.qmd          # main text, ends with ::: {#refs}
├── supplement.qmd          # SI, has its own ::: {#refs}
├── setup.R                 # loads saved results from analysis project
├── title.tex               # Quarto template partial (with corresponding author)
├── render.sh               # renders both QMDs; optional pdfunite --combine
├── sync-to-submission.sh   # journal packaging (blind review, figure renaming)
├── Makefile                # delegates to scripts
├── .gitignore
└── README.md
```

Two QMDs, each with its own `::: {#refs}` div, means Pandoc produces
separate bibliographies natively. No TeX splitting needed for references.

### study.toml: manuscript section

Extend the existing `study.toml` format with a `[manuscript]` table. This
keeps analysis config and manuscript config in one place.

```toml
[manuscript]
title = "Trust in Scientists Shapes Climate Attitudes"
date = "last-modified"

# short keys referencing the author registry (see below)
authors = ["ik", "cs", "ky", "jb"]
corresponding = "jb"

# bibliography
csl = "camb-a"                       # resolved from registry or local path
bibliography = ["references.bib"]    # project-specific; shared bib appended automatically

# latex
latex_modules = ["core", "causal-notation"]   # opt-in from modular headers
pdf_engine = "lualatex"

# analysis link
results_path = "../saved-results/"   # where setup.R loads .qs files from
```

Fields not specified fall back to defaults in `~/.config/margo/config.toml`.

### Author registry

A user-level TOML file at `~/.config/margo/authors.toml` stores author
metadata once. Referenced by short key in `study.toml`.

```toml
[jb]
name = "Joseph A. Bulbulia"
affiliation = "Victoria University of Wellington, New Zealand"
orcid = "0000-0002-5861-2056"
email = "joseph.bulbulia@vuw.ac.nz"

[cs]
name = "Chris G. Sibley"
affiliation = "School of Psychology, University of Auckland, New Zealand"
orcid = "0000-0002-4064-8800"

[ky]
name = "Kumar Yogeeswaran"
affiliation = "University of Canterbury, New Zealand"
orcid = "0000-0002-1978-5077"

[ik]
name = "Inkuk E. Kim"
affiliation = "Victoria University of Wellington, New Zealand"
orcid = "0000-0003-3169-6576"
```

At scaffold time, `margo init --manuscript` expands these into the QMD's
YAML `author:` block. Adding `corresponding: true` to the author whose
key matches `manuscript.corresponding`.

### Bibliography and CSL

Default paths configured in `~/.config/margo/config.toml`:

```toml
[bibliography]
shared = "/Users/joseph/GIT/templates/bib/references.bib"

[csl]
dir = "/Users/joseph/GIT/templates/csl/"
default = "camb-a"
```

The generated QMD YAML uses absolute paths (no fragile relative paths).
A local `references.bib` is created for project-specific entries. The
`bibliography:` field lists the local file first, shared file second.

Short CSL names (`camb-a`, `nature`, `apa-7`) resolve against `csl.dir`.

### LaTeX headers: modular split

Split the monolith into composable modules stored at
`~/.config/margo/latex/`:

| Module | Contents | Size (est.) |
|---|---|---|
| `core.tex` | booktabs, hyperref, amsmath, graphicx, authblk, orcidlink, xcolor, geometry tweaks, caption formatting | ~200 lines |
| `causal-notation.tex` | counterfactual notation (`\Wzero`, `\A{}`, `\Ltv{}`, `\Hist{}`), policy commands, colour definitions | ~300 lines |
| `diagrams.tex` | TikZ libraries, SWIG macros, circle definitions, causal-diagram infrastructure | ~6,500 lines |

Manuscripts include `core.tex` by default. Others opt-in via
`manuscript.latex_modules` in `study.toml`. The generated YAML header
contains `\input{}` lines only for selected modules.

Extraction plan:
1. Identify the boundary between core packages and causal notation in
   `latex-for-quarto.tex` (roughly lines 1-200 vs 200-500 vs 500+)
2. Extract into three files, verify each compiles independently
3. Keep `latex-for-quarto.tex` as a backwards-compatible wrapper that
   `\input`s all three (existing manuscripts continue to work)

### title.tex: corresponding-author handling

Extend the current template partial to detect `by-author.attributes.corresponding`
and render a correspondence footnote:

```latex
$for(by-author)$
\author{$by-author.name.literal$$if(by-author.orcid)$~\textcolor[HTML]{A6CE39}{\aiOrcid}\hspace{0.1em}\href{https://orcid.org/$by-author.orcid$}{$by-author.orcid$}$endif$$if(by-author.attributes.corresponding)$\thanks{Corresponding author: $by-author.email$}$endif$}
  $if(by-author.affiliations)$
    $for(by-author.affiliations)$
      \affil{$if(by-author.affiliations.name)$\small{$by-author.affiliations.name$}$endif$}
    $endfor$
  $endif$
$endfor$
```

The `\thanks{}` command places the email as a title footnote, standard for
journal submissions.

### setup.R generation

The existing GRF templates generate eight numbered R scripts. The
manuscript variant generates a single `setup.R` that:

1. Loads packages (margot, ggplot2, kableExtra, tinytable, qs)
2. Reads `study.toml` for `results_path`
3. Loads saved `.qs` result objects
4. Defines formatting helpers (`pretty_int()`, `format_effect()`,
   `style_manuscript_table()`)
5. Builds table and plot objects ready for inline reference in QMD

This parallels what the trust-science-env `setup.R` does (17 KB), but
generates from config rather than being hand-written.

### Measures integration

Margo's `/measure` workspace already stores variable metadata with fields
matching what SI tables need (description, scale, reference, items). A
future command could export this directly:

```
/measure export-si --format qmd > supplement-measures.qmd
```

This is not part of the initial scaffold but the two-QMD structure
accommodates it: `supplement.qmd` can `{{< include supplement-measures.qmd >}}`
when ready.

### render.sh

```bash
#!/bin/bash
set -euo pipefail
quarto render manuscript.qmd --to pdf
quarto render supplement.qmd --to pdf
if command -v pdfunite &>/dev/null; then
  pdfunite manuscript.pdf supplement.pdf combined.pdf
fi
```

### sync-to-submission.sh

Template based on the trust-science-env version, parameterised for the
project. Handles:
- Blind review (strip author names/affiliations from TeX)
- Figure renaming (fig1.pdf, figS1.pdf)
- Zip packaging

With two-QMD architecture, no TeX splitting is needed; each QMD already
compiles to a self-contained PDF with its own references.

## Scope and boundaries

This is a **separate project type** from the existing `grf` and
`grf-event` templates. It extends margo's template system
(`src/templates/`) without modifying existing scaffolding.

Implementation order:

1. Author registry (`authors.toml` format and reader)
2. `study.toml` manuscript section (schema extension)
3. LaTeX module extraction (one-time refactor of `latex-for-quarto.tex`)
4. `title.tex` corresponding-author fix
5. `margo init --manuscript` scaffolding (QMDs, setup.R, scripts)
6. Measures-to-SI export (later phase)

## Relation to existing infrastructure

| Component | Current location | After |
|---|---|---|
| Shared bib | `GIT/templates/bib/references.bib` | Same, referenced via `config.toml` |
| CSL files | `GIT/templates/csl/` | Same, referenced via `config.toml` |
| LaTeX preamble | `GIT/latex/latex-for-quarto.tex` | Split into `~/.config/margo/latex/{core,causal-notation,diagrams}.tex`; monolith kept as wrapper |
| Author metadata | Duplicated in each QMD | `~/.config/margo/authors.toml` |
| Build scripts | Hand-written per project | Generated from templates |
| Variable descriptions | Manual SI tables | Future: `/measure export-si` |
