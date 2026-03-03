# Measures Quickstart

## 1. Goal

This guide shows the shortest path to load a measures file, inspect records, edit fields, validate quality, review changes, and save.

## 2. Start margo

Launch the REPL.

```bash
margo
```

## 3. Load a measures source

Load a specific file.

```text
/measure load /path/to/boilerplate_unified.json
```

You can also rely on auto-discovery if your file is in common locations.

```text
/measure load
```

## 4. Confirm workspace source

Check path, detected format, record count, and dirty state.

```text
/measure source
```

## 5. Inspect records

List records, with optional fuzzy pattern filtering.

```text
/measure list
/measure list science
```

Show full details for one record.

```text
/measure show trust_science
```

## 6. Edit records

Add a new record.

```text
/measure add wellbeing_index
```

Edit one field.

```text
/measure edit wellbeing_index description wellbeing index summary score
```

Rename and delete records when needed.

```text
/measure rename wellbeing_index wellbeing_scale
/measure delete wellbeing_scale
```

## 7. Run quality checks

Run baseline validation.

```text
/measure validate
```

Export names missing a field. The default field is `description`.

```text
/measure export-missing
/measure export-missing reference
```

## 8. Review changes

See in-session diffs compared with the loaded baseline.

```text
/measure diff
```

## 9. Save

Save back to the loaded source path.

```text
/measure save
```

Save to a new path.

```text
/measure save /path/to/measures_db.json
```

## 10. Supported input and output formats

The current workspace supports these file types.

```text
boilerplate_unified.json
measures_db.json
variable_metadata.tsv
variable_metadata.csv
```
