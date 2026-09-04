# Spec: v97.6.0 — E2E デモ（発注 → 承認 → SAP 反映）

## Background

v97.4.0 で `pipeline_workflow.fav`（条件分岐 pipeline）、v97.5.0 で `iflow.fav`（BTP connector）が完成した。
本バージョンでは、発注書作成 → 承認フロー起動 → 承認完了 → SAP 反映 の完全な E2E デモを
`infra/e2e-demo/sap-odata/workflow_demo/` ディレクトリとして整備する。

## Goals

1. `infra/e2e-demo/sap-odata/workflow_demo/README.md` を新規作成する
   - デモの目的・前提条件・実行手順を記載する
2. `infra/e2e-demo/sap-odata/workflow_demo/run.sh` を新規作成する
   - `fav run` を用いた pipeline 実行スクリプト
3. `fav/src/driver.rs` に `mod v97600_tests` を追加する（2 テスト）

## ディレクトリ構成

```
infra/e2e-demo/sap-odata/
  pipeline_workflow.fav    -- 既存（v97.4.0 で作成）
  workflow_demo/
    README.md              -- デモ手順（本バージョンで新規）
    run.sh                 -- 実行スクリプト（本バージョンで新規）
```

## README.md の内容（概要）

```markdown
# SAP Workflow E2E デモ

## 概要

発注書作成 → 承認フロー起動 → 自動承認ルーティング → SAP 反映 の完全な E2E デモ。

`route_by_approval_result` pipeline（`pipeline_workflow.fav`）が承認結果に基づいて
自動的にルーティングを行い、承認時は SAP Workflow 起動 + S3 監査ログ書き込み、
却下時はログ出力 + S3 却下ログ書き込みを実行する。

## 前提条件

- Favnir CLI (`fav`) がインストール済みであること
- `infra/e2e-demo/sap-odata/pipeline_workflow.fav` が存在すること（v97.4.0 で作成済み）

## 実行手順

```sh
bash run.sh
```
```

## run.sh の内容（概要）

```sh
#!/usr/bin/env bash
set -euo pipefail

# SAP Workflow E2E デモ実行スクリプト（v97.6.0）
# route_by_approval_result pipeline を実行する（bash run.sh で呼び出す想定）
fav run ../pipeline_workflow.fav
```

## Success Criteria

- `infra/e2e-demo/sap-odata/workflow_demo/README.md` が存在する
- `infra/e2e-demo/sap-odata/workflow_demo/run.sh` が存在する
- `mod v97600_tests` の全テストが pass する
- `cargo test` で 4,225 tests, 0 failures（+2）

## Error Codes

新規エラーコードなし。

## Files to Modify

| ファイル | 種別 | 内容 |
|---|---|---|
| `infra/e2e-demo/sap-odata/workflow_demo/README.md` | 新規 | デモ手順 |
| `infra/e2e-demo/sap-odata/workflow_demo/run.sh` | 新規 | 実行スクリプト |
| `fav/src/driver.rs` | 追記 | `mod v97600_tests`（2 テスト） |
| `CHANGELOG.md` | 追記 | v97.6.0 エントリ |
| `versions/current.md` | 更新 | 最新安定版を v97.6.0 に変更 |
