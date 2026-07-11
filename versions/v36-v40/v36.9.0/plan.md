# v36.9.0 実装計画 — v37.0 前調整・安定化

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/lint.rs` | 変更 | W025 メッセージに `[see also: E0380 schema_field_missing]` 追加 |
| `fav/src/driver.rs` | 変更 | `cmd_validate` サマリー行追加 / `v36800_tests` スタブ化 / `v36900_tests` 追加 |
| `fav/Cargo.toml` | 更新 | `version = "36.8.0"` → `"36.9.0"` |
| `CHANGELOG.md` | 追記 | `[v36.9.0]` エントリ追加 |
| `site/content/docs/data-quality.mdx` | 新規作成 | Data Quality First 機能群のドキュメント |
| `versions/current.md` | 更新 | 最新安定版 v36.9.0、次バージョン v37.0.0 |
| `versions/roadmap/roadmap-v36.1-v37.0.md` | 更新 | v36.9.0 完了済みにマーク（✅） |

## 実装順序

### Step 1: CHANGELOG.md に [v36.9.0] エントリ追加

`## [v36.8.0]` の `---` セパレータ直後に挿入（日付は実装当日）。

### Step 2: lint.rs — W025 メッセージ更新

`check_w025_schema_mismatch` 関数内の `format!` 文字列を変更:
- 変更前: `"field `{}` not found in schema `{}` (available: {})"`
- 変更後: `"field `{}` not found in schema `{}` (available: {}) [see also: E0380 schema_field_missing]"`

変更は 1 行のみ。周辺コードへの影響なし。

### Step 3: driver.rs — `cmd_validate` サマリー行追加

挿入位置: `if has_errors { process::exit(1); }` の直後。

```rust
// 6. サマリー出力（v36.9.0）
let total_fields: usize = schema_defs.iter().map(|sd| sd.fields.len()).sum();
println!(
    "Validated: {} schema(s), {} field(s) checked",
    schema_defs.len(),
    total_fields
);
```

### Step 4: site/content/docs/data-quality.mdx 新規作成

`site/content/docs/` 以下に `data-quality.mdx` を作成。spec.md の内容に従う。

### Step 5: driver.rs — `v36800_tests::cargo_toml_version_is_36_8_0` スタブ化

ライブアサーション → `// Stubbed: version bumped to 36.9.0` に変更。

### Step 6: driver.rs — `v36900_tests` モジュール追加

`v36800_tests` の閉じ `}` の行番号を Read で特定してから Edit を実行する。

### Step 7: Cargo.toml バージョン更新

Step 2〜6 完了・コンパイルエラー解消後に `36.8.0` → `36.9.0` に更新。

## 依存関係

- `lint.rs` の `check_w025_schema_mismatch` は単一 `format!` 呼び出しで message を生成 → 1 行変更で完了
- `cmd_validate` の `schema_defs` は `Vec<&SchemaDef>` — `.len()` と `.iter().map(|sd| sd.fields.len()).sum()` でサマリー計算可能
- `v36900_tests` は `use super::*` 不要（`include_str!` のみ使用）
- `validate_summary_line_added` テストは `include_str!("driver.rs").contains("Validated: {} schema(s)")` でソースレベル確認（`cmd_validate` は I/O 関数なので副作用なしテストが困難なため）

## リスク

| リスク | 対処 |
|---|---|
| W025 既存テストがメッセージ文字列をアサートしている場合に失敗 | T0 で `v36300_tests` 等の W025 テストを確認し、影響範囲を特定する |
| `cmd_validate` サマリー行が既存テストの出力比較を壊す | T0 で `cmd_validate` 呼び出しテストを確認 |
| `data-quality.mdx` のパスが site 構造と不一致 | T0 で `site/content/docs/` 直下の既存 mdx を確認 |
