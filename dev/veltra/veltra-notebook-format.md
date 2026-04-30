# Veltra Notebook Format

## Goal
Veltra notebook should stay text-first, git-friendly, and easy to inspect. The base direction follows Forge notebook `.fnb`:

- notebook source is a readable Markdown file
- code cells are fenced blocks
- execution output is stored separately
- runtime and UI can regenerate outputs at any time

For Veltra, the product value added on top is:

- explain metadata
- trace metadata
- artifact metadata
- lightweight data previews

## Recommended Files
A notebook is split into two files.

- `analysis.vnb`
  - notebook source
  - Markdown-first
  - Favnir code cells
  - light notebook metadata
- `analysis.vnb.out.json`
  - execution outputs
  - explain snapshots
  - trace snapshots
  - artifact info snapshots

Optional artifact output:

- `artifacts/analysis.fvc`

This keeps the notebook itself stable in git while letting outputs be refreshed or discarded.

## Why Markdown-First
Compared to a JSON-first notebook document, Markdown-first is better for Veltra MVP because:

- diffs are readable
- merge conflicts are less painful
- notebooks can be edited without a custom UI
- Favnir code and prose can live together naturally
- it matches the Forge notebook direction already explored

Veltra should treat the notebook as a source document, not as a binary state container.

## File Extension
Recommended extension:

- `.vnb` = Veltra Notebook

This keeps the format separate from Forge `.fnb` while staying short and recognizable.

## Source File Structure
A `.vnb` file is a Markdown document with optional front matter and fenced code cells.

Example:

```md
---
title: User Import Analysis
runtime: favnir
version: 0.1
---

# Setup

```fav name="setup"
bind rows <- Csv.read("users.csv")
rows
```

# Normalize

```fav name="normalize"
trf NormalizeName: String -> String = |name| {
    String.trim(name)
}
```

# Run

```fav name="run"
public fn main() -> Unit !Io {
    IO.println("hello")
}
```
```

## Front Matter
Minimal front matter should be allowed.

Recommended fields:

- `title`
- `runtime`
- `version`
- `default_rune` optional
- `entry_cell` optional

Example:

```yaml
---
title: User Import Analysis
runtime: favnir
version: 0.1
default_rune: data.import
entry_cell: run
---
```

This metadata should stay intentionally small. Execution state should not be embedded here.

## Cell Syntax
Use fenced code blocks with language `fav`.

Recommended inline attributes:

- `name`
- `hidden=true`
- `skip=true`
- `artifact=true`
- `explain=true`

Example:

```md
```fav name="db_setup" hidden=true
bind conn <- Db.connect("analytics")
conn
```
```

### Attribute Meaning
- `name`
  - stable cell identifier
- `hidden=true`
  - hidden by default in UI
- `skip=true`
  - excluded from normal run-all
- `artifact=true`
  - marks a cell as interesting for artifact build output
- `explain=true`
  - ask UI/runtime to surface explain output by default

## Shared Scope Model
Veltra should follow the Forge notebook model here.

- cells execute in shared notebook scope
- definitions from earlier cells are visible to later cells
- resetting the notebook clears shared state
- rerunning a cell overwrites the notebook state from that cell forward only through explicit execution, not automatically

This gives a notebook feel close to Jupyter, but with Favnir semantics.

## Output File Structure
Outputs are stored in `.vnb.out.json`.

Recommended top-level structure:

```json
{
  "version": "0.1",
  "notebook": "analysis.vnb",
  "cells": {
    "setup": {
      "status": "ok",
      "stdout": [],
      "inspect": [],
      "result_preview": {
        "kind": "list",
        "summary": "3 rows"
      },
      "explain": null,
      "trace": null,
      "artifact": null,
      "executed_at": "2026-04-29T12:00:00Z"
    }
  }
}
```

## Per-Cell Output Fields
Recommended per-cell fields:

- `status`
  - `ok`, `error`, `skipped`
- `stdout`
  - text outputs
- `inspect`
  - structured inspect outputs
- `result_preview`
  - lightweight preview of the cell result
- `explain`
  - explain snapshot
- `trace`
  - trace snapshot
- `artifact`
  - artifact path and info summary if built
- `executed_at`
  - timestamp
- `duration_ms`
  - optional runtime metric
- `error`
  - structured error payload when failed

## Explain Snapshot
Veltra should make explain a first-class output.

Recommended structure:

```json
{
  "entry": "normalize",
  "types": ["String -> String"],
  "effects": ["Pure"],
  "flows": ["NormalizeName"],
  "notes": []
}
```

This can grow later, but the key is that explain data stays detached from notebook source.

## Trace Snapshot
Trace output should also be first-class.

Recommended structure:

```json
{
  "events": ["UserCreated"],
  "trace_enabled_functions": ["main"],
  "steps": []
}
```

The exact trace schema can evolve later. For MVP, a compact static/dynamic summary is enough.

## Artifact Snapshot
When a notebook or cell triggers `fav build`, the output record should keep:

- artifact path
- entry signature
- effect summary
- emitted events summary
- bytecode summary

Example:

```json
{
  "path": "artifacts/analysis.fvc",
  "entry_signature": "() -> Unit !Io",
  "effects": ["!Io", "!Emit<UserCreated>"],
  "summary": {
    "functions": 4,
    "globals": 3,
    "synthetic_closures": 1
  }
}
```

## Suggested Project Layout
Recommended default layout:

```text
notebooks/
  user-import.vnb
  user-import.vnb.out.json
artifacts/
  user-import.fvc
runes/
  ...
settings/
  ...
```

This keeps notebooks, artifacts, and reusable runes separate.

## Veltra UI Implications
This file model keeps UI cost low.

The UI only needs to:

- load Markdown notebook source
- parse `fav` code fences
- call runtime for run/explain/build
- update `.vnb.out.json`
- render result/explain/trace/artifact panes

A full custom binary notebook format is unnecessary for MVP.

## Relationship to Favnir
Suggested role split:

- Favnir
  - language
  - CLI
  - runtime
  - artifact format
- Veltra
  - notebook product
  - execution UI
  - explain/trace/artifact views
  - managed cloud experience

Veltra notebooks should therefore be product documents powered by Favnir execution.

## MVP Recommendation
For the first Veltra notebook implementation, keep the scope intentionally small.

Required:

- `.vnb` Markdown source
- `fav` fenced code cells
- `.vnb.out.json` output file
- shared-scope execution
- explain pane data
- artifact info snapshot

Later:

- `.ipynb` export
- richer data previews
- comments/collaboration
- scheduled notebooks
- notebook-level artifact publishing

## Decision
The recommended Veltra notebook format is:

- Markdown-first source file: `.vnb`
- separate output state file: `.vnb.out.json`
- optional built artifact: `.fvc`
- shared notebook scope
- first-class explain, trace, and artifact metadata

This is the cleanest continuation of the Forge notebook direction, while fitting Veltra's product goals much better than a heavy JSON-only notebook format.
