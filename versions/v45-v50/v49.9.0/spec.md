# Spec: v49.9.0 — v50.0 前調整・安定化

Date: 2026-07-18
Status: Draft

---

## 概要

v50.0 Production 2.0 宣言前の最終安定化バージョン。コードフリーズとして新機能追加は行わない。
`language-maturity-overview.mdx` を完成版に仕上げ、全 lint / clippy クリーンを確認する。
`cargo test` 全通過を確認して v50.0 へ向けた準備を完了する。

---

## 背景

v49.8.0 で `language-maturity-overview.mdx` の骨子は作成済み。
v49.9.0 ではこのファイルをより充実した完成版に仕上げる（"完成させる"）。
テスト `cargo_toml_version_is_49_9_0` は Cargo.toml のバージョン文字列を include_str! で読み込んで確認する。
テスト `language_maturity_overview_doc_exists` は v49.8.0 の `docs_site_v50_overview_exists` と重複しないよう、
より詳細な内容チェック（v50.0 宣言文の存在確認など）を行う。

---

## 仕様

### `language-maturity-overview.mdx` 充実化

v49.8.0 で作成した骨子に以下を追加する:

- `"Production 2.0"` の宣言文（既存）
- 各バージョン（v46〜v49）の機能一覧テーブルまたはリスト
- `"v49"` または `"v46"` が含まれる（バージョン履歴の記述）

### テスト仕様

1. `cargo_toml_version_is_49_9_0`
   - `include_str!("../Cargo.toml")` で Cargo.toml を読み込み
   - `"49.9.0"` が含まれることを確認
   - 注: `src/driver.rs` からの相対パスは `../Cargo.toml` = `fav/Cargo.toml`

2. `language_maturity_overview_doc_exists`
   - `include_str!("../../site/content/docs/language-maturity-overview.mdx")` を読み込み
   - `"| v49 |"` が含まれることを確認（T1 で追加する機能一覧テーブルの行）
   - `"| v46 |"` が含まれることを確認（テーブル形式が正しく追加されたことを確認）
   - 注: `"Language Maturity"` / `"v50"` / `"Production 2.0"` は v49.8.0 の `docs_site_v50_overview_exists` でカバー済みのため重複させない

---

## テスト

`v499000_tests` モジュールに 2 件追加（`v498000_tests` の直前）:

1. `cargo_toml_version_is_49_9_0`
2. `language_maturity_overview_doc_exists`

---

## 完了条件

- `cargo test` 3087 tests passed, 0 failed（3085 + 2 件）
- `cargo clippy -- -D warnings` クリーン
- `cargo fmt -- --check` クリーン（Rust ソース対象）
- `CHANGELOG.md` に v49.9.0 エントリ追加
- `versions/current.md` を v49.9.0 に更新
- `versions/roadmap/roadmap-v49.1-v50.0.md` の v49.9.0 実績を記入
