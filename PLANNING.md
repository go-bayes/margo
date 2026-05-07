## Current orientation

As of 2026-05-07, `margo` is still the scaffold generator and interactive
helper for `margot`-based workflows. The immediate design pressure comes
from private `epic-models` audits, especially multi-wave LMTP and three-wave
GRF workflows. Public `margo` notes should describe general workflow
decisions and avoid private study names, local filesystem paths, or
person-specific shorthand.

Current GRF scaffold state:

- `margo init grf` now generates a shared `src/00-preflight.R` layer.
- Generated GRF scripts should fail with clear missing-package messages.
- Generated GRF scripts should not use `qs`, `here_read_qs()`,
  `here_save_qs()`, or `pacman::p_load()`.
- Source data import is Arrow-based through `margot::here_read_arrow()`,
  controlled by `paths.source_arrow_name` in `study.toml`.
- Saved analysis objects use explicit `margot::here_save()` /
  `margot::here_read()` calls with `paths.push_mods`.
- The GRF event-study scaffold has not yet received the same cleanup and
  still needs review for `qs`, `pacman`, output contracts, and audit outputs.

Workflow defaults to preserve:

- Generate sourceable R chunks with clear contracts between scripts.
- Create labels and reusable metadata once, early in the workflow.
- Prefer one canonical analysis data frame over hand-built domain-specific
  data frames.
- Generate human-readable audit outputs, especially missingness structure and
  cut-point checks before estimation.
- For GRF manuscripts, make average treatment effect and policy-tree reporting
  the main path; keep RATE/Qini results in appendices unless explicitly
  requested.
- For future publication scaffolds, generate a minimal `setup.R`,
  `manuscript.qmd`, and `supplement.qmd` that read saved model/result objects
  and regenerate plots rather than saving plot artefacts as workflow state.

Immediate next implementation questions:

1. Bring `grf-event` in line with the cleaned GRF scaffold.
2. Decide whether publication scaffolding is a separate command or an option
   attached to `margo init grf`.
3. Add a general study-validation command before investing in a richer TUI.
4. Keep LMTP schema work in private workflow notes until the multi-wave
   realisation is stable enough to make public.

## Short term questions
Checklist:

- add policy tree decision points?  -- outcomes to reverse? fairness exclusions for policy trees? 
- subgroup analyses?  
- data checks in separate cli (for lab/ specific to data we use? or general?)


1. should we let users compile projects once settings are fixed, and accept harder debugging to reduce coding? (I think not)
2. Allow users to choose cut points for continuous vars when creating binary versions. (YES, this is in the script: we might need to reveal histogram for this to make sense? -- suggest it go into the validation cli)
3. Plan extensibility for time-varying confounders in the framework (as part of TMLE/lmtp)
4. Enable confounders from the same wave as exposure when exposure cannot affect them. (lmtp)
5. Add a helper that warns when `paths.pull_data` points at a file instead of a directory.
6. Add `/vars export-missing` to print variables with no metadata description.
7. Add richer `/vars` metadata support (`label`, `scale`, `notes`) beyond a single description field.

## TUI planning (currently doubtful this is usesful, necesaary)
1. Keep margo stable and plan a separate margot TUI track using the latest ratatui refactor.
(consider a TUI that organizes variables left to right as confounders, exposures, time-varying confounders, time-varying outcomes, and end-of-study outcomes)
2. Use a pipeline layout with baseline, exposure, and outcomes tiles moving right to left, maybe...
3. For lmtp, show multiple exposure tiles with time-varying confounders on a timeline, although we can do this in a validation toml...
4. Use tachyonfx for motion and draw inspiration from tek for crisp borders and typography... pretty and all, but a priority? 
5. Decide how much animation aids understanding without distracting from selection tasks....prob place anything like this at end -- carrot to stick
6. Decide between fixed columns or flowing tiles to keep focus and keyboard navigation clear.
7. Decide input style: list, search, or hybrid, and show defaults and overrides.
8. Decide how selections write templates while preserving user edits and config preferences.

## Measures workbench plan (margo-first; supersedes standalone bptui)
1. Treat `margo` as the single interactive editor for measures metadata used by boilerplate report workflows, and avoid new runtime coupling to bptui internals.
2. Define one canonical interchange schema for measures records (`name`, `description`, `reference`, `waves`, `keywords`, `items`, `standardised`, `standardised_date`, `label`, `scale`, `notes`), with strict field normalisation and stable ordering.
3. Implement robust import adapters in `margo` for `boilerplate_unified.json`, `measures_db.json`, `measures_db.csv`, and `variable_metadata.tsv/csv`, preserving unknown fields where possible.
4. Add write-safe export paths in `margo` with transactional save semantics (temp file + atomic replace), backup checkpoints, and deterministic formatting to reduce noisy diffs in git.
5. Add a `measure` command group in `margo` REPL/CLI for list, find, edit, add, delete, validate, standardise, and bulk operations targeting boilerplate-compatible files.
6. Add quality and completion utilities in `margo`: missing description report, missing core fields report, duplicate name detection, and normalisation warnings before save.
7. Add schema-aware transforms for R workflows: label mapping generation, scale extraction, and notes/description harmonisation aligned with boilerplate expectations.
8. Keep metadata lookup precedence in `/vars` as local project files first, then boilerplate, then bptui-compatible sources, with explicit source reporting for traceability.
9. Migration track: provide a one-time import path from existing bptui JSON files into the canonical margo schema and document bptui deprecation for this workflow.
10. Documentation and rollout: publish `margo` measure-workbench usage docs and examples showing end-to-end edit -> save -> boilerplate report generation.

### `/measure` command design (REPL + CLI parity)
1. `/measure load [path]` loads a measures source file (`boilerplate_unified.json`, `measures_db.json`, `measures_db.csv`, `variable_metadata.tsv/csv`); if omitted, use auto-discovery.
2. `/measure source` prints active source file, format, record count, and dirty state.
3. `/measure list [pattern]` lists measures with fuzzy filter and key fields (`name`, short description, scale/notes indicators).
4. `/measure show <name>` prints full canonical record including passthrough fields.
5. `/measure find <query>` jumps/selects next matching record for iterative editing.
6. `/measure add <name>` creates a new canonical record scaffold.
7. `/measure edit <name> <field> <value>` updates one field with schema-aware coercion and validation.
8. `/measure edit-batch <field> <from> <to>` applies controlled replacements with preview.
9. `/measure delete <name>` removes one record with confirmation.
10. `/measure rename <old> <new>` renames a measure key while preserving content.
11. `/measure validate` runs completeness and schema checks (missing description, duplicate names, malformed fields).
12. `/measure standardise` applies normalisation rules (trim, case policy, stable field order, scale extraction helpers).
13. `/measure export-missing [field]` prints measures missing required fields (default `description`).
14. `/measure backup [label]` writes timestamped checkpoint before risky operations.
15. `/measure save [path]` writes deterministic output using atomic replace and backup policy.
16. `/measure diff` shows in-session changes since load to support review before save.
17. `/measure import-bptui <path>` migrates legacy bptui JSON into canonical schema with migration report.
18. `/measure help` prints command reference and workflow tips.

### Staged implementation checklist
1. Phase 1: schema and storage core.
2. Implement canonical `MeasureRecord` model and parser/writer adapters.
3. Add deterministic serialisation, passthrough-field retention, and atomic save utilities.
4. Add in-memory session state (`loaded`, `dirty`, `source`, `history`).
5. Phase 2: read and inspect workflow.
6. Implement `/measure load`, `/measure source`, `/measure list`, `/measure show`, `/measure find`.
7. Implement `/measure validate` and `/measure export-missing` with human-readable summaries.
8. Phase 3: edit workflow.
9. Implement `/measure add`, `/measure edit`, `/measure delete`, `/measure rename`.
10. Add `/measure diff`, `/measure backup`, and `/measure save` with rollback safety.
11. Phase 4: quality transforms.
12. Implement `/measure standardise` and scale/notes harmonisation rules aligned to boilerplate expectations.
13. Add batch edit operation with dry-run preview mode.
14. Phase 5: migration and deprecation.
15. Implement `/measure import-bptui` migration path and migration diagnostics.
16. Document bptui deprecation timeline and margo-first migration guidance.
17. Phase 6: docs and adoption.
18. Add cookbook docs for `load -> edit -> validate -> save -> boilerplate report`.
19. Add release notes and example fixtures for each supported format.
