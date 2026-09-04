# SAP Workflow E2E デモ

## 概要

発注書作成 → 承認フロー起動 → 自動承認ルーティング → SAP 反映 の完全な E2E デモ。

`route_by_approval_result` pipeline（`../pipeline_workflow.fav`）が承認結果に基づいて
自動的にルーティングを行い、承認時は SAP Workflow 起動 + S3 監査ログ書き込み、
却下時はログ出力 + S3 却下ログ書き込みを実行する。

## 前提条件

- Favnir CLI (`fav`) がインストール済みであること
- `infra/e2e-demo/sap-odata/pipeline_workflow.fav` が存在すること（v97.4.0 で作成済み）

## 実行手順

```sh
bash run.sh
```

## pipeline の流れ

```
stage Decide  : ctx.approval.request_approval() で TaskDecision を取得
    |
    v
stage Route   : match decision {
                  Approve     -> SAP Workflow 起動 + S3 承認ログ
                  Reject(msg) -> ログ出力 + S3 却下ログ
                }
```
