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
