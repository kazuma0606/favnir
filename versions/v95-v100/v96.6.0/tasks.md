# Tasks: v96.6.0 — S/4HANA Clean Core REST API wrapper（`CleanCoreClient`）

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/current.md` の最新安定版が `v96.5.0` であることを確認する
- [x] `fav/src/driver.rs` に `mod v96500_tests` が存在することを確認する（v96.5.0 完了済みの証拠）
- [x] `cargo test 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,201 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `96.0.0` であることを確認する（パッチ版のため Cargo.toml version は宣言版 96.0.0 のまま）

## T1: `runes/sap-odata/clean_core.fav` を新規作成

- [x] ファイル冒頭にコメントヘッダーを記述する（ファイル名・バージョン）
- [x] `public type CleanCoreClient = { base_url: String, token: String }` を定義する
- [x] `public fn CleanCoreClient.get(client: CleanCoreClient, path: String) -> String` スタブを定義する
  - 実装: `String.concat(["GET ", client.base_url, path])`

## T2: `fav/src/driver.rs` に `mod v96600_tests` を追加

- [x] `mod v96500_tests` の直後に `#[cfg(test)] mod v96600_tests { ... }` を追加する
- [x] `clean_core_fav_exists` テストを追加する（`clean_core.fav` に `CleanCoreClient` が含まれる）
- [x] `clean_core_fav_has_get_fn` テストを追加する（`clean_core.fav` に `CleanCoreClient.get` が含まれる）
- [x] テスト内のファイルパスが `std::fs::read_to_string("../runes/sap-odata/clean_core.fav")` であることを確認する（`include_str!` ではなく `read_to_string`）

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,203 tests, 0 failures であることを確認する

## T4: `CHANGELOG.md` に v96.6.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に `[v96.6.0]` エントリを追加する

## T5: `versions/current.md` 更新

- [x] 最新安定版を `v96.6.0` に更新する（テスト数 4,203）

## T-last: CI 事前確認（T3 の `cargo test` 全 pass 確認後に実施すること）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
