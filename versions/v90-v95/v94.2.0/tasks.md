# Tasks: v94.2.0 — `ChangeSet` + `ctx.sap.batch()` 実装

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,144 tests, 0 failures を確認する（着手前ベースライン）
- [x] `runes/sap-odata/batch.fav` に `BatchRequest` が含まれることを確認する（v94.1.0 完了済みの証拠）
- [x] `fav/src/driver.rs` に `mod v94100_tests` が存在することを確認する（v94.1.0 完了済みの証拠）
- [x] `runes/sap-odata/types.fav` に `SapClient` interface が存在することを確認する

## T1: `batch.fav` に `ChangeSet<T>` と `batch_request_builder` を追加する

- [x] `runes/sap-odata/batch.fav` に `public type ChangeSet<T>` を追記する（operations: List<BatchOperation<T>>）
- [x] `runes/sap-odata/batch.fav` に `public fn batch_request_builder<T>` を追記する

## T2: `types.fav` の `SapClient` interface に `batch` メソッドを追加する

- [x] `runes/sap-odata/types.fav` の `SapClient` interface 末尾に `fn batch(...)` を追加する

## T3: `driver.rs` に `mod v94200_tests` を追加する

- [x] `mod v94100_tests { ... }` の直後に `#[cfg(test)] mod v94200_tests { ... }` を追加する（2 テスト）
- [x] `change_set_type_defined`: `batch.fav` に `ChangeSet` が含まれることを確認する
- [x] `sap_client_has_batch_method`: `types.fav` に `batch` が含まれることを確認する

## T4: `cargo build` でコンパイル確認

- [x] `cargo build` を実行し、コンパイルエラーがないことを確認する

## T5: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,146 tests, 0 failures であることを確認する

## T5a: `CHANGELOG.md` に v94.2.0 エントリを追記する

- [x] `CHANGELOG.md` の先頭に v94.2.0 エントリを追加する（`change_set_type_defined` / `sap_client_has_batch_method` テスト記載）

## T-last: CI 事前確認（T5 の `cargo test` 全 pass 確認後に実施すること）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## T6: tasks.md を COMPLETE に更新する

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする
