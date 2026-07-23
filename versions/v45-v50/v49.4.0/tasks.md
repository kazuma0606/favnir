# Tasks: v49.4.0 — ドキュメントサイト全面更新 Phase 1

Status: COMPLETE
Date: 2026-07-18

---

## T0 — 事前確認

- [x] `cargo test` 3075 passed, 0 failed を確認（ベース確認）（ロードマップ推定 3070 との差分 +5 は v49.1〜v49.3 の実績を反映）
- [x] `site/content/docs/syntax/` が存在しないことを確認（新規作成対象）
- [x] `site/content/docs/modules/` が存在しないことを確認（新規作成対象）
- [x] `site/content/docs/migration-guide-import.mdx` が存在することを確認（`modules/import.mdx` からのリンク先）

## T1 — MDX ファイル作成

- [x] `site/content/docs/syntax/return.mdx` 新規作成
  - [x] frontmatter: `title: "Return Statement"` / `order: 1` / `category: "Syntax"` / `description`
  - [x] 本文に `"return"` キーワードを含む（テスト assert 条件）
  - [x] 本文に `"guard"` キーワードを含む（テスト assert 条件）— "Guard Pattern" セクション見出しで対応
  - [x] `return expr if condition` の構文例を含む
- [x] `site/content/docs/modules/import.mdx` 新規作成
  - [x] frontmatter: `title: "Import Syntax 2.0"` / `order: 1` / `category: "Modules"` / `description`
  - [x] 本文に `"import"` キーワードを含む（テスト assert 条件）
  - [x] 本文に `"W035"` キーワードを含む（テスト assert 条件）
  - [x] パッケージ import とローカル import の例を含む

## T2 — `v494000_tests` 追加

- [x] `v494000_tests` モジュールを `v493000_tests` の直前に追加（2テスト）
  - [x] `docs_return_syntax_exists`: `"return"` と `"guard"` が含まれることを確認
  - [x] `docs_import_v2_exists`: `"import"` と `"W035"` が含まれることを確認

## T3 — バージョン更新・完了

- [x] `fav/Cargo.toml` version → `"49.4.0"`
- [x] `cargo test` 3077 passed, 0 failed（3075 + 2 件）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `CHANGELOG.md` に v49.4.0 エントリ追加（`syntax/return.mdx` / `modules/import.mdx` 新規作成を明記）
- [x] `versions/current.md` を v49.4.0（3077 tests）に更新、進行中バージョンを `v49.5.0` に更新
- [x] `versions/roadmap/roadmap-v49.1-v50.0.md` の v49.4.0 実績を 3077 に記入
- [x] tasks.md を COMPLETE に更新（T0〜T3 全 `[x]`）

---

> **注記**: 初回テスト実行で `docs_return_syntax_exists` が FAILED — MDX 本文に英語 "guard" がなく "ガード節" のみだったため。"Guard Pattern" セクション見出しを追加して修正。
> **注記**: `site/content/docs/stdlib/` の更新は本バージョンのスコープ外（v2.mdx は v47 で作成済み）
> **注記**: `cargo clean` はこのバージョンのスコープ外（v50.0.0 で実施）
