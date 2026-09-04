# Spec: v97.1.0 — `WorkflowInstance` 型 + `ctx.sap.workflow_start()`

## Background

v97.0.0（SAP Multi-system 1.0）の完了後、SAP Workflow スプリント（v97.1〜v98.0）を開始する。
第 1 ステップとして、SAP Workflow Management API に対応する型と起動関数を追加する。

SAP ワークフローは「発注承認」「請求書検証」等の業務プロセスを SAP 上で管理する仕組み。
Favnir からワークフローを起動（`workflow_start`）し、インスタンス情報を `WorkflowInstance` 型で受け取れるようにする。

## Goals

1. `runes/sap-odata/workflow.fav` を新規作成する
2. `WorkflowStatus` ADT（`Running` / `Completed` / `Canceled` / `Suspended`）を定義する
3. `WorkflowInstance` レコード型（`instance_id`、`definition`、`status`、`started_at`）を定義する
4. `ctx.sap.workflow_start(definition, context)` スタブ関数を定義する
5. `fav/src/driver.rs` に `mod v97100_tests`（2 テスト）を追加する

## 型・API 定義

```favnir
-- runes/sap-odata/workflow.fav

public type WorkflowStatus =
    | Running
    | Completed
    | Canceled
    | Suspended

public type WorkflowInstance = {
    instance_id: String,
    definition:  String,
    status:      WorkflowStatus,
    started_at:  String
}

-- ワークフロー起動スタブ
public fn workflow_start(definition: String, context: String) -> WorkflowInstance {
    WorkflowInstance {
        instance_id: String.concat(["wf-", definition]),
        definition:  definition,
        status:      Running,
        started_at:  "2026-09-01T00:00:00Z"
    }
}
```

### 使用例

```favnir
bind instance <- ctx.sap.workflow_start("PurchaseOrderApproval", Json.encode(po))
bind _        <- ctx.io.println("Workflow started: " ++ instance.instance_id)
```

## Success Criteria

- `cargo test` で 4,215 tests, 0 failures
- `cargo clippy --locked -- -D warnings` pass
- `./target/debug/fav fmt --check self/compiler.fav` pass
- `./target/debug/fav fmt --check self/checker.fav` pass

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `runes/sap-odata/workflow.fav` | 新規作成（`WorkflowStatus` / `WorkflowInstance` / `workflow_start`） |
| `fav/src/driver.rs` | `mod v97100_tests`（2 テスト）追加 |
| `CHANGELOG.md` | `[v97.1.0]` エントリ追加（先頭） |

**注**: サイトドキュメント（`site/content/docs/guides/sap-workflow.mdx`）は v97.8.0 で対応予定。本バージョンでは対象外。

**注**: `WorkflowStatus` および `WorkflowInstance` は `public` 修飾子を付けて公開する。ロードマップのコード例では `public` が省略されているが、外部から参照するため `public` が必要。
