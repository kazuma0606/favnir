# Tasks: v46.9.0 — Developer Experience ドキュメント + v47.0 前調整

Status: COMPLETE
Date: 2026-07-17

---

## T0 — 事前確認

- [x] `cargo test` 3010 passed, 0 failed を確認

## T1 — MDX ドキュメント作成

- [x] `site/content/docs/tools/fav-test.mdx` を新規作成
  - [x] `#[test]` 構文説明
  - [x] `fav test` / `fav test --filter` コマンド説明
  - [x] `assert_eq` / `assert_ne` / `assert_ok` / `assert_err` 一覧
  - [x] 出力フォーマット例
- [x] `site/content/docs/tools/developer-experience.mdx` を新規作成
  - [x] fav test セクション
  - [x] LSP クイックフィックスセクション（`quickFix` / E0102 / E0101）
  - [x] `fav explain --types` セクション
  - [x] `fav explain --lineage --show-dead` セクション
  - [x] v47.0 に向けての予告

## T2 — `driver.rs`: `v469000_tests` 追加

- [x] `v469000_tests` モジュールを `v468000_tests` の直前に追加
  - [x] `fav_test_doc_exists`: fav-test.mdx の存在と内容確認
  - [x] `developer_experience_doc_exists`: developer-experience.mdx の存在と内容確認

## T3 — テスト＆完了

- [x] `cargo test` 3012 passed, 0 failed（3010 + 2 件）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `fav/Cargo.toml` version → `"46.9.0"`
- [x] `CHANGELOG.md` に v46.9.0 エントリ追加
- [x] `versions/current.md` を v46.9.0（3012 tests）に更新（サブスプリント参照も `roadmap-v46.1-v47.0.md` に更新）
- [x] tasks.md を COMPLETE に更新（T0〜T4 全チェック）

## T4 — v47.0 前調整

- [x] v46.1〜v46.9 の全機能が `cargo test` で通過していることを確認（3012 passed, 0 failed）
- [x] `cargo clippy -- -D warnings` クリーン確認（v47.0 コードフリーズ確認）
- [x] v47.0 tasks.md に以下のテスト名を引き継ぎメモ:
  - `cargo_toml_version_is_47_0_0`
  - `changelog_has_v47_0_0`
  - `milestone_has_developer_experience`（MILESTONE.md に `"Developer Experience"` が含まれる）
  - `readme_mentions_developer_experience`
  - `cargo clean` 実施確認（★クリーンアップ）

---

## コードレビュー指摘と対応（spec-reviewer）

| 重大度 | 内容 | 対応 |
|---|---|---|
| [HIGH] | ロードマップのテスト数 3011 が実態と不一致 | `roadmap-v46.1-v47.0.md` の v46.9.0 を 3012 に、v47.0 最低テスト数も ≥ 3012 に修正 |
| [HIGH] | MDX ファイル未作成（実装前提） | plan.md の実装順序が正しく対処済み |
| [MED] | v47.0 引き継ぎ項目が tasks.md に未記載 | tasks.md に T4 セクションを新設し 5 つのチェック項目を追加 |
| [MED] | developer-experience.mdx コードフェンス構造 | plan.md の注意事項に「ネスト確認」を追記 |
| [LOW] | current.md サブスプリント参照が古い | plan.md 注意事項に明記 + T3 で `roadmap-v46.1-v47.0.md` に更新 |
