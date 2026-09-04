# Tasks: v97.1.0 — `WorkflowInstance` 型 + `ctx.sap.workflow_start()`

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/current.md` の最新安定版が `v97.0.0` であることを確認する
- [x] `fav/src/driver.rs` に `mod v97000_tests` が存在することを確認する（v97.0.0 完了済みの証拠）
- [x] `cargo test 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,213 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `97.0.0` であることを確認する（パッチ版のため Cargo.toml version は宣言版 97.0.0 のまま）

## T1: `runes/sap-odata/workflow.fav` を新規作成

- [x] `WorkflowStatus` ADT（`Running` / `Completed` / `Canceled` / `Suspended`）を定義する
- [x] `WorkflowInstance` レコード型（`instance_id` / `definition` / `status` / `started_at`）を定義する
- [x] `workflow_start(definition: String, context: String) -> WorkflowInstance` スタブ関数を定義する

## T2: `fav/src/driver.rs` に `mod v97100_tests` を追加

- [x] `mod v97000_tests` の直後に `#[cfg(test)] mod v97100_tests { ... }` を追加する
- [x] `workflow_fav_exists` テストを追加する
  - `runes/sap-odata/workflow.fav` が存在することを確認
- [x] `workflow_fav_has_workflow_instance` テストを追加する
  - `workflow.fav` に `WorkflowInstance` が含まれることを確認

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,215 tests, 0 failures であることを確認する

## T4: `CHANGELOG.md` に v97.1.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に `[v97.1.0]` エントリを追加する

## T5: `versions/current.md` 更新

- [x] 最新安定版を `v97.1.0` に更新する（テスト数 4,215）

## T-last: CI 事前確認（T3 の `cargo test` 全 pass 確認後に実施すること）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
