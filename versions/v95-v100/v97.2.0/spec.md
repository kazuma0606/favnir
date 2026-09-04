# Spec: v97.2.0 — タスク照会 + 完了操作

## Background

v97.1.0 で `WorkflowInstance` 型とワークフロー起動（`workflow_start`）を実装した。
v97.2.0 では起動後のワークフロータスクを照会し、承認 / 却下の完了操作を行う型と関数を追加する。

`WorkflowTask` はワークフロー上の個別タスク（承認待ちアイテム等）を表す型。
`TaskDecision` は承認（`Approve`）または却下（`Reject(理由)`）を型安全に表現する ADT。

## Goals

1. `WorkflowTask` レコード型を `runes/sap-odata/workflow.fav` に追加する
2. `TaskDecision` ADT（`Approve` / `Reject(String)`）を追加する
3. `workflow_tasks(instance_id)` スタブ関数（タスク一覧照会）を追加する
4. `workflow_task_complete(task_id, decision)` スタブ関数（タスク完了）を追加する
5. `fav/src/driver.rs` に `mod v97200_tests`（2 テスト）を追加する

## 型・API 定義

```favnir
public type WorkflowTask = {
    task_id:    String,
    subject:    String,
    processor:  String,
    created_at: String,
    context:    String      -- JSON ペイロード
}

public type TaskDecision =
    | Approve
    | Reject(String)         -- 却下理由

-- タスク照会スタブ
public fn workflow_tasks(instance_id: String) -> List<WorkflowTask> {
    List.empty()
}

-- タスク完了スタブ（承認 / 却下）
public fn workflow_task_complete(task_id: String, decision: TaskDecision) -> String {
    match decision {
        Approve      -> String.concat(["completed:", task_id])
        Reject(msg)  -> String.concat(["rejected:", task_id, ":", msg])
    }
}
```

### 使用例

```favnir
bind tasks <- ctx.sap.workflow_tasks(instance.instance_id)
bind first <- List.first(tasks)
bind _     <- ctx.sap.workflow_task_complete(first.task_id, Approve)
```

## Success Criteria

- `cargo test` で 4,217 tests, 0 failures
- `cargo clippy --locked -- -D warnings` pass
- `./target/debug/fav fmt --check self/compiler.fav` pass
- `./target/debug/fav fmt --check self/checker.fav` pass
- `versions/current.md` が `v97.2.0`（4,217 tests）に更新されていること

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `runes/sap-odata/workflow.fav` | `WorkflowTask` / `TaskDecision` / `workflow_tasks` / `workflow_task_complete` 追加 |
| `fav/src/driver.rs` | `mod v97200_tests`（2 テスト）追加 |
| `CHANGELOG.md` | `[v97.2.0]` エントリ追加（先頭） |

**注**: `workflow_tasks` / `workflow_task_complete` はスタブ実装。実際の SAP API 呼び出しは将来実装予定。

**注**: `WorkflowTask` / `TaskDecision` / スタブ関数はすべて `public` 修飾子を付ける（v97.1.0 の `workflow.fav` スタイルに統一）。ロードマップのコード例では `public` が省略されているが、外部参照のために必要。
