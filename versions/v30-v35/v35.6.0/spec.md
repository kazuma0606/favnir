# v35.6.0 spec — ctx 構文統一 + Production Ready 宣言

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v35.6.0 |
| コードネーム | v35.0A |
| テーマ | `!Effect` 廃止完結に伴うドキュメントの ctx 構文統一 + v35.0 Production Ready 宣言 |
| 前提 | v35.5.0 COMPLETE（Effect enum・effects フィールド・parse_effects_acc 完全削除） |
| 完了条件 | `v35600_tests` 全テスト pass・`cargo test` 0 failures |

## 背景と目的

v35.5.0 で `Effect` enum および関連 AST フィールドを完全削除し、`!Effect` 廃止が言語レベルで完結した。

本バージョンでは、ドキュメント・サイト・README を「ctx: AppCtx による Capability Context」パターンに統一し、v35.0 Production Ready マイルストーンを正式宣言する。

具体的には:
1. **ctx-syntax-guide.mdx 整備**: E0374 エラーと `ctx: AppCtx` パターンを文書化した公式ガイドページ
2. **README.md 更新**: ctx: AppCtx パターンを README に記載
3. **MILESTONE.md 更新**: v35.0 Production Ready 宣言を追記
4. **MDX ドキュメント統一**: `site/content/` 全 MDX の fenced code block から `!Effect` を除去し ctx 構文に統一（スプリント中に実施済み）

## ロードマップとの差異

`versions/roadmap/roadmap-v35.1-v36.0.md` では v35.6.0 を
`fav deploy status` + `fav rollback` と計画していたが、
`!Effect` 廃止完結ドキュメント作業を v35.0A として優先したため本バージョンで実施する。

`fav deploy status` + `fav rollback` は後続バージョンで対応する（`roadmap-v35.1-v36.0.md` の差し替えスロット要更新）。

## 実装スコープ

### 対象ファイル

| ファイル | 変更内容 |
|---|---|
| `site/content/docs/ctx-syntax-guide.mdx` | E0374・`ctx: AppCtx` パターンを記載した公式ガイド（スプリント中に作成済み — 本バージョンで確認） |
| `README.md` | `ctx: AppCtx` または `AppCtx` パターンを記載（スプリント中に更新済み — 本バージョンで確認） |
| `MILESTONE.md` | Production Ready 宣言を追記（スプリント中に追記済み — 本バージョンで確認） |
| `CHANGELOG.md` | `## [v35.6.0]` エントリ追加（既存） |
| `fav/src/driver.rs` | `v35600_tests` モジュール（5 件）を正式化・`cargo_toml_version_is_35_6_0` を生きたアサーションに修正 |
| `fav/Cargo.toml` | バージョン `35.5.0` → `35.6.0` |

### 対象外（スコープ外）

- `fav deploy status` / `fav rollback` コマンド（ロードマップ計画、後続対応）
- `site/content/` 全 MDX の詳細な `!Effect` 除去作業（スプリント中に実施済み）

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `MILESTONE.md` に `"Production Ready"` が含まれる | `milestone_has_production_ready` テスト |
| 2 | `ctx-syntax-guide.mdx` に `"E0374"` と `"ctx: AppCtx"` が含まれる | `ctx_syntax_guide_has_e0374_section` テスト |
| 3 | `README.md` に `"AppCtx"` が含まれる | `readme_ctx_syntax_documented` テスト |
| 4 | `CHANGELOG.md` に `"[v35.6.0]"` が含まれる | `changelog_has_v35_6_0` テスト |
| 5 | `site/content/` MDX の fenced code block に `!Effect` が残存しない | T0 で grep 確認（スプリント中実施済みの事後確認） |
| 6 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 前バージョン実測値） | `cargo test` 実行 + テスト数確認 |

## 設計決定

- **`cargo_toml_version_is_35_6_0` の扱い**: 現在 `assert!(cargo.contains("35."), ...)` という弱い半スタブ状態。Cargo.toml を 35.6.0 に bump する際に `assert!(cargo.contains("35.6.0"), ...)` の生きたアサーションへ修正する（v35.7.0 bump 時にスタブ化）
- **MDX 大量更新**: スプリント中に一括実施済みのため、本バージョンでの追加作業なし
- **Production Ready 宣言**: MILESTONE.md への追記のみ（専用 MDX ページは不要）
