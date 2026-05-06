# Favnir Post-v1.0 Roadmap

日付: 2026-04-30

---

## Phase A: Favnir v2.0.0 言語機能拡充

v1.0.0 の安定化と並行して、v2.0.0 に向けた言語機能を順次追加する。

### A-0. 構文リネーム（v2.0.0 で確定）

- `trf` → `stage`
- `flw` → `seq`
- `cap` → `interface`
- v1.x → v2.0.0 移行ガイドを提供

### A-1. 抽象化システム（`interface` / `abstract`）

- `interface` 宣言・`impl` 手書き・`impl` 自動合成・`with` 糖衣構文
- `abstract type` / `abstract stage` / `abstract seq`
- `invariant` の型システム統合
- `Gen` interface（`Stat.one<T>` の基盤）
- `Field` interface（代数構造 + `fav-algebraic-structures.md` に基づく）

### A-2. `Task<T>` 非同期モデル

- `async fn` / `async stage` / `async seq` 宣言
- `bind` による `Task<T>` 自動解除（`await` キーワードなし）
- `Task.run` / `Task.all` / `Task.race` / `Task.timeout`
- `chain` による `Task<T>!` 一括処理

### A-3. `stat` ルーン

- `Stat.int/float/bool/string/choice` プリミティブ生成
- `Stat.normal/uniform` 分布駆動生成
- `Stat.one<T>/list<T>` 型駆動生成（`Gen` interface 依存）
- `Stat.profile<T>/drift<T>` 統計的推論
- `Stat.sample/sample_outliers` サンプリング

### A-4. `validate` ルーンファミリー

- `validate` 共通型（`ValidationError` 等）
- `validate.field`（フィールドレベル検証）
- `validate.flow`（パイプライン・ドメイン検証）
- `validate.db`（DB 行・CSV 行検証）

### A-5. キーワードと標準状態の拡充

- `std.states` ルーン（`Email/PosInt/NonEmptyString/Url` 等）
- `Stream<T>`（遅延非同期シーケンス、`collect/yield` との統合）
- `fav bundle` / `fav explain --format json`（到達可能性解析基盤）

---

## Phase B: Favnir 1.x hardening

`v1.0.0` のあと、まず実装の完成度を上げる。

### B-0. parity / cleanup

- ignored VM parity test 解消
- `Unknown` fallback の削減
- VM / WASM 差異の縮小
- diagnostics の改善

### B-1. self-host expansion

- parser subset
- checker subset
- formatter
- explain generator

を段階的に `.fav` へ移す

### B-2. stronger metadata

- explain JSON schema 固定
- trace JSON schema 固定
- artifact metadata schema 固定

これは Veltra に直結する。

---

## Phase C: Veltra-ready APIs

言語本体と製品の境界を API として固定する段階。

### C-1. notebook kernel protocol

- execute cell
- reset
- shutdown
- partial outputs
- structured error

### C-2. explain/trace API

- cell/unit explain
- `stage/seq` explain
- artifact summary
- emitted event summary

### C-3. artifact registry interface

- rune metadata
- artifact info
- compatibility/version checks

---

## Phase D: Veltra MVP

製品名 `Veltra` 側の最小実装。

### D-1. notebook format

- `.vnb`
- `.vnb.out.json`
- artifact linkage

### D-2. notebook runtime

- Favnir kernel
- explain pane
- artifact pane
- trace pane

### D-3. GCP-backed MVP

- Cloud Run
- GCS
- Cloud SQL
- BigQuery

### D-4. hosted workflow

- saved notebooks
- execution history
- artifact storage
- shared workspaces

---

## Phase E: product differentiation

ここからが「他にはない」方向。

### E-1. explain-first workflow

- PR/CI で effect diff
- artifact review
- flow graph

### E-2. AI-native authoring

- safe composition suggestion
- metadata-aware completion
- explainable generated flows

### E-3. data platform integration

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
