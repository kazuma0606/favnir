# Spec: v46.9.0 — Developer Experience ドキュメント + v47.0 前調整

## 概要

v46.1〜v46.8 で実装した Developer Experience 機能のドキュメントを
`site/content/docs/` に追加し、v47.0 コードフリーズに備える。

---

## 問題

以下の機能が実装済みだがドキュメントが存在しない:
- `fav test`（`#[test]` インラインテスト、v46.1〜v46.3）
- LSP クイックフィックス（E0102 did-you-mean / E0101 引数追加提案、v46.5）
- `fav explain` 2.0（dead path 可視化・`--lineage --show-dead`・`--types`、v46.6〜v46.8）

---

## 解決策

以下 2 つの MDX ドキュメントを新規作成する:

### 1. `site/content/docs/tools/fav-test.mdx`

`fav test` コマンドのリファレンスドキュメント。

内容:
- `#[test]` アノテーション構文
- `fav test <file.fav>` / `fav test --filter <name>` の使い方
- `assert_eq` / `assert_ok` / `assert_err` / `assert_ne` の説明と例
- pass/fail 出力フォーマット

### 2. `site/content/docs/tools/developer-experience.mdx`

Developer Experience 機能全体の概要ドキュメント。

内容:
- v46.x DX 機能一覧（fav test / LSP quickfix / fav explain 2.0）
- LSP クイックフィックス（E0102 did-you-mean / E0101 引数追加）の説明
- `fav explain --types` / `--lineage --show-dead` / `--format mermaid`（dead path）の説明
- 今後の予定（v47.0 Developer Experience 宣言）

---

## テスト（+2）

| テスト名 | 内容 |
|---|---|
| `fav_test_doc_exists` | `site/content/docs/tools/fav-test.mdx` が存在し `#[test]`・`assert_eq`・`fav test` が含まれる |
| `developer_experience_doc_exists` | `site/content/docs/tools/developer-experience.mdx` が存在し `fav explain --types`・`--show-dead`・`quickFix` が含まれる |

---

## 完了条件

- `cargo test` 3012 passed, 0 failed（3010 + 2 件）
- `cargo clippy -- -D warnings` クリーン
- `fav/Cargo.toml` version → `"46.9.0"`
- `CHANGELOG.md` に v46.9.0 エントリ追加
- `versions/current.md` を v46.9.0（3012 tests）に更新
- `tasks.md` を COMPLETE に更新

---

## v47.0 前調整

本バージョンはコードフリーズ（新機能追加なし）。
v46.1〜v46.8 の全機能が実装完了していることを確認し、
v47.0 マイルストーン宣言の準備とする。

- `cargo test` 全通過（failures=0）確認
- `cargo clippy -- -D warnings` クリーン確認
- v47.0 テスト名（`cargo_toml_version_is_47_0_0` 等）を v47.0 tasks.md に引き継ぐ
