# Favnir vs Veltra

## Purpose
This note separates the language roadmap from the product roadmap.

Favnir and Veltra are related, but they are not the same thing.

- **Favnir** is the language, runtime, artifact format, and developer tooling.
- **Veltra** is the notebook product and managed platform built on top of Favnir.

This distinction matters because Favnir is still evolving. Veltra should reuse Favnir where possible, but should not force all product requirements into the language core.

## Short Version
- Favnir = `language/runtime/tooling`
- Veltra = `notebook/product/platform`

Favnir should own semantics and execution.
Veltra should own user experience and hosted workflow.

## What Belongs to Favnir
Favnir should continue to own:

- syntax and parser
- type/effect checker
- `fn`, `trf`, `flw`, `rune`
- generics and `cap`
- `bind`, `chain`, `match`, pattern system
- typed IR
- bytecode
- VM
- `.fvc` artifact format
- CLI
  - `fav run`
  - `fav check`
  - `fav build`
  - `fav exec`
  - later `fav fmt`, `fav lint`, `fav test`, `fav explain`
- stdlib
- explain engine
- trace engine
- notebook kernel protocol later

Favnir should be the execution and semantics layer.

## What Belongs to Veltra
Veltra should own:

- notebook file format
- notebook output file format
- notebook UI
- workspace/project model
- execution orchestration
- notebook storage/history
- explain pane
- trace pane
- artifact pane
- collaboration/workspace features
- hosted runtime
- GCP integrations
  - BigQuery
  - GCS
  - scheduler
  - artifact storage
- connector management
- SaaS/product concerns

Veltra should be the product experience layer.

## Why They Should Be Separate
If Favnir absorbs too much notebook/product logic, the language becomes harder to evolve cleanly.
If Veltra tries to reimplement language/runtime features, the product becomes fragile.

The clean split is:

- Favnir defines how code works
- Veltra defines how users work with that code

## Favnir Roadmap Areas That Directly Help Veltra
The current Favnir roadmap already builds several layers Veltra needs.

### Most important milestones for Veltra
- **v0.6.0**
  - typed IR
  - bytecode
  - VM
  - `.fvc` artifact
  - `fav build` / `fav exec`
- **v0.7.0**
  - usable stdlib
- **v0.8.0**
  - `fmt`, `lint`, `test`, `explain`
- **v0.9.0**
  - WASM/backend sandbox direction

These are strong foundations for a notebook product.

## Main Gaps Between Favnir and Veltra
There are still product-specific needs that do not belong in Favnir core.

### 1. Notebook execution model
Favnir today is mainly file-based and `main`-oriented.
Veltra wants:

- cell execution
- shared notebook scope
- rerun by cell
- reset notebook scope
- named cell execution

This is a notebook runtime concern, not a language-core concern.

### 2. Explain granularity
Favnir explain is currently strongest at file or artifact level.
Veltra needs explain at:

- cell level
- `trf` level
- `flw` level
- possibly selection level later

Favnir should expose structured explain data.
Veltra should decide how to present it.

### 3. Structured notebook outputs
Favnir has `emit`, trace, and `exec --info`.
Veltra additionally needs notebook-oriented output payloads:

- stdout
- inspect output
- trace summary
- artifact snapshot
- explain snapshot
- result preview

This belongs to Veltra output/state design.

### 4. Cloud connectors
Favnir can own generic effects such as:

- `Db`
- `Io`
- `Network`
- `File`

Veltra can own hosted/cloud integrations such as:

- BigQuery
- GCS
- scheduled execution
- artifact registry integration

This keeps Favnir portable and Veltra product-specific.

### 5. Product metadata
Favnir has:

- `namespace`
- `rune`
- `fav.toml`

Veltra additionally needs:

- notebook metadata
- workspace metadata
- run history
- output retention
- artifact retention
- connector configuration

That should remain product-side.

## Recommended Boundary
### Favnir responsibilities
- parser/checker/compiler/vm
- CLI
- artifact format
- stdlib
- explain engine
- trace engine
- structured machine-readable explain/trace outputs
- notebook kernel protocol later

### Veltra responsibilities
- `.vnb` source format
- `.vnb.out.json` output format
- notebook editor and panes
- execution API
- persistence and history
- connectors and cloud integration
- team/workspace layer
- billing and product operations

## How Veltra Should Use Favnir
Veltra should call into Favnir for:

- `run`
- `check`
- `build`
- `exec --info`
- explain generation
- trace generation

Veltra should not duplicate Favnir execution logic.

## Veltra-First Additions Favnir Should Eventually Expose
Favnir does not need notebook UX, but it should expose the data Veltra needs.

Recommended future Favnir-facing interfaces:

- structured explain JSON
- structured trace JSON
- notebook kernel execution API
- cell-scope execution hooks
- artifact metadata API

These are language/runtime APIs that the product can consume.

## Practical Rule
When a new requirement appears, decide it like this:

### Put it in Favnir if:
- it changes language semantics
- it changes execution semantics
- it changes type/effect behavior
- it changes artifact/runtime behavior
- it should work equally in CLI and product

### Put it in Veltra if:
- it is about notebook UX
- it is about cloud orchestration
- it is about collaboration or persistence
- it is about presenting Favnir data to end users
- it is product-specific integration logic

## Example Split
### Favnir
- `fav build main.fav -o main.fvc`
- `fav exec main.fvc --info`
- explain output as structured JSON
- trace output as structured JSON

### Veltra
- notebook cell "Run"
- notebook pane "Explain"
- notebook pane "Artifact"
- save `.vnb.out.json`
- render previews and traces
- call BigQuery/GCS-backed jobs

## Product Framing
Recommended relationship:

- **Favnir** = language and execution engine
- **Veltra** = explainable data notebook and platform powered by Favnir

This keeps both names useful and avoids forcing one layer to solve the other's job.

## Conclusion
Favnir is still evolving, but that is not a blocker for Veltra planning.

Favnir should keep evolving as a clean language/runtime/tooling stack.
Veltra should be designed in parallel as the notebook/platform layer that consumes Favnir.

The correct relationship is not:

- Favnir vs Veltra

It is:

- Favnir underneath
- Veltra on top
