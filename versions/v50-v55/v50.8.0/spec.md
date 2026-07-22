# Spec: v50.8.0 — ドキュメントサイト DX 3.0 記事

## 概要

DX 3.0 スプリント（v50.1〜v51.0）で実装した 2 つの機能のドキュメントページを新規作成する。

1. `site/content/docs/tools/diagnostics.mdx` — 統一診断出力・`fav explain --error` の使い方
2. `site/content/docs/tools/trace-watch.mdx` — `fav run --trace/--watch` のデバッグパターン

---

## 背景・対象機能

### diagnostics.mdx の対象機能（v50.1〜v50.3 で実装済み）

- **v50.1.0**: `error_catalog.rs` の全エラーコードに `suggestion` テキスト追加
- **v50.2.0**: `fav check --json` / LSP diagnostics / CLI stderr で `suggestion` + `span` 一貫出力
- **v50.3.0**: `fav explain --error <code>` 正式導線追加・全コード explain テキスト追加

### trace-watch.mdx の対象機能（v50.7.0 で実装済み）

- **v50.7.0**: `fav run --trace` の構造化ログ（`[trace] stage=NAME  out=VALUE`）
- **v50.7.0**: `--watch` フィールド追跡（`[watch] target: — → value  (stage: name)`）
  - CLI `--watch` フラグ解析は未実装（API のみ）— docs には「将来対応予定」として記載

---

## 成果物仕様

### 1. `diagnostics.mdx`

**ファイルパス**: `site/content/docs/tools/diagnostics.mdx`

**必須コンテンツ:**
- H1: `# 診断出力 (Diagnostics)`
- 統一診断フォーマット（JSON / LSP / CLI の例）
- `fav check --json` の出力例（`suggestion` + `span` フィールド）
- `fav explain --error <code>` の使い方
- `fav explain --error --list` による全コード一覧表示
- 主要エラーコードの説明例（E0001 / E0018 程度）
- キーワード: `diagnostics`, `fav explain`, `suggestion`（テスト用）

**最小文字数**: 実用的なリファレンスとして 300 文字以上

### 2. `trace-watch.mdx`

**ファイルパス**: `site/content/docs/tools/trace-watch.mdx`

**必須コンテンツ:**
- H1: `# トレース & ウォッチ (Trace & Watch)`
- `fav run --trace` の使い方と出力例（`[trace] stage=NAME  out=VALUE`）
- 既存 `fav run --debug`（DAP）との違い
- `--watch` フィールド追跡の説明（v50.7.0 で API 実装済み、CLI フラグは将来対応）
- パイプラインデバッグのユースケース例
- キーワード: `--trace`, `[trace]`, `trace-watch`（テスト用）

**最小文字数**: 実用的なリファレンスとして 300 文字以上

---

## テスト仕様

### `docs_diagnostics_page_exists`

```rust
let content = include_str!("../../site/content/docs/tools/diagnostics.mdx");
assert!(content.len() >= 300, "diagnostics.mdx is too short");
assert!(content.contains("diagnostics") || content.contains("fav explain") || content.contains("suggestion"));
```

### `docs_trace_watch_page_exists`

```rust
let content = include_str!("../../site/content/docs/tools/trace-watch.mdx");
assert!(content.len() >= 300, "trace-watch.mdx is too short");
assert!(content.contains("--trace") || content.contains("[trace]") || content.contains("trace-watch"));
```

---

## バージョン要件

- `fav/Cargo.toml` version: `50.8.0`
- テスト数: 3105 → **3107**
  - `v508000_tests` 3 件追加（`cargo_toml_version_is_50_8_0` + docs 用 2 件）、`v507000_tests::cargo_toml_version_is_50_7_0` 1 件削除 = 純増 +2
  - ロードマップ記載の「Rust テスト 2 件」は docs 用テスト（`docs_diagnostics_page_exists` / `docs_trace_watch_page_exists`）のみのカウント。バージョン assertion テスト（`cargo_toml_version_is_50_8_0`）は毎バージョン共通で追加・前バージョン削除するため差し引きゼロとして慣例上カウント外。

---

## 完了条件

- `site/content/docs/tools/diagnostics.mdx` が存在し、300 文字以上
- `site/content/docs/tools/trace-watch.mdx` が存在し、300 文字以上
- `cargo test` 3107 tests passed, 0 failed
- `cargo clippy -- -D warnings` クリーン
- `v508000_tests` 3 件:
  - `cargo_toml_version_is_50_8_0`
  - `docs_diagnostics_page_exists`
  - `docs_trace_watch_page_exists`

---

## ロードマップ対応

roadmap-v50.1-v51.0.md v50.8.0 より:

> `site/content/docs/tools/diagnostics.mdx` — 統一された診断出力・`fav explain --error` の使い方。
> `site/content/docs/tools/trace-watch.mdx` — `fav run --trace/--watch` のデバッグパターン。

**差異・制約の明記:**
- `trace-watch.mdx` に記載する `--watch` は v50.7.0 時点で CLI フラグ未実装（API のみ）。ドキュメントには「将来対応予定」として記載し、現在の動作（`set_watch_fields` テスト API 経由）との乖離を明示する。
- ロードマップの「Rust テスト 2 件」は docs 用テストのみカウント。本 spec では慣例に従いバージョン assertion テスト（`cargo_toml_version_is_50_8_0`）を含む 3 件を実装し、前バージョン assertion 1 件を削除することで純増 +2 を達成する。
