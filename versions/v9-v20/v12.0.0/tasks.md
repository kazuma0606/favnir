# Favnir v12.0.0 Tasks

Date: 2026-06-06
Theme: Python トランスパイラ完成宣言

---

## Phase A — CHANGELOG.md 更新

- [x] A-1: v11.1.0〜v11.9.0 の各エントリを CHANGELOG.md に追記
- [x] A-2: v12.0.0 エントリを CHANGELOG.md に追記（最上部）

---

## Phase B — README.md 更新

- [x] B-1: README.md「主要機能」テーブルに `fav transpile --target python` を追記

---

## Phase C — site/content/docs/transpile/python.mdx

- [x] C-1: `site/content/docs/transpile/` ディレクトリ作成
- [x] C-2: `site/content/docs/transpile/python.mdx` 新規作成
  - 概要・インストール・基本的な使い方
  - `--out-dir` / `--check` / `--run` オプション
  - エフェクト → Python ライブラリ対応表
  - `!Postgres` → psycopg2 変換例
  - `!AWS` → boto3 変換例
  - lineage コメント（`--lineage`）
  - fav2py E2E デモへのリンク

---

## Phase D — Rust テスト（2 件）

- [x] D-1: `driver.rs` に `v12000_tests` モジュール追加
  - [x] `version_is_12_0_0` — `CARGO_PKG_VERSION == "12.0.0"`
  - [x] `python_mdx_doc_exists` — `site/content/docs/transpile/python.mdx` の存在確認
- [x] D-2: `cargo test v12000` — 2 件通過
- [x] D-3: `cargo test --lib` — 705 件通過

---

## Phase E — バージョン更新 + コミット

- [x] E-1: `fav/Cargo.toml` version → `"12.0.0"`
- [x] E-2: `cargo build` で `Cargo.lock` 更新
- [x] E-3: `git commit & push` — CI 確認

---

## 完了条件サマリー

| 確認項目 | 状態 |
|---|---|
| CHANGELOG.md に v11.1.0〜v12.0.0 全エントリ記載 | |
| README.md に Python トランスパイラ追記 | |
| `site/content/docs/transpile/python.mdx` 作成 | |
| `cargo test v12000` 2 件通過 | |
| `cargo test --lib` 1290 件以上通過 | |
