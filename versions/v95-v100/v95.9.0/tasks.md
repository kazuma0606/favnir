# Tasks: v95.9.0 — 安定化・コードフリーズ

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/current.md` の最新安定版が `v95.8.0` であることを確認する
- [x] `fav/src/driver.rs` に `mod v95800_tests` が存在することを確認する（v95.8.0 完了済みの証拠）
- [x] `fav/src/main.rs` に `sap-mock` が存在することを確認する（v95.8.0 完了済みの証拠）
- [x] `cargo test 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,182 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `95.0.0` であることを確認する
  （スプリント内バージョンでは Cargo.toml を更新しない）

## T1: `driver.rs` にスプリント総括テストを追加

- [x] `mod v95800_tests` の直後に `#[cfg(test)] mod v95900_tests { ... }` を追加する
- [x] `sprint1_sap_mock_registered` テストを追加する（`include_str!("main.rs")` で `"sap-mock"` が含まれる）
- [x] `sprint1_rpc_fav_complete` テストを追加する（`rpc.fav` に `FunctionImportParam` / `function_import` / `action_import` が含まれる）

## T2: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,184 tests, 0 failures であることを確認する

## T3: CHANGELOG / current.md 更新

- [x] `CHANGELOG.md` の先頭に `[v95.9.0]` エントリを追加する
- [x] `versions/current.md` の最新安定版を `v95.9.0` に更新する

## T4: tasks.md 更新

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする

## T-last: CI 事前確認（T2 の `cargo test` 全 pass 確認後に実施すること）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
