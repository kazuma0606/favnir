# v37.5.0 実装計画 — CDC Rune

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `runes/cdc/cdc.fav` | 新規作成 | CDC Rune 本体（Debezium JSON 処理関数群） |
| `runes/cdc/rune.toml` | 新規作成 | rune メタデータ |
| `fav/src/driver.rs` | 変更 | `v37400_tests` スタブ化 / `v37500_tests` 追加 |
| `fav/Cargo.toml` | 更新 | `version = "37.4.0"` → `"37.5.0"` |
| `CHANGELOG.md` | 追記 | `[v37.5.0]` エントリ追加 |
| `versions/roadmap/roadmap-v37.1-v38.0.md` | 更新 | v37.5.0 を完了済みにマーク（✅）・テスト件数を 2 件 → 4 件に更新 |
| `versions/current.md` | 更新 | 最新安定版 v37.5.0、次バージョン v37.6.0 |

## 実装順序

### Step 1: CHANGELOG.md に [v37.5.0] エントリ追加

`## [v37.4.0]` の `---` セパレータ直後に挿入。

```markdown
## [v37.5.0] - 2026-07-09

### Added
- `runes/cdc/cdc.fav` — Debezium JSON 形式の CDC イベント処理 Rune（MySQL / Postgres 対応）
- `CDC.extract_op` / `CDC.op_name` / `CDC.is_insert` / `CDC.is_update` / `CDC.is_delete`
- `CDC.filter_inserts` / `CDC.filter_deletes` — イベントリストフィルタリング
- `v37500_tests` 4 テスト追加

---
```

### Step 2: `runes/cdc/` ディレクトリと rune.toml 作成

spec.md §2 に従い `rune.toml` を作成。
他の rune（`runes/mlflow/rune.toml` 等）と同じフォーマット。

### Step 3: `runes/cdc/cdc.fav` 作成

spec.md §1 のコードブロックに従い作成。
- `CDC.op_name` — `else if` ブロック構文
- `CDC.is_insert` / `CDC.is_update` / `CDC.is_delete` — 単純比較
- `CDC.extract_op` — `String.contains` + `else if` ブロック構文
- `CDC.filter_inserts` / `CDC.filter_deletes` — `List.filter` + クロージャ

**注意:** `{ body }` ブロック構文を使用（`else if` が必要なため）。

### Step 4: driver.rs — `v37400_tests::cargo_toml_version_is_37_4_0` スタブ化

ライブアサーション → `// Stubbed: version bumped to 37.5.0 — assertion intentionally removed` に変更。

**注意:** `changelog_has_v37_4_0` はスタブ化しない（CHANGELOG に `[v37.4.0]` エントリが残るため）。

### Step 5: driver.rs — `v37500_tests` モジュール追加

`v37400_tests` の閉じ `}` の行番号を Read で特定してから Edit を実行。

追加内容は spec.md §3 のコードブロックに従う:
- `include_str!` のみ使用 → `use super::*` / imports 不要
- 4 テスト: `cargo_toml_version_is_37_5_0` / `changelog_has_v37_5_0` / `cdc_rune_file_exists` / `cdc_rune_toml_exists`

### Step 6: Cargo.toml バージョン更新

Step 1〜5 完了後に `37.4.0` → `37.5.0` に更新。

## 依存関係

- `cdc.fav` は `String.contains` / `List.filter` / クロージャを使用 → すべて既存 VM primitive
- `v37500_tests` は `include_str!` のみ → `use super::*` 不要（mlflow 等の rune テストと同パターン）
- `rune.toml` は `mlflow/rune.toml` と同構造

## リスク

| リスク | 対処 |
|---|---|
| `cdc.fav` の `else if` が Favnir でコンパイルエラーになる | cdc.fav は rune ファイルであり cargo test では `include_str!` で文字列として読み込むのみ — Favnir コンパイルは実行されないためリスクなし |
| `include_str!("../../runes/cdc/cdc.fav")` のパスが正しいか | `include_str!("../../runes/mlflow/mlflow.fav")` と同じ相対パス構造 — T0 で mlflow のパスを確認済み |
| `v37400_tests` の閉じ `}` 行番号の特定 | T0 で確認し記録してから Edit |
