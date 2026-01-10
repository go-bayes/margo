## Short term questions
1. Should we let users compile projects once settings are fixed, and accept harder debugging to reduce coding?
2. Allow users to choose cut points for continuous vars when creating binary versions.
3. Plan extensibility for time-varying confounders in the framework.
4. Enable confounders from the same wave as exposure when exposure cannot affect them.

## TUI planning
1. Keep margo stable and plan a separate margot TUI track using the latest ratatui refactor.
2. Use a pipeline layout with baseline, exposure, and outcomes tiles moving right to left.
3. For lmtp, show multiple exposure tiles with time-varying confounders on a timeline.
4. Use tachyonfx for motion and draw inspiration from tek for crisp borders and typography.
5. Decide how much animation aids understanding without distracting from selection tasks.
6. Decide between fixed columns or flowing tiles to keep focus and keyboard navigation clear.
7. Decide input style: list, search, or hybrid, and show defaults and overrides.
8. Decide how selections write templates while preserving user edits and config preferences.
9. Decide release strategy, optional subcommand or feature flag, plus fallback when terminals lack effects.
10. Consider a TUI that organizes variables left to right as confounders, exposures, time-varying confounders, time-varying outcomes, and end-of-study outcomes.
