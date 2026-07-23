# Spec: v49.8.0 — ドキュメントサイト全面更新 Phase 2 + CHANGELOG 整理

Date: 2026-07-18
Status: Draft

---

## 概要

v49 系全体の CHANGELOG / MILESTONE 整理を行い、v50.0 宣言に向けたドキュメントの骨子を作成する。
`site/content/docs/language-maturity-overview.mdx` を新規作成し、v50.0 の言語成熟度を概説する。
MILESTONE.md に Language Maturity マイルストーン記述を追加する。

---

## 背景

v49.1〜v49.7 で全機能の統合・安定化・セキュリティ審査が完了した。
v50.0 Production 2.0 宣言には `milestone_has_language_maturity` / `readme_mentions_language_maturity` 等のテストが必要であり、
v49.8.0 でその骨子を作成しておくことで v49.9〜v50.0 の作業を円滑にする。

---

## 仕様

### `site/content/docs/language-maturity-overview.mdx`

新規作成するドキュメントページ。以下を含む:

- `title: "Language Maturity Overview"`
- `category: "Docs"`
- Favnir v50 の成熟度概要（return ガード節 / 成熟した stdlib / 明確なモジュールシステム / インラインテスト）
- 文字列 `"Language Maturity"` を含む（`milestone_has_language_maturity` テスト用）
- 文字列 `"v50"` を含む

### `MILESTONE.md`

既存ファイルに Language Maturity マイルストーン記述を追加。
`"Language Maturity"` という文字列を含むエントリを追記する。

---

## テスト

`v498000_tests` モジュールに 2 件追加（`v497000_tests` の直前）:

1. `docs_site_v50_overview_exists`
   - `include_str!("../../site/content/docs/language-maturity-overview.mdx")` を読み込み
   - `"Language Maturity"` と `"v50"` が含まれることを確認

2. `milestone_has_language_maturity`
   - `include_str!("../../MILESTONE.md")` を読み込み
   - `"Language Maturity"` が含まれることを確認

---

## site/ 全ページ最終チェック

ロードマップ記載の「`site/content/docs/` 全ページの最終チェック」��、特定の成果物を生まない確認作業。
明白なリンク切れ・frontmatter 欠落がないことを目視確認し、問題があれば当バージョンで修正する。

---

## 完了条件

- `cargo test` 3085 tests passed, 0 failed（3083 + 2 件）
- `cargo clippy -- -D warnings` クリーン
- `CHANGELOG.md` に v49.8.0 エントリ追加
- `versions/current.md` を v49.8.0 に更新
- `versions/roadmap/roadmap-v49.1-v50.0.md` の v49.8.0 実績を記入
