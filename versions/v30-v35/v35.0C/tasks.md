# v35.8.0 (v35.0C) タスクリスト

## ステータス: COMPLETE

## T0: 事前確認

- [x] 現在のテスト数が 2621（0 failures）であることを確認
- [x] Cargo.toml バージョンが `35.7.0` であることを確認

## T1: lsp/completion.rs 修正

- [x] IO.* 関数シグネチャから `!Io` 除去（8関数）
- [x] Csv.* 関数シグネチャから `!Io` 除去（2関数）
- [x] Gen.* 関数シグネチャから `!Gen` 除去（3関数）
- [x] Http.* 関数シグネチャから `!Http` 除去（4関数）
- [x] Llm.* 関数シグネチャから `!Llm` 除去（3関数）
- [x] Db.* 関数シグネチャから `!Db` 除去（3関数）
- [x] Sys.sleep シグネチャから `!Io` 除去（1関数）
- [x] Rune 説明文（配列リテラル）から `!Effect` 除去（12件）

## T2: error_catalog.rs 修正

- [x] E0310 `fix:` を ctx: AppCtx ベースに書き換え
- [x] E0311 `fix:` を ctx: AppCtx ベースに書き換え
- [x] E0312 `fix:` を ctx: AppCtx ベースに書き換え
- [x] E0313 `fix:` を ctx: AppCtx ベースに書き換え
- [x] E0314 `fix:` を ctx: AppCtx ベースに書き換え
- [x] E0315 `fix:` を ctx: AppCtx ベースに書き換え
- [x] E0319 `fix:` を ctx: AppCtx ベースに書き換え

## T3: mcp/mod.rs 修正

- [x] db rune ドキュメント文字列から `!Io` 除去（5関数）
- [x] http rune ドキュメント文字列から `!Io` 除去（4関数）
- [x] log rune ドキュメント文字列から `!Io` 除去（4関数）

## T4: main.rs 修正

- [x] help テキスト `!DbRead/!DbWrite effects` → `DbRead/DbWrite lineage tags`

## T5: バージョン管理

- [x] `fav/Cargo.toml` バージョンを `35.8.0` に更新
- [x] `CHANGELOG.md` に `## [35.8.0]` エントリを追加
- [x] `driver.rs` の `v35700_tests::cargo_toml_version_is_35_7_0` をスタブ化

## T6: テスト追加

- [x] `driver.rs` に `v35800_tests` モジュールを追加
  - [x] `cargo_toml_version_is_35_8_0`
  - [x] `lsp_completion_signatures_no_effect`
  - [x] `error_catalog_fix_no_effect_syntax`
  - [x] `mcp_docs_no_effect_annotation`
  - [x] `changelog_has_v35_8_0`

## T7: テスト実行

- [x] `cargo test` 全通過（0 failures）
- [x] 新規 5 テストが pass

## T8: ドキュメント更新

- [x] `versions/v30-v35/v35.0C/tasks.md` を COMPLETE ステータスに更新

## コードレビュー対応

| 指摘 | 優先度 | 対応 |
|---|---|---|
| （実施後に記録） | — | — |
