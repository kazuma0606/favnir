# v35.7.0 spec — docs_server.rs !Effect 完全除去

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v35.7.0 |
| コードネーム | v35.0B |
| テーマ | `docs_server.rs` から `!Effect` 表記を完全除去し、!Effect 廃止を言語・ドキュメント・ランタイム全レイヤーで完結させる |
| 前提 | v35.6.0（v35.0A）COMPLETE — サイト MDX 128 ファイル・ctx 構文統一・Production Ready 宣言済み |
| 完了条件 | `v35700_tests` 全テスト pass・`cargo test` 0 failures |

## 背景と目的

v35.5.0〜v35.6.0 で `!Effect` 廃止作業の大部分を完了した：

- `ast.rs` の `Effect` enum・`effects` フィールド削除
- `checker.rs` / `compiler.rs` / `lineage.rs` / `ir.rs` / `reachability.rs` の Effect 参照削除
- `lint.rs` の W007/W021 no-op 化
- `site/content/` 128 MDX ファイルからの `!Effect` アノテーション除去
- `README.md` / `MILESTONE.md` 更新

残存する唯一の `!Effect` 参照は `fav/src/docs_server.rs` の stdlib 関数表示文字列（`IO_FUNCTIONS` 定数内 3 関数エントリ）であった。

本バージョンでは `docs_server.rs` の残存箇所を除去し、Favnir コードベース全体から `!Effect` 表記を**完全に排除**する。

## ロードマップとの差異

`roadmap-v35.1-v36.0.md` では v35.7.0 を `fav deploy --dry-run`（デプロイ前差分表示）と計画していたが、`!Effect` 廃止完結シリーズ（v35.0B）を優先したため本バージョンで実施する。

`fav deploy --dry-run` は後続バージョンで対応する。

## 実装スコープ

### sprint（v35.0B）で完了済み

| ファイル | 変更内容 |
|---|---|
| `fav/src/docs_server.rs` | IO_FUNCTIONS 3 関数の `signature` から `!Io` 除去、`effects` を `&[]` に統一（計 6 行） |
| `CHANGELOG.md` | `## [v35.7.0]` エントリ追加済み |
| `fav/src/driver.rs` | `v35700_tests` モジュール（5 件）pre-existing |

### 本セッションで実施

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `v35600_tests::cargo_toml_version_is_35_6_0` をスタブ化（バンプ前に必須） |
| `fav/src/driver.rs` | `v35700_tests::cargo_toml_version_is_35_7_0` 半スタブ → 生きたアサーションに修正 |
| `fav/Cargo.toml` | バージョン `35.6.0` → `35.7.0` |

## docs_server.rs 変更詳細（sprint 完了済み）

| 関数 | 変更前 | 変更後 |
|---|---|---|
| `IO.println` — signature | `"String -> Unit !Io"` | `"String -> Unit"` |
| `IO.println` — effects | `&["Io"]` | `&[]` |
| `IO.print` — signature | `"String -> Unit !Io"` | `"String -> Unit"` |
| `IO.print` — effects | `&["Io"]` | `&[]` |
| `IO.read_line` — signature | `"() -> String !Io"` | `"() -> String"` |
| `IO.read_line` — effects | `&["Io"]` | `&[]` |

`StdlibFunction` 構造体の `effects` フィールド自体は後方互換のため残存。

**`build_stdlib_json` の公開レベル**: `pub fn`（`docs_server.rs` line 701）。`v35700_tests::docs_server_io_effects_empty` は `crate::docs_server::build_stdlib_json()` を直接呼び出すため、`pub fn` であることが必須。sprint 完了済みで確認済み。

## v35700_tests の内容（pre-existing）

| テスト名 | 検証内容 |
|---|---|
| `cargo_toml_version_is_35_7_0` | Cargo.toml に `"35.7.0"` が含まれる（半スタブ → 修正対象） |
| `docs_server_io_signatures_no_effect` | `docs_server.rs` の signature 行に `!Io` / `!Http` / `!Aws` が含まれない |
| `docs_server_io_effects_empty` | `build_stdlib_json()` の IO モジュール全関数の effects が空配列 |
| `changelog_has_v35_7_0` | `CHANGELOG.md` に `[v35.7.0]` が含まれる |
| `effect_annotation_fully_purged` | `docs_server.rs` ソース全体に `!Io"` / `!Http"` / `!Aws"` の文字列リテラルが存在しない |

## 設計決定

- **`cargo_toml_version_is_35_7_0` の扱い**: 現在 `assert!(cargo.contains("35."), ...)` という弱い半スタブ状態。`v35600_tests::cargo_toml_version_is_35_6_0` をスタブ化した後、`assert!(cargo.contains("35.7.0"), ...)` の生きたアサーションへ修正し、Cargo.toml を 35.7.0 に bump する（v35.8.0 bump 時にスタブ化）
- **`docs_server.rs` 修正の事後確認**: sprint 中に実施済みのため、本セッションでの追加コード変更なし

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `docs_server.rs` に `!Io"` / `!Http"` / `!Aws"` の文字列リテラルが存在しない | `docs_server_io_signatures_no_effect` + `effect_annotation_fully_purged` テスト |
| 2 | `build_stdlib_json()` の IO 関数 effects が空配列 | `docs_server_io_effects_empty` テスト |
| 3 | `CHANGELOG.md` に `[v35.7.0]` が含まれる | `changelog_has_v35_7_0` テスト |
| 4 | `Cargo.toml` バージョンが `35.7.0` | `cargo_toml_version_is_35_7_0` テスト |
| 5 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2646、今回追加テストなし・前バージョンと同数維持） | `cargo test` 実行結果 |
