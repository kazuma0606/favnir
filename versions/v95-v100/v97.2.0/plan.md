# Plan: v97.2.0 — タスク照会 + 完了操作

## 実装ステップ

1. **`runes/sap-odata/workflow.fav` に型・関数を追加**
   - `WorkflowTask` レコード型（`task_id` / `subject` / `processor` / `created_at` / `context`）
   - `TaskDecision` ADT（`Approve` / `Reject(String)`）
   - `workflow_tasks(instance_id: String) -> List<WorkflowTask>` スタブ（`List.empty()` を返す）
   - `workflow_task_complete(task_id: String, decision: TaskDecision) -> String` スタブ（`match decision` で分岐）

2. **`fav/src/driver.rs` に `mod v97200_tests` 追加**
   - `mod v97100_tests` の直後に追加
   - テスト 1: `workflow_fav_has_workflow_task` — `workflow.fav` に `WorkflowTask` が含まれることを確認
   - テスト 2: `workflow_fav_has_task_decision` — `workflow.fav` に `TaskDecision` が含まれることを確認
   - ファイルパス: `std::fs::read_to_string("../runes/sap-odata/workflow.fav")`
   - `use super::*` は不要（`std::fs` のみ使用）

3. **`cargo test` で 4,217 tests, 0 failures を確認**

4. **CI 事前確認**
   - `cargo clippy --locked -- -D warnings` pass
   - `./target/debug/fav fmt --check self/compiler.fav` pass
   - `./target/debug/fav fmt --check self/checker.fav` pass

5. **`CHANGELOG.md` に `[v97.2.0]` エントリを追加**

6. **`versions/current.md` を v97.2.0 に更新**

## 注意事項

- `TaskDecision` は `Reject(String)` タプルバリアントを持つ ADT — `Reject(msg)` でパターンマッチ
- `workflow_task_complete` の `match decision` は全バリアントを網羅すること（`Approve` / `Reject(_)`）
- スタブ引数（`instance_id`, `task_id`）のコメントを追加しておく（将来の SAP API 呼び出しで使用予定）
