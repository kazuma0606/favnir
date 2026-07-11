# v35.8.0 spec — !Effect 廃止完結（LSP / エラーカタログ / MCP / help テキスト）

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v35.8.0 |
| コードネーム | v35.0C |
| テーマ | LSP 補完・エラーカタログ・MCP ドキュメント・help テキストから残存 `!Effect` を完全除去し、廃止を完結させる |
| 前提 | v35.7.0（v35.0B）COMPLETE — `docs_server.rs` から `!Effect` 完全除去済み |
| 完了条件 | `v35800_tests` 全テスト pass・`cargo test` 0 failures |

## 背景と目的

v35.7.0 でコードベース・サイト・ドキュメントの主要な `!Effect` 参照を除去したが、以下の 4 ファイルに残存が確認された。いずれもユーザーが直接目にする箇所（IDE 補完・エラーメッセージ・CLI help・MCP ドキュメント）である。

本バージョンではこれらを除去し、Favnir 全体から `!Effect` 表記を完全に排除する。

## ロードマップとの差異

`roadmap-v35.1-v36.0.md` では v35.8.0 を「デプロイ cookbook + ドキュメント」と計画していたが、`!Effect` 廃止完結シリーズ（v35.0C）を優先したため本バージョンで実施する。

デプロイ cookbook は後続バージョンで対応する。

## 実装スコープ

### sprint（v35.0C）で完了済み

| ファイル | 変更内容 |
|---|---|
| `fav/src/lsp/completion.rs` | IO/Http/Llm/Db/Gen/Csv/Sys 計 ~25 関数のシグネチャから `!Effect` 除去、KNOWN_RUNES 説明文 12 件から `!Effect` タグ除去 |
| `fav/src/error_catalog.rs` | E0310〜E0319 の `fix:` フィールド 7 件を `ctx: AppCtx` ベースの修正提案に書き換え |
| `fav/src/mcp/mod.rs` | db/http/log rune ドキュメント文字列（計 ~13 行）から `!Io` 除去 |
| `fav/src/main.rs` | help テキスト `!DbRead/!DbWrite effects` → `DbRead/DbWrite lineage tags` に修正 |
| `CHANGELOG.md` | `## [v35.8.0]` エントリ追加済み |
| `fav/src/driver.rs` | `v35800_tests` モジュール（5 件）pre-existing |

### 本セッションで実施

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `v35700_tests::cargo_toml_version_is_35_7_0` をスタブ化（バンプ前に必須） |
| `fav/src/driver.rs` | `v35800_tests::cargo_toml_version_is_35_8_0` 現スタブ → 生きたアサーションに修正 |
| `fav/Cargo.toml` | バージョン `35.7.0` → `35.8.0` |

## v35800_tests の内容（pre-existing）

| テスト名 | 検証内容 |
|---|---|
| `cargo_toml_version_is_35_8_0` | Cargo.toml に `"35.8.0"` が含まれる（現スタブ → 修正対象） |
| `lsp_completion_signatures_no_effect` | `lsp/completion.rs` の signature/description 行に `!Io"` / `!Http"` / `!Db"` / `!Llm"` / `!Gen"` / `!Auth"` / `!AWS"` / `!Cache"` / `!Queue"` が含まれない |
| `error_catalog_fix_no_effect_syntax` | `error_catalog.rs` の `fix:` フィールドに `!Db` / `!Auth` / `!Env` / `!AWS` / `!Snowflake` / `!Postgres` / `!Stream` / `!Redis` / `!MySQL` / `!MongoDB` が含まれない |
| `mcp_docs_no_effect_annotation` | `mcp/mod.rs` のドキュメント文字列に `!Io\n` / `!Http\n` / `!Db\n` が含まれない |
| `changelog_has_v35_8_0` | `CHANGELOG.md` に `[v35.8.0]` が含まれる |

## 設計決定

- **`cargo_toml_version_is_35_8_0` の扱い**: 現在 `// stubbed: version bumped to 35.7.0`（空ボディ）のスタブ状態。`v35700_tests::cargo_toml_version_is_35_7_0` をスタブ化した後、`assert!(cargo.contains("35.8.0"), ...)` の生きたアサーションへ修正し、Cargo.toml を 35.8.0 に bump する（v35.9.0 bump 時にスタブ化）
- **lsp/completion.rs などの修正**: sprint 中に一括実施済みのため、本セッションでの追加コード変更なし

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `lsp/completion.rs` に `!Effect` 文字列リテラルが存在しない | `lsp_completion_signatures_no_effect` テスト |
| 2 | `error_catalog.rs` の `fix:` フィールドに `!Effect` 構文が存在しない | `error_catalog_fix_no_effect_syntax` テスト |
| 3 | `mcp/mod.rs` のドキュメント文字列に `!Io` 等が存在しない | `mcp_docs_no_effect_annotation` テスト |
| 4 | `CHANGELOG.md` に `[v35.8.0]` が含まれる | `changelog_has_v35_8_0` テスト |
| 5 | `Cargo.toml` バージョンが `35.8.0` | `cargo_toml_version_is_35_8_0` テスト |
| 6 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2646、今回追加テストなし・前バージョンと同数維持） | `cargo test` 実行結果 |
