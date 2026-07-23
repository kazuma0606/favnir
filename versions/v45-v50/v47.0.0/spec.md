# Spec: v47.0.0 — Developer Experience 宣言 ★クリーンアップ

## 概要

v46.1〜v46.9 で実装した Developer Experience 機能群を総括し、
「Developer Experience」マイルストーンを宣言する。

---

## 宣言文

> 「インラインテスト・LSP クイックフィックス・型情報可視化が揃い、
>  Favnir の開発体験が実用水準に達した。
>
>  これが Favnir v47.0 — Developer Experience の姿である。」

---

## 実装スコープ

新機能追加なし（コードフリーズ）。以下のドキュメント・宣言作業のみ:

| 作業 | 内容 |
|---|---|
| `MILESTONE.md` 更新 | v47.0.0 — Developer Experience エントリを追加 |
| `README.md` 更新 | `"Developer Experience"` への言及を追加 |
| `Cargo.toml` version bump | `46.9.0` → `47.0.0` |
| `CHANGELOG.md` エントリ追加 | v47.0.0 マイルストーン宣言エントリ |
| `v47000_tests` 追加 | 4 件の宣言確認テスト |
| `cargo clean` ★クリーンアップ | ビルド生成物のクリア |

---

## テスト（+4）

| テスト名 | 内容 |
|---|---|
| `cargo_toml_version_is_47_0_0` | `Cargo.toml` に `version = "47.0.0"` が含まれる |
| `changelog_has_v47_0_0` | `CHANGELOG.md` に `[v47.0.0]` が含まれる |
| `milestone_has_developer_experience` | `MILESTONE.md` に `"Developer Experience"` が含まれる |
| `readme_mentions_developer_experience` | `README.md` に `"Developer Experience"` が含まれる |

---

## 達成コンポーネント一覧（v46.1〜v46.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| `#[test]` ブロック AST + parser | v46.1 | `FnDef.is_test = true`、`#[test] fn` 解析 |
| `fav test` コマンド実装 | v46.2 | `cmd_test`、`#[test]` 収集と VM 実行ループ |
| assertion 拡充 | v46.3 | `assert_ok` / `assert_err` / `assert_ne` VM primitive |
| LSP inlay hints 強化 | v46.4 | `textDocument/inlayHint`、パイプライン推論型表示 |
| LSP クイックフィックス強化 | v46.5 | E0102 did-you-mean / E0101 引数追加提案 |
| `fav explain` 2.0 Phase 1 | v46.6 | dead path（点線）/ error path（赤）Mermaid 可視化 |
| `fav explain --lineage` 2.0 | v46.7 | `is_dead` フラグ + `--show-dead` CLI |
| `fav explain --types` | v46.8 | ステージ宣言型一覧表示 |
| DX ドキュメント + v47.0 前調整 | v46.9 | `fav-test.mdx` / `developer-experience.mdx` |

---

## 完了条件

- `cargo test` ≥ 3016 passed, 0 failed（3012 + 4 件）
- `cargo clippy -- -D warnings` クリーン
- `fav/Cargo.toml` version → `"47.0.0"`
- `CHANGELOG.md` に v47.0.0 エントリ追加
- `MILESTONE.md` に v47.0.0 Developer Experience エントリ追加
- `README.md` に `"Developer Experience"` 追加
- `versions/current.md` を v47.0.0 に更新
- `★クリーンアップ`（`cargo clean`）完了
- `tasks.md` を COMPLETE に更新
