# Tasks: v49.5.0 — cookbook 更新

Status: COMPLETE
Date: 2026-07-18

---

## T0 — 事前確認

- [x] `cargo test` 3077 passed, 0 failed を確認（ベース確認）（ロードマップ推定 3072 との差分 +5 は v49.1〜v49.4 の実績を反映）
- [x] `fav/src/driver.rs` に `v494000_tests` モジュールが存在することを確認（挿入位置の前提）
- [x] `site/content/cookbook/return-guard-pattern.mdx` が存在しないことを確認（新規作成対象）
- [x] `site/content/cookbook/inline-testing.mdx` が存在しないことを確認（新規作成対象）
- [x] `site/content/cookbook/modular-pipelines.mdx` が存在しないことを確認（新規作成対象）

## T1 — cookbook MDX ファイル作成

- [x] `site/content/cookbook/return-guard-pattern.mdx` 新規作成
  - [x] frontmatter: `title: "Return Guard Pattern"` / `category: "クックブック"` / `description`
  - [x] 本文に英語 `"return Result"` キーワードを含む（テスト assert 条件）
  - [x] 本文に英語 `"guard"` キーワードを含む（テスト assert 条件）
  - [x] `return expr if condition` のコード例を含む
- [x] `site/content/cookbook/inline-testing.mdx` 新規作成
  - [x] frontmatter: `title: "Inline Testing"` / `category: "クックブック"` / `description`
  - [x] 本文に `"#[test]"` を含む（テスト assert 条件）
  - [x] 本文に `"assert"` を含む（テスト assert 条件）
  - [x] `fav test` 実行例を含む
- [x] `site/content/cookbook/modular-pipelines.mdx` 新規作成（テスト対象外）
  - [x] frontmatter: `title: "Modular Pipelines"` / `category: "クックブック"` / `description`
  - [x] 新 import 構文のコード例を含む

## T2 — `v495000_tests` 追加

- [x] `v495000_tests` モジュールを `v494000_tests` の直前に追加（2テスト）
  - [x] `cookbook_return_guard_exists`: `"return Result"` と `"guard"` が含まれることを確認
  - [x] `cookbook_fav_test_exists`: `"#[test]"` と `"assert"` が含まれることを確認

## T3 — バージョン更新・完了

- [x] `fav/Cargo.toml` version → `"49.5.0"`
- [x] `cargo test` 3079 passed, 0 failed（3077 + 2 件）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `CHANGELOG.md` に v49.5.0 エントリ追加（3 cookbook ファイル新規作成を明記）
- [x] `versions/current.md` を v49.5.0（3079 tests）に更新、進行中バージョンを `v49.6.0` に更新
- [x] `versions/roadmap/roadmap-v49.1-v50.0.md` の v49.5.0 実績を 3079 に記入
- [x] tasks.md を COMPLETE に更新（T0〜T3 全 `[x]`）

---

> **注記**: `cookbook_fav_test_exists` は `"#[test]"` アノテーション文字列でアサート（`"fav test"` コマンド名より構文的に一意）
> **注記**: `cargo clean` はこのバージョンのスコープ外（v50.0.0 で実施）
