# Favnir v1.0.0 Done Definition

日付: 2026-04-30

## 方針

`v1.0.0` は「すべてを入れる版」ではない。

目的は:

- 言語仕様を固定する
- 実行系を一貫させる
- self-hosting への入口を作る
- tooling の最小完成形を揃える

Veltra や cloud/notebook/product 拡張は、その後の開発で扱う。

---

## v1.0.0 Done Definition

### 1. 言語仕様

以下が固定されていること:

- `type`
- `bind`
- `trf`
- `flw`
- `rune`
- effect
- `T? / T!`
- `chain`
- `cap`
- `async/await`
- `public / internal / private`
- `namespace / use / workspace`

つまり、これ以上コア構文を大きく揺らさない状態。

### 2. 実行系の一貫性

以下が同じ意味論で動くこと:

- `fav run`
- `fav test`
- `fav build`
- `fav exec`
- `fav build --target wasm`

具体的には:

- VM が正規実行系
- `.fvc` artifact が安定
- `.wasm` backend が MVP として安定
- ignored parity test を減らし、実行差異を意識的に管理できる

### 3. Tooling

最低限そろっていること:

- `fav fmt`
- `fav lint`
- `fav test`
- `fav explain`
- `fav exec --info`

加えて:

- structured diagnostics
- structured test result
- structured explain output

の入口があること

### 4. self-hosting 入口

v1.0.0 で完全 self-host は必須ではない。
ただし、入口は必要。

必要条件:

- selfhost 対象の優先順位が決まっている
- subset が定義されている
- bootstrap 方針が決まっている

最低限、次のどれかが `.fav` 側へ移り始めていると強い。

- formatter subset
- explain subset
- parser subset

### 5. Editor / LSP 最小実装

以下の最小セット:

- hover
- diagnostics
- go to definition
- symbol outline

Favnir は type/effect/flow metadata が強みなので、LSP は v1 に入る価値が高い。

### 6. Package / project model

以下が固定されていること:

- `rune`
- `workspace`
- `fav.toml`
- module / namespace 解決
- 可視性の解決順

### 7. Documentation

v1.0.0 時点で必要:

- core spec
- examples
- CLI reference
- roadmap
- migration / design notes

特に「何ができて何がまだできないか」が明確であること。

---

## v1.0.0 に入れすぎないもの

### 言語本体では後回し

- richer constraint system
- macro
- full coroutine model
- field-level visibility
- advanced effect polymorphism
- transport-specific syntax

### 製品化では後回し

- notebook UI
- shared cloud workspace
- BigQuery / GCS first-class connectors
- collaboration
- scheduler
- registry/signing UI
- artifact marketplace

---

## Post-1.0 Roadmap

## Phase A: Favnir 1.x hardening

`v1.0.0` のあと、まず言語本体の完成度を上げる。

### A-1. parity / cleanup

- ignored VM parity test 解消
- `Unknown` fallback の削減
- VM / WASM 差異の縮小
- diagnostics の改善

### A-2. self-host expansion

- parser subset
- checker subset
- formatter
- explain generator

を段階的に `.fav` へ移す

### A-3. stronger metadata

- explain JSON schema 固定
- trace JSON schema 固定
- artifact metadata schema 固定

これは Veltra に直結する。

---

## Phase B: Veltra-ready APIs

言語本体と製品の境界を API として固定する段階。

### B-1. notebook kernel protocol

- execute cell
- reset
- shutdown
- partial outputs
- structured error

### B-2. explain/trace API

- cell/unit explain
- `trf/flw` explain
- artifact summary
- emitted event summary

### B-3. artifact registry interface

- rune metadata
- artifact info
- compatibility/version checks

---

## Phase C: Veltra MVP

製品名 `Veltra` 側の最小実装。

### C-1. notebook format

- `.vnb`
- `.vnb.out.json`
- artifact linkage

### C-2. notebook runtime

- Favnir kernel
- explain pane
- artifact pane
- trace pane

### C-3. GCP-backed MVP

- Cloud Run
- GCS
- Cloud SQL
- BigQuery

### C-4. hosted workflow

- saved notebooks
- execution history
- artifact storage
- shared workspaces

---

## Phase D: product differentiation

ここからが「他にはない」方向。

### D-1. explain-first workflow

- PR/CI で effect diff
- artifact review
- flow graph

### D-2. AI-native authoring

- safe composition suggestion
- metadata-aware completion
- explainable generated flows

### D-3. data platform integration

- BigQuery / GCS / scheduler
- registry / signing
- policy checks

---

## 実務的な進め方

### いまやること

- `v1.0.0` は既存 roadmap を完遂する
- 新しい大機能は入れすぎない
- docs と CLI を揃える

### `v1.0.0` 後にやること

- Veltra-ready API を切り出す
- notebook/runtime 境界を固定する
- hosted 体験を作る

---

## 一言でいうと

`v1.0.0` は:

> Favnir を「使える言語」として締める版

`post-1.0` は:

> Favnir を核に Veltra を「売れる製品」に育てる版
