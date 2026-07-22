# Tasks: v54.7.0 — ドキュメントサイト Production 3.0 overview ページ

Status: COMPLETE
Date: 2026-07-23

---

## T0 — 事前確認

- [x] `cargo test` 3197 passed, 0 failed を確認（ベース確認）
- [x] `cargo clippy -- -D warnings` クリーンであることを確認
- [x] `driver.rs` に `v54700_tests` が**存在しない**ことを確認
- [x] `driver.rs` に `v54600_tests` が存在することを確認（挿入位置の確認）
- [x] `site/content/docs/production3-overview.mdx` が**存在しない**ことを確認
- [x] `Cargo.toml` の現在バージョンが `54.6.0` であることを確認

---

## T1 — `site/content/docs/production3-overview.mdx` 新規作成

- [x] `# Production 3.0 — Favnir v55 への道のり` タイトル
- [x] `## v51 — Developer Experience 3.0` セクション:
  - [x] 全エラーコード `fav explain --error` 対応完備を記載
  - [x] LSP インレイヒントを記載
  - [x] `fav run --trace` を記載
  - [x] `fav run --watch` 自動再実行を記載
  - [x] `--watch-diff` / `--watch-summary` を**含めない**（v54.2.0 の成果物のため）
- [x] `## v52 — Performance & Scale` セクション:
  - [x] `par` 並列 stage 実行・バックプレッシャー・`fav bench --compare` を記載
- [x] `## v53 — Data Quality & Observability 2.0` セクション:
  - [x] `assert_schema` / `--lineage --with-schema` / `--audit-log` を記載
- [x] `## v54 — Integration Sprint` セクション:
  - [x] v54.1〜v54.5 の各サブバージョン機能を列挙
  - [x] `--watch-diff` / `--watch-summary` を v54 セクションに正しく記載
- [x] `## v55 — Production 3.0 宣言` セクション:
  - [x] `"v55"` を含む
  - [x] Production 3.0 宣言文を記載
- [x] `## 関連ドキュメント` セクション:
  - [x] `dx3-overview.mdx` / `integration-overview.mdx` / `data-quality-overview.mdx` リンク
  - [x] `MILESTONE.md` リンクを**含めない**（サイト構造から到達不可）

---

## T2 — `driver.rs` — `v54700_tests` 追加

- [x] `v54600_tests` の直前に `v54700_tests` を追加（2 テスト）:
  - [x] `use super::*` を追加（実質不要だが他テストモジュールとの慣習統一のため）
  - [x] `docs_production3_overview_exists`:
    - [x] `include_str!("../../site/content/docs/production3-overview.mdx")` が非空
    - [x] `"Production 3.0"` を含む
  - [x] `docs_production3_has_v55`:
    - [x] `"v55"` を含む
- [x] `cargo build` → コンパイルエラーなし確認（`include_str!` パス検証）

---

## T3 — `fav/Cargo.toml` 更新 + テスト実行

- [x] `version = "54.6.0"` → `version = "54.7.0"` に変更
- [x] `cargo test -j 8 -- --test-threads=8` 実行 → 3199 passed, 0 failed を確認
- [x] `cargo clippy -- -D warnings` クリーンを確認

---

## T4 — 後処理

- [x] `CHANGELOG.md`: v54.7.0 エントリ追加（v54.6.0 の直上）
- [x] `versions/current.md` を v54.7.0（3199 tests）に更新
- [x] `roadmap-v54.1-v55.0.md` の v54.7.0 実績欄を更新（COMPLETE・3199 tests・2026-07-23）

---

## T5 — コードレビュー対応

- [x] [MED] `--watch-diff` / `--watch-summary` が v51 セクションに誤帰属 → v51 セクションから削除
- [x] [MED] テストのアサーションが浅い → v54.9.0 で `production3_overview_doc_complete` 拡充予定（現バージョン対応不要）
- [x] [LOW] `MILESTONE.md` への相対パスがサイト構造から到達不可 → リンクを削除
- [x] [LOW] `.mdx` 拡張子リンク形式 → 既存 MDX と同形式のため対応不要
- [x] [LOW] `par` の説明が既存実装との区別が曖昧 → overview ページとして許容範囲のため対応不要

---

## T6 — tasks.md 完了

- [x] tasks.md を COMPLETE に更新（T0〜T6 全 `[x]`）
