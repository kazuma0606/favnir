# Tasks: v48.9.0 — Module ドキュメント + migration guide + v49.0 前調整

Status: COMPLETE
Date: 2026-07-18

---

## T0 — 事前確認

- [x] `cargo test` 3063 passed, 0 failed を確認（ベース確認）
- [x] `site/content/docs/module-system.mdx` が存在しないことを確認
- [x] `site/content/docs/migration-guide-import.mdx` が存在しないことを確認

## T1 — MDX ファイル作成

- [x] `site/content/docs/module-system.mdx` 新規作成
  - [x] フロントマター（title / order / category / description）
  - [x] パッケージ import 構文（`import kafka` / `import kafka as k`）の説明
  - [x] ローカル import 構文（`import "./src/helpers" as helpers`）の説明
  - [x] `fav.toml [runes]` 宣言の説明
  - [x] 循環 import（E0418）の説明
  - [x] 旧構文（W035 非推奨）の説明
- [x] `site/content/docs/migration-guide-import.mdx` 新規作成
  - [x] フロントマター（title / order / category / description）
  - [x] 移行背景・理由の説明
  - [x] 3 ステップの移行手順
  - [x] 旧構文 vs 新構文の変換対応表
  - [x] `import rune` / `migration` / `W035` のキーワードを含む（テスト用）

## T2 — `driver.rs` テスト追加

- [x] `v489000_tests` モジュールを `v488000_tests` の直前に追加（2テスト）
  - [x] `module_system_doc_exists`: `include_str!("../../site/content/docs/module-system.mdx")` → `"Module System"` 必須 + `E0418` or `import` 含有確認
  - [x] `import_migration_guide_exists`: `include_str!("../../site/content/docs/migration-guide-import.mdx")` → `import rune` / `migration` / `W035` 含有確認

## T3 — バージョン更新・完了

- [x] `fav/Cargo.toml` version → `"48.9.0"`
- [x] `CHANGELOG.md` に v48.9.0 エントリ追加
- [x] `cargo test` 3065 passed, 0 failed（3063 + 2 件）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `versions/current.md` を v48.9.0（3065 tests）に更新、進行中バージョンを `v49.0.0` に更新
- [x] `versions/roadmap/roadmap-v48.1-v49.0.md` の v48.9.0 実績を 3065 に記入
- [x] tasks.md を COMPLETE に更新（T0〜T3 全 `[x]`）

---

> **注記**: コードフリーズ — v49.0.0 に向けて本バージョン以降はコード追加なし（MDX + テストのみ）
> **注記**: `cargo clean` は v49.0.0 で実施（本バージョンはスコープ外）
