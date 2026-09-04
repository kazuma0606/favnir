# Tasks: v94.1.0 — `BatchRequest<T>` 型定義

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,142 tests, 0 failures を確認する（着手前ベースライン）
- [x] `versions/current.md` の最新安定版が v94.0.0 になっていることを確認する
- [x] `fav/src/driver.rs` に `mod v94000_tests` が存在することを確認する（v94.0.0 完了済みの証拠）
- [x] `fav/src/sap_metadata.rs` が存在することを確認する（v93.1.0 完了済みの証拠）
- [x] `runes/sap-odata/query_builder.fav` に `QueryBuilder` / `Page` が含まれることを確認する（v93.x.0 完了済みの証拠）

## T1: `runes/sap-odata/batch.fav` を新規作成する

- [x] `runes/sap-odata/batch.fav` を新規作成する
- [x] `BatchOperation<T>` ADT（BatchCreate / BatchUpdate / BatchDelete）を定義する
- [x] `BatchRequest<T>` record（entity_set / operations）を定義する
- [x] `BatchResponse<T>` record（succeeded / failed）を定義する
- [x] `BatchError` record（index / message）を定義する

## T2: `driver.rs` に `mod v94100_tests` を追加する

- [x] `mod v94000_tests { ... }` の直後に `#[cfg(test)] mod v94100_tests { ... }` を追加する（2 テスト）
- [x] `sap_batch_file_exists`: `../runes/sap-odata/batch.fav` が存在することを確認する
- [x] `batch_request_type_defined`: `batch.fav` に `BatchRequest` が含まれることを確認する

## T3: `cargo build` でコンパイル確認

- [x] `cargo build` を実行し、コンパイルエラーがないことを確認する

## T4: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,144 tests, 0 failures であることを確認する

## T-last: CI 事前確認（`cargo build` 完了後に実施すること）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## T5: tasks.md を COMPLETE に更新する

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする
