# Tasks: v97.2.0 — タスク照会 + 完了操作

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/current.md` の最新安定版が `v97.1.0` であることを確認する
- [x] `fav/src/driver.rs` に `mod v97100_tests` が存在することを確認する（v97.1.0 完了済みの証拠）
- [x] `cargo test 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,215 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `97.0.0` であることを確認する（パッチ版のため Cargo.toml version は宣言版 97.0.0 のまま）

## T1: `runes/sap-odata/workflow.fav` に型・関数を追加

- [x] `WorkflowTask` レコード型を追加する（`task_id` / `subject` / `processor` / `created_at` / `context`）
- [x] `TaskDecision` ADT を追加する（`Approve` / `Reject(String)`）
- [x] `workflow_tasks(instance_id: String) -> List<WorkflowTask>` スタブを追加する
- [x] `workflow_task_complete(task_id: String, decision: TaskDecision) -> String` スタブを追加する（`match decision` で全バリアント網羅）

## T2: `fav/src/driver.rs` に `mod v97200_tests` を追加

- [x] `mod v97100_tests` の直後に `#[cfg(test)] mod v97200_tests { ... }` を追加する
- [x] `workflow_fav_has_workflow_task` テストを追加する（`workflow.fav` に `WorkflowTask` が含まれることを確認）
- [x] `workflow_fav_has_task_decision` テストを追加する（`workflow.fav` に `TaskDecision` が含まれることを確認）

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,217 tests, 0 failures であることを確認する

## T4: `CHANGELOG.md` に v97.2.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に `[v97.2.0]` エントリを追加する

## T5: `versions/current.md` 更新

- [x] 最新安定版を `v97.2.0` に更新する（テスト数 4,217）

## T-last: CI 事前確認（T3 の `cargo test` 全 pass 確認後・T4/T5 完了後に実施すること）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
