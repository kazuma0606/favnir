# Tasks: v49.8.0 — ドキュメントサイト全面更新 Phase 2 + CHANGELOG 整理

Status: COMPLETE
Date: 2026-07-18

---

## T0 — 事前確認

- [x] `cargo test` 3083 passed, 0 failed を確認（ベース確認）
- [x] `site/content/docs/language-maturity-overview.mdx` が存在しないことを確認
- [x] `MILESTONE.md` が `favnir/` 直下に存在することを確認
- [x] `MILESTONE.md` に `"Language Maturity"` が含まれ��いないことを確認（重複追記防止）
- [x] `v497000_tests` モジュールが `driver.rs` に存在することを確認（挿入位置の前提）

## T1 — ドキュメント作成

- [x] `site/content/docs/language-maturity-overview.mdx` 新規作成
  - [x] `title: "Language Maturity Overview"` を含む
  - [x] `"Language Maturity"` という文字列を含む
  - [x] `"v50"` という文字列を含む
  - [x] v50 の4本柱（return / stdlib / modules / inline tests）を記述
- [x] `MILESTONE.md` に `"Language Maturity"` エントリを追加

## T2 — `v498000_tests` 追加

- [x] `v498000_tests` モジュールを `v497000_tests` の直前に追加（2 テスト）
- [x] 挿入後 `grep -n v498000_tests src/driver.rs` で存在確認
  - [x] `docs_site_v50_overview_exists`:
    - [x] `include_str!("../../site/content/docs/language-maturity-overview.mdx")` でファイルを読み込む
    - [x] `"Language Maturity"` が含まれることを確認
    - [x] `"v50"` が含まれることを確認
  - [x] `milestone_has_language_maturity`:
    - [x] `include_str!("../../MILESTONE.md")` でファイルを読み込む
    - [x] `"Language Maturity"` が含まれることを確認

## T3 — バージョン更新・完了

- [x] `fav/Cargo.toml` version → `"49.8.0"`
- [x] `cargo test` 3085 passed, 0 failed
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `CHANGELOG.md` に v49.8.0 エントリ追加（ドキュメント Phase 2 + MILESTONE を明記）
- [x] `versions/current.md` を v49.8.0（3085 tests）に更新、進行中バージョンを `v49.9.0` に更新
- [x] `versions/roadmap/roadmap-v49.1-v50.0.md` の v49.8.0 実績を 3085 に記入
- [x] tasks.md を COMPLETE に更新（T0〜T3 全 `[x]`）
