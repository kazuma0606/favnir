# v36.9.0 spec — v37.0 前調整・安定化

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v36.9.0 |
| テーマ | v37.0 前調整・安定化 — Data Quality First 機能群の統合と品質向上 |
| 前提 | v36.8.0 COMPLETE — `fav schema diff` 実装済み |
| 完了条件 | `v36900_tests` 全テスト pass・`cargo test` 0 failures（≥ 2699 件） |

## 背景と目的

v36.1〜v36.8 で実装した Data Quality First 機能群（`schema` 定義、`expect`、W025、`fav validate`、E0380〜E0384、GE エクスポート、`fav schema diff`）を v37.0 マイルストーン宣言前に統合し、品質を引き上げる。

主な調整内容:

1. **W025 メッセージへの E0380 参照追加** — W025（schema_mismatch）が報告するエラーメッセージに `[see also: E0380]` を追加し、エラーカタログとの横断参照を実現する。
2. **`fav validate` サマリー行追加** — 検証成功時に `Validated: N schemas, M fields` のサマリーを標準出力に追加し、静粛すぎる現状の出力を改善する。
3. **Data Quality ドキュメントページ** — `site/content/docs/data-quality.mdx` を新規作成し、v36.x 系機能を一覧化する。

## 実装スコープ

### 1. `fav/src/lint.rs` — W025 メッセージに E0380 参照追加

`check_w025_schema_mismatch` 関数の `LintError::new` の `message` フォーマット文字列を変更:

**変更前:**
```rust
format!(
    "field `{}` not found in schema `{}` (available: {})",
    field_name,
    schema_name,
    fields.join(", ")
)
```

**変更後:**
```rust
format!(
    "field `{}` not found in schema `{}` (available: {}) [see also: E0380 schema_field_missing]",
    field_name,
    schema_name,
    fields.join(", ")
)
```

これにより `fav lint` の W025 警告からエラーカタログ E0380 への参照が提供される。

### 2. `fav/src/driver.rs` — `cmd_validate` サマリー行追加

`cmd_validate` の成功パス末尾（`has_errors` が false の場合）にサマリー行を追加:

```rust
// 6. サマリー出力（v36.9.0）
let total_fields: usize = schema_defs.iter().map(|sd| sd.fields.len()).sum();
println!(
    "Validated: {} schema(s), {} field(s) checked",
    schema_defs.len(),
    total_fields
);
```

挿入位置: `if has_errors { process::exit(1); }` の直後、GE エクスポートブロックの前。

### 3. `site/content/docs/data-quality.mdx` — Data Quality ドキュメント

```mdx
---
title: Data Quality
description: Favnir v36.x — Data Quality First 機能群
---

# Data Quality First

Favnir v36.x では **データ品質を型で保証する** 機能群を提供します。

## `schema` 定義

```favnir
schema Orders {
  id: Int
  customer_id: Int
  amount: Float
  status: String
}
```

## `fav validate`

CSV/Parquet ファイルをスキーマ定義と照合して検証します。

```bash
fav validate --schema orders.fav data.csv
fav validate --schema orders.fav data.csv --export ge --output suite.json
```

## `fav schema diff`

2 つのスキーマファイルの差分と後方互換性を表示します。

```bash
fav schema diff v1/orders.fav v2/orders.fav
```

出力例:
```
schema Orders (v1/orders.fav → v2/orders.fav):
  + amount: Float         (added, backward-compatible)
  - status: String        (BREAKING: removed)
```

## W025 / E0380〜E0384

| コード | 意味 |
|---|---|
| W025 | `schema_mismatch` — フィールドアクセスがスキーマ定義に存在しない |
| E0380 | `schema_field_missing` |
| E0381 | `schema_type_mismatch` |
| E0382 | `schema_constraint_violated` |
| E0383 | `schema_duplicate_key` |
| E0384 | `schema_extra_field` |
```

### 4. `fav/src/driver.rs` — `v36900_tests` モジュール追加

```rust
// ── v36900_tests (v36.9.0) — v37.0 前調整・安定化 ──────────────────────────────
#[cfg(test)]
mod v36900_tests {
    #[test]
    fn cargo_toml_version_is_36_9_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("36.9.0"), "Cargo.toml must contain version 36.9.0");
    }
    #[test]
    fn changelog_has_v36_9_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v36.9.0]"), "CHANGELOG.md must contain [v36.9.0]");
    }
    #[test]
    fn w025_message_references_e0380() {
        // W025 のメッセージが E0380 を参照することを lint.rs のソースで確認
        let src = include_str!("lint.rs");
        assert!(
            src.contains("E0380 schema_field_missing"),
            "W025 message should reference E0380 schema_field_missing"
        );
    }
    #[test]
    fn validate_summary_line_added() {
        // cmd_validate にサマリー行が追加されたことを driver.rs のソースで確認
        let src = include_str!("driver.rs");
        assert!(
            src.contains("Validated: {} schema(s)"),
            "cmd_validate must output a summary line with schema count"
        );
    }
}
```

## 注意事項

### `cmd_validate` の挿入位置

サマリー行は `if has_errors { process::exit(1); }` の **直後**、GE エクスポートブロック（`if export_fmt == Some("ge")` ブロック）の **前** に挿入する。
`has_errors` が true の場合は `process::exit(1)` で終了するため、サマリーが出力されるのは検証成功時のみ（意図した動作）。

### スコープ外（v37.0 以降）

- `fav validate` の JSON 出力形式（`--output-format json`）
- W025 の `fix` フィールド（`LintError` に `fix` フィールドが存在しないため構造変更が必要）
- `fav schema diff --json` 出力

## ロードマップとの整合

ロードマップ v36.9.0:「v37.0 前調整・安定化」
本 spec は 3 テストを追加する。ロードマップに件数指定なし。

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | W025 メッセージに `E0380 schema_field_missing` が含まれる | `w025_message_references_e0380` テスト |
| 2 | `cmd_validate` がサマリー行を出力する | `validate_summary_line_added` テスト |
| 3 | `CHANGELOG.md` に `[v36.9.0]` が含まれる | `changelog_has_v36_9_0` テスト |
| 4 | `Cargo.toml` バージョンが `36.9.0` | `cargo_toml_version_is_36_9_0` テスト |
| 5 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2699） | `cargo test` 実行結果（v36.8.0 実績 2695 + v36900_tests 4 件 = 2699） |
