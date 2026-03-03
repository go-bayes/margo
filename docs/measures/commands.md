# Measures Command Reference

## 1. Scope

This reference documents the `/measure` command group in the margo REPL.

## 2. Open command help

Show command usage summary.

```text
/measure
```

## 3. Load and source commands

Load a measures file into the in-session workspace.

```text
/measure load [path]
```

Show the loaded source path, file format, record count, and dirty state.

```text
/measure source
```

## 4. Inspection commands

List measures, optionally filtered by a fuzzy pattern.

```text
/measure list [pattern]
```

Show one full record by name.

```text
/measure show <name>
```

## 5. Mutation commands

Add a new record.

```text
/measure add <name>
```

Edit one field in one record.

```text
/measure edit <name> <field> <value>
```

Rename a record.

```text
/measure rename <old> <new>
```

Delete a record.

```text
/measure delete <name>
```

The command aliases `rm` and `del` are accepted for delete.

```text
/measure rm <name>
/measure del <name>
```

## 6. Validation and reporting commands

Run baseline validation for duplicate names and missing descriptions.

```text
/measure validate
```

List records missing a field. The default field is `description`.

```text
/measure export-missing [field]
```

## 7. Change review and persistence commands

Show summary of added, removed, and changed records compared with the loaded baseline.

```text
/measure diff
```

Save to the current source path.

```text
/measure save
```

Save to a new path.

```text
/measure save <path>
```

## 8. Current editable fields

The `edit` command currently supports these fields.

```text
description
reference
waves
keywords
label
scale
notes
standardised
standardised_date
items
```

## 9. Examples

Load a source and inspect records.

```text
/measure load /Users/joseph/GIT/bptui/boilerplate_unified.json
/measure source
/measure list trust
/measure show trust_science
```

Edit and save.

```text
/measure edit trust_science notes reviewed for 2026 manuscript
/measure validate
/measure diff
/measure save
```

Create, rename, and delete.

```text
/measure add temporary_measure
/measure rename temporary_measure temporary_measure_v2
/measure delete temporary_measure_v2
```
