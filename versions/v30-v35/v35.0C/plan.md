# v35.8.0 (v35.0C) 実装計画

## Phase 1: lsp/completion.rs 修正

`!Io`, `!Http`, `!Llm`, `!Db`, `!Gen` をシグネチャ末尾から除去。
Python スクリプトで一括置換（正規表現 ` ![A-Z][a-zA-Z]*"` → `"`）。

## Phase 2: error_catalog.rs 修正

E0310〜E0319 の `fix:` フィールドを `ctx: AppCtx` ベースに書き換え。
各エラーごとに個別 Edit（7 件）。

## Phase 3: mcp/mod.rs 修正

db/http/log rune のドキュメント文字列から ` !Io` を除去（正規表現 ` !Io\\n` → `\\n`）。

## Phase 4: main.rs 修正

`!DbRead/!DbWrite effects` → `DbRead/DbWrite lineage tags`（1 行 Edit）。

## Phase 5: バージョン管理

- `Cargo.toml`: `35.7.0` → `35.8.0`
- `CHANGELOG.md`: `## [35.8.0]` エントリ追加
- `driver.rs` v35600_tests の cargo_toml バージョンアサートをスタブ化（v35.8.0 で実施済みパターン）

## Phase 6: テスト追加・実行

`driver.rs` に `v35800_tests` 5 件追加 → `cargo test` 全通過確認。

## 実装順序

1. completion.rs（Python 一括）
2. error_catalog.rs（7件 Edit）
3. mcp/mod.rs（Python 一括）
4. main.rs（1件 Edit）
5. Cargo.toml + CHANGELOG.md
6. driver.rs テスト追加
7. cargo test
