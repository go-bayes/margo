## Short term questions
1. Should we let users compile projects once settings are fixed, and accept harder debugging to reduce coding?

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
