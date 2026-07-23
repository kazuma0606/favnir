# Plan: v49.4.0 — ドキュメントサイト全面更新 Phase 1

## 作業順序

### Step 1: MDX ファイル作成

#### `site/content/docs/syntax/return.mdx`
- `site/content/docs/syntax/` ディレクトリを新規作成
- `return.mdx` を作成
  - frontmatter: `title`, `order: 1`, `category: "Syntax"`, `description`
  - 本文に `return` および `guard` キーワードを含む（テスト assert 条件）

#### `site/content/docs/modules/import.mdx`
- `site/content/docs/modules/` ディレクトリを新規作成
- `import.mdx` を作成
  - frontmatter: `title`, `order: 1`, `category: "Modules"`, `description`
  - 本文に `import` および `W035` キーワードを含む（テスト assert 条件）

### Step 2: `v494000_tests` 追加

`v493000_tests` の直前に挿入（2テスト）:
- `docs_return_syntax_exists` — `syntax/return.mdx` に `"return"` と `"guard"` が含まれることを確認
- `docs_import_v2_exists` — `modules/import.mdx` に `"import"` と `"W035"` が含まれることを確認

`include_str!` パス:
- `"../../site/content/docs/syntax/return.mdx"`
- `"../../site/content/docs/modules/import.mdx"`

### Step 3: `Cargo.toml` version 更新

`"49.3.0"` → `"49.4.0"`

### Step 4: 完了処理

- `cargo test` 3077 passed を確認
- `cargo clippy -- -D warnings` クリーン確認
- `CHANGELOG.md` に v49.4.0 エントリ追加
- `versions/current.md` 更新（v49.4.0・3077 tests・進行中 v49.5.0）
- `versions/roadmap/roadmap-v49.1-v50.0.md` の v49.4.0 実績を記入
- `tasks.md` を COMPLETE に更新

---

## 変更ファイル一覧

| ファイル | 変更種別 |
|---|---|
| `site/content/docs/syntax/return.mdx` | 新規作成 |
| `site/content/docs/modules/import.mdx` | 新規作成 |
| `fav/src/driver.rs` | `v494000_tests` 追加（2テスト）|
| `fav/Cargo.toml` | version 更新 |
| `CHANGELOG.md` | v49.4.0 エントリ |
| `versions/current.md` | バージョン更新 |
| `versions/roadmap/roadmap-v49.1-v50.0.md` | 実績記入 |
| `versions/v45-v50/v49.4.0/tasks.md` | COMPLETE 更新 |

## 変更しないファイル

| ファイル | 理由 |
|---|---|
| `site/content/docs/stdlib/v2.mdx` | v47 シリーズで作成済み・本バージョンのスコープ外 |
| `site/content/docs/migration-guide-import.mdx` | v48.9.0 で作成済み（import 移行ガイド）。`modules/import.mdx` からリンクするため存在確認を T0 で実施 |
| `site/content/docs/module-system.mdx` | v48.9.0 で作成済み（モジ���ールシステム概要）|
