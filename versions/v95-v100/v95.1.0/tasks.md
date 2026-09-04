# Tasks: v95.1.0 — OData `$delta` / `DeltaLink` 型定義

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/current.md` の最新安定版が `v95.0.0` であることを確認する
- [x] `cargo test` を実行し、4,164 tests, 0 failures を確認する（着手前ベースライン）
- [x] `runes/sap-odata/batch.fav` が存在することを確認する（v94.1.0 完了済みの証拠）
- [x] `runes/sap-odata/sap_odata.fav` に `BatchOperation` / `BatchRequest` の re-export があることを確認する（v94.8.0 完了済みの証拠）
- [x] `fav/src/driver.rs` に `mod v95000_tests` が存在することを確認する（v95.0.0 完了済みの証拠）
- [x] `fav/Cargo.toml` の version が `95.0.0` であることを確認する

## T1: `runes/sap-odata/delta.fav` を新規作成する

- [x] `runes/sap-odata/delta.fav` を新規作成する
- [x] `DeltaResult<T>` 型を定義する（`entities: List<T>` / `delta_link: String` / `has_more: Bool`）
- [x] `DeletedEntity` 型を定義する（`id: String` / `reason: String`）
- [x] `public fn delta_link_is_valid(link: String) -> Bool` を定義する（`String.length(link) > 0`）
- [x] Favnir 構文で `bind` を使い、`let` を使っていないことを確認する
- [x] コメントは `--` で記述されていることを確認する

## T2: `fav/src/driver.rs` に `mod v95100_tests` を追加する

- [x] `mod v95000_tests { ... }` の直後に `#[cfg(test)] mod v95100_tests { ... }` を追加する
- [x] `delta_fav_exists` テストを追加する（`../runes/sap-odata/delta.fav` が存在することを確認）
- [x] `delta_result_type_defined` テストを追加する（`delta.fav` に `"DeltaResult"` が含まれることを確認）

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,166 tests, 0 failures であることを確認する

## T4: tasks.md を更新する

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする

## T-last: CI 事前確認（T3 の `cargo test` 全 pass 確認後に実施すること）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
