# Tasks: v94.9.0 — 安定化・コードフリーズ

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,158 tests, 0 failures を確認する（着手前ベースライン）
- [x] `fav/src/driver.rs` に `mod v94800_tests` が存在することを確認する（v94.8.0 完了済みの証拠）
- [x] `site/content/docs/guides/sap-integration.mdx` が存在することを確認する（v94.8.0 完了済みの証拠）

## T1: `driver.rs` に `mod v94900_tests` を追加する

- [x] `mod v94800_tests { ... }` の直後に `#[cfg(test)] mod v94900_tests { ... }` を追加する（2 テスト）
- [x] `sap_advanced_smoke_all_features`: 以下が全て存在することを確認する
  - [x] `../runes/sap-odata/batch.fav`（$batch）
  - [x] `../runes/sap-odata/query_builder.fav`（QueryBuilder<T>）
  - [x] `src/sap_metadata.rs`（Metadata Infer）— パスは `"src/sap_metadata.rs"`（`../fav/src/...` ではない）
  - [x] `../infra/lambda/sap-sync/main.tf`（SnapStart Lambda）
- [x] `sap_advanced_era_doc_complete`: `../site/content/docs/guides/sap-integration.mdx` が存在することを確認する

## T2: `CHANGELOG.md` に v94.9.0 エントリを追記する

- [x] `CHANGELOG.md` の先頭に v94.9.0 エントリを追加する

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,160 tests, 0 failures であることを確認する

## T4: tasks.md を COMPLETE に更新する

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする

## T-last: CI 事前確認（T3 の `cargo test` 全 pass 確認後に実施すること）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
