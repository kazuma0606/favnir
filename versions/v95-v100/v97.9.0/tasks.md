# Tasks: v97.9.0 — 安定化・コードフリーズ

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/current.md` の最新安定版が `v97.8.0` であることを確認する
- [x] `fav/src/driver.rs` に `mod v97800_tests` が存在することを確認する（v97.8.0 完了済みの証拠）
- [x] `cargo test 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,229 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `97.0.0` であることを確認する（パッチ版のため Cargo.toml version は宣言版 97.0.0 のまま）

## T1: `fav/src/driver.rs` に `mod v97900_tests` を追加

- [x] `mod v97800_tests` の直後に `#[cfg(test)] mod v97900_tests { ... }` を追加する
- [x] `sap_workflow_mdx_has_iflow_client` テストを追加する（v97.5.0 ↔ v97.8.0 整合性確認）
- [x] `mock_fav_has_impl_approval_client` テストを追加する（v97.3.0 ↔ v97.7.0 整合性確認）

## T2: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,231 tests, 0 failures であることを確認する

## T3: `CHANGELOG.md` に v97.9.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に `[v97.9.0]` エントリを追加する

## T4: `versions/current.md` 更新

- [x] `最終更新:` ヘッダーを `v97.9.0` に更新する
- [x] 最新安定版を `v97.9.0` に更新する（テスト数 4,231）

## T-last: CI 事前確認（T2 の `cargo test` 全 pass 確認後・T3/T4 完了後に実施すること。`cargo test` 再実行は不要。Clippy / fmt のみ確認する）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
