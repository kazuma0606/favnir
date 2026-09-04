# Spec: v96.9.0 — 安定化・コードフリーズ

## Background

v96.1.0〜v96.8.0 で SAP Multi-system スプリントの主要機能実装が完了した。
v96.9.0 では新機能追加を行わず、スプリント全体の安定化とコードフリーズを行う。

具体的には v96.x.0 で追加した全 Rune ファイルが正しく揃っているかを確認する
統合チェックテストを追加し、v97.0.0 宣言への準備を整える。

## Goals

1. `fav/src/driver.rs` に `mod v96900_tests`（2 テスト）を追加する
   - `v96_sprint_new_rune_files_present`: v96.x.0 で追加した 3 Rune ファイルが存在することを確認
   - `v96_sprint_connection_fav_has_retry_on_status`: `RetryPolicy` の `retry_on_status` フィールドを確認
2. 全 CI チェック通過（cargo test / clippy / fav fmt）

## Success Criteria

- `fav/src/driver.rs` に `mod v96900_tests` が含まれる
- `cargo test` で 4,209 tests, 0 failures
- `cargo clippy --locked -- -D warnings` 通過
- `./target/debug/fav fmt --check self/compiler.fav` 通過
- `./target/debug/fav fmt --check self/checker.fav` 通過

## Error Codes

新規エラーコードなし。

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `mod v96900_tests`（2 テスト）を追加 |
