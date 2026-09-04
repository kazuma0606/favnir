# Roadmap v97.1.0 〜 v98.0.0 — SAP Workflow 1.0

Date: 2026-08-30
Status: 未着手

マスターロードマップ: [roadmap-v95.1-v100.0.md](roadmap-v95.1-v100.0.md)

---

## 前提

- 直前完了: v97.0.0「SAP Multi-system 1.0 宣言」（tests = 4,213）
- 本スプリントは SAP Platform Era の第 3 スプリント
- 目標: v98.0.0「SAP Workflow 1.0 宣言」（tests = 4,235）

### 着手前チェックリスト

- `versions/current.md` の最新安定版が v97.0.0 になっていることを確認する
- `runes/sap-odata/cross_system.fav` が存在することを確認する（v96.7.0 完了済みの証拠）
- `fav/src/driver.rs` に `mod v97000_tests` が存在することを確認する（v97.0.0 完了済みの証拠）
- `fav/Cargo.toml` の version が `97.0.0` であることを確認する

### スプリントの性格

SAP Platform Era の**ワークフロー・承認フロースプリント**。

SAP Workflow Management による承認フロー起動・タスク操作と、
SAP BTP Integration Suite（iFlow）への接続を実装する。
「人間の判断」を `!Approval` エフェクト型で型システムに取り込むことが最大のテーマ。

---

## バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v97.1.0 | `WorkflowInstance` 型 + `ctx.sap.workflow_start()` | 4213 + 2 = 4215 | 未着手 |
| v97.2.0 | タスク照会 `ctx.sap.workflow_tasks()` + 完了操作 `workflow_task_complete()` | 4215 + 2 = 4217 | 未着手 |
| v97.3.0 | `!Approval` エフェクト型（人間の承認を型で表現） | 4217 + 2 = 4219 | 未着手 |
| v97.4.0 | 条件分岐 pipeline（ワークフロー結果に基づく `match` stage） | 4219 + 2 = 4221 | 未着手 |
| v97.5.0 | SAP BTP Integration Suite connector（`iFlowClient`） | 4221 + 2 = 4223 | 未着手 |
| v97.6.0 | E2E デモ（発注 → 自動承認ルーティング → SAP 反映） | 4223 + 2 = 4225 | 未着手 |
| v97.7.0 | `MockWorkflowClient`（承認フローのオフラインテスト） | 4225 + 2 = 4227 | 未着手 |
| v97.8.0 | サイトドキュメント（Workflow / Approval パターンガイド） | 4227 + 2 = 4229 | 未着手 |
| v97.9.0 | 安定化・コードフリーズ | 4229 + 2 = 4231 | 未着手 |
| v98.0.0 | SAP Workflow 1.0 宣言 ★クリーンアップ | 4231 + 4 = 4235 | 未着手 |

---

## v97.1.0 — `WorkflowInstance` 型 + `ctx.sap.workflow_start()`

SAP Workflow Management API に対応する型と起動関数を追加する。

```favnir
type WorkflowStatus =
    | Running
    | Completed
    | Canceled
    | Suspended

type WorkflowInstance = {
    instance_id: String,
    definition:  String,
    status:      WorkflowStatus,
    started_at:  String
}

-- ワークフロー起動
bind instance <- ctx.sap.workflow_start("PurchaseOrderApproval", Json.encode(po))
bind _        <- ctx.io.println("Workflow started: " ++ instance.instance_id)
```

**修正ファイル**: `runes/sap-odata/workflow.fav`（新規）、`fav/src/driver.rs`

---

## v97.2.0 — タスク照会 + 完了操作

ワークフロータスクの照会と完了（承認 / 却下）操作を実装する。

```favnir
type WorkflowTask = {
    task_id:     String,
    subject:     String,
    processor:   String,
    created_at:  String,
    context:     String      -- JSON ペイロード
}

type TaskDecision =
    | Approve
    | Reject(String)         -- 却下理由

-- タスク照会
bind tasks <- ctx.sap.workflow_tasks(instance.instance_id)
bind first <- List.first(tasks)   -- tasks.first ではなく List.first(tasks) を使う

-- タスク完了（承認）
bind _ <- ctx.sap.workflow_task_complete(first.task_id, Approve)
```

**修正ファイル**: `runes/sap-odata/workflow.fav`、`fav/src/driver.rs`

---

## v97.3.0 — `!Approval` エフェクトマーカー + ctx interface

`!Approval` を pipeline シグネチャのエフェクトマーカーとして追加する。
ctx パターンに従い、`ApprovalClient` interface を `AppCtx` に `approval: ApprovalClient` フィールドとして追加する。
`effect Approval { ... }` による独立エフェクト宣言は行わない（ctx パターン統一）。

```favnir
-- ApprovalClient は AppCtx のフィールドとして注入される（SapClient と同様）
-- ctx.approval.request_approval() でアクセス

-- !Approval マーカーを持つ pipeline — 型から承認フローが必要なことが分かる
pipeline approve_purchase_order !SapOData !Approval {
    stage Request {
        bind po       <- ctx.sap.purchase_order_by_id(po_id, false)
        bind decision <- ctx.approval.request_approval(
            "発注 " ++ po.po_number ++ " の承認依頼",
            Json.encode(po)
        )
    }
    |> stage Apply {
        bind _ <- match decision {
            Approve     -> ctx.sap.workflow_start("POApproval", po_id)
            Reject(msg) -> ctx.io.println("却下: " ++ msg)
        }
    }
}
```

**修正ファイル**: `runes/sap-odata/workflow.fav`、`runes/ctx/ctx.fav`（`approval` フィールド追加）、`fav/src/driver.rs`
**Rust 側**: `Effect::Approval` を `Effect` enum に追加、`checker.fav` の exhaustive match 更新

---

## v97.4.0 — 条件分岐 pipeline

ワークフロー結果（承認 / 却下）に基づいて処理を分岐する pipeline パターンを実装する。

```favnir
pipeline route_by_approval_result !SapOData !Approval !S3 {
    stage Decide {
        bind decision <- ctx.approval.request_approval("発注承認", context)
    }
    |> stage Route {
        bind _ <- match decision {
            Approve     -> pipeline {
                bind _ <- ctx.sap.workflow_start("POApproval", po_id)
                bind _ <- ctx.s3.put_object("audit", po_id ++ "_approved.json", context)
            }
            Reject(msg) -> pipeline {
                bind _ <- ctx.io.println("却下理由: " ++ msg)
                bind _ <- ctx.s3.put_object("audit", po_id ++ "_rejected.json", msg)
            }
        }
    }
}
```

**修正ファイル**: `infra/e2e-demo/sap-odata/pipeline_workflow.fav`（新規）、`fav/src/driver.rs`

---

## v97.5.0 — SAP BTP Integration Suite connector

SAP BTP Integration Suite の iFlow（統合フロー）を Favnir から呼び出す `iFlowClient` を追加する。

```favnir
type IFlowClient = {
    base_url:  String,
    oauth_url: String,
    client_id: String
}

type IFlowMessage = {
    headers: List<String>,
    body:    String
}

-- iFlow を起動してメッセージを送信
bind resp <- ctx.sap_iflow.send("OrderSync_iFlow", IFlowMessage {
    headers: ["Content-Type: application/json"],
    body:    Json.encode(order)
})
```

**修正ファイル**: `runes/sap-odata/iflow.fav`（新規）、`fav/src/driver.rs`

---

## v97.6.0 — E2E デモ（発注 → 承認 → SAP 反映）

発注書作成 → 承認フロー起動 → 承認完了 → SAP 反映 の完全な E2E デモを実装する。

```
infra/e2e-demo/sap-odata/
  pipeline_workflow.fav    -- 既存（v97.4.0 で作成）
  workflow_demo/
    README.md              -- デモ手順
    run.sh                 -- 実行スクリプト
```

**修正ファイル**: `infra/e2e-demo/sap-odata/workflow_demo/`（新規）、`fav/src/driver.rs`

---

## v97.7.0 — `MockWorkflowClient`

承認フローをオフラインでテストするための `MockWorkflowClient` を追加する。

```favnir
-- モック: 常に承認
type MockWorkflowClient = {
    auto_approve: Bool,
    reject_reason: Option<String>
}

-- Ctx.mock に workflow クライアントを追加
bind ctx <- Ctx.mock(MockWorkflowClient { auto_approve: true, reject_reason: Option.none() })
```

**修正ファイル**: `runes/sap-odata/mock.fav`（既存）、`runes/ctx/ctx.fav`、`fav/src/driver.rs`

---

## v97.8.0 — サイトドキュメント

`site/content/docs/guides/sap-workflow.mdx` を新規作成する。

**内容**:
- Workflow / Approval パターンガイド
- `!Approval` エフェクト型の使い方
- iFlow connector の設定方法
- E2E デモのウォークスルー

**修正ファイル**: `site/content/docs/guides/sap-workflow.mdx`（新規）、`fav/src/driver.rs`

---

## v97.9.0 — 安定化・コードフリーズ

- `mod v97900_tests`（2 テスト）を `fav/src/driver.rs` に追加する
  - `sap_workflow_mdx_has_iflow_client`: v97.5.0（iflow.fav）↔ v97.8.0（MDX）の整合性確認
  - `mock_fav_has_impl_approval_client`: v97.3.0（ApprovalClient）↔ v97.7.0（MockWorkflowClient）の整合性確認
- 全テスト通過確認（4,231 tests, 0 failures）
- `cargo clippy --locked -- -D warnings` 通過
- `./target/debug/fav fmt --check self/compiler.fav` 通過
- `./target/debug/fav fmt --check self/checker.fav` 通過

---

## v98.0.0 — SAP Workflow 1.0 宣言

**宣言文**:

> 「Favnir が、人間の判断を型に閉じ込めた。
>
>  `!Approval` エフェクトが pipeline のシグネチャに現れた時、
>  それはコードが「ここで人間の承認が必要」と語っているのだ。
>
>  承認フローが型になった。それが、Favnir SAP Workflow 1.0 である。」

**v98000_tests（4 テスト）**:
- `cargo_toml_version_is_98_0_0`
- `changelog_has_v98_0_0`
- `milestone_has_sap_workflow`
- `readme_mentions_sap_workflow`

---

## スプリント終了時の確認

- [ ] 4,235 tests, 0 failures
- [ ] `cargo clean` を実施する（★クリーンアップ）
- [ ] `cargo test` で 4,235 tests, 0 failures を再確認する（cargo clean 後）
- [ ] `cargo clippy --locked -- -D warnings` pass
- [ ] `./target/debug/fav fmt --check self/compiler.fav` pass
- [ ] `./target/debug/fav fmt --check self/checker.fav` pass
- [ ] `versions/current.md` を v98.0.0 に更新
- [ ] `MILESTONE.md` に v98.0.0 エントリを追加
- [ ] `README.md` に `## v98.0 — SAP Workflow 1.0` セクションを追加
