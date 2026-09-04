# Tasks: v96.9.0 — 安定化・コードフリーズ

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/current.md` の最新安定版が `v96.8.0` であることを確認する
- [x] `fav/src/driver.rs` に `mod v96800_tests` が存在することを確認する（v96.8.0 完了済みの証拠）
- [x] `cargo test 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,207 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `96.0.0` であることを確認する（パッチ版のため Cargo.toml version は宣言版 96.0.0 のまま）

## T1: `fav/src/driver.rs` に `mod v96900_tests` を追加

- [x] `mod v96800_tests` の直後に `#[cfg(test)] mod v96900_tests { ... }` を追加する
- [x] `v96_sprint_new_rune_files_present` テストを追加する
  - `clean_core.fav`、`cross_system.fav`、`connection.fav` の 3 ファイルがすべて存在することを確認
- [x] `v96_sprint_connection_fav_has_retry_on_status` テストを追加する
  - `connection.fav` に `retry_on_status` が含まれることを確認

## T2: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,209 tests, 0 failures であることを確認する

## T3: `CHANGELOG.md` に v96.9.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に `[v96.9.0]` エントリを追加する

## T4: `versions/current.md` 更新

- [x] 最新安定版を `v96.9.0` に更新する（テスト数 4,209）

## T-last: CI 事前確認（T2 の `cargo test` 全 pass 確認後に実施すること）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
