# Favnir Post-v1.0 Roadmap

日付: 2026-04-30

---

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

### `v1.0.0` 後にやること

- Veltra-ready API を切り出す
- notebook/runtime 境界を固定する
- hosted 体験を作る

---

## 一言でいうと

`post-1.0` は:

> Favnir を核に Veltra を「売れる製品」に育てる版
