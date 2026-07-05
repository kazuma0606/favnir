# v35.7.0 (v35.0B) タスクリスト

## ステータス: COMPLETE

## T0: 事前確認

- [x] 現在のテスト数が 2616（0 failures）であることを確認
- [x] Cargo.toml バージョンが `35.6.0` であることを確認
- [x] `docs_server.rs` に `!Io` が残存することを確認（grep）

## T1: docs_server.rs 修正

- [x] `IO.println` の `signature` から `!Io` を除去
- [x] `IO.println` の `effects` を `&[]` に変更
- [x] `IO.print` の `signature` から `!Io` を除去
- [x] `IO.print` の `effects` を `&[]` に変更
- [x] `IO.read_line` の `signature` から `!Io` を除去
- [x] `IO.read_line` の `effects` を `&[]` に変更

## T2: バージョン管理

- [x] `fav/Cargo.toml` バージョンを `35.7.0` に更新
- [x] `CHANGELOG.md` に `## [35.7.0]` エントリを追加

## T3: テスト追加

- [x] `driver.rs` に `v35700_tests` モジュールを追加
  - [x] `cargo_toml_version_is_35_7_0`
  - [x] `docs_server_io_signatures_no_effect`
  - [x] `docs_server_io_effects_empty`
  - [x] `changelog_has_v35_7_0`
  - [x] `effect_annotation_fully_purged`

## T4: テスト実行

- [x] `cargo test` 全通過（0 failures）
- [x] 新規 5 テストが pass していることを確認

## T5: ドキュメント更新

- [x] `versions/v30-v35/v35.0B/tasks.md` を COMPLETE ステータスに更新
- [x] 全チェックボックスを `[x]` に変更

## コードレビュー対応

| 指摘 | 優先度 | 対応 |
|---|---|---|
| （実施後に記録） | — | — |
