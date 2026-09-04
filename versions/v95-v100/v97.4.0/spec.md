# Spec: v97.4.0 — 条件分岐 pipeline

## Background

v97.3.0 で `ApprovalClient` interface と `!Approval` エフェクトマーカーを導入した。
v97.4.0 では承認フロー結果（`Approve` / `Reject(msg)`）に基づいて処理を分岐する
条件分岐 pipeline の E2E デモファイルを実装する。

`ctx.approval.request_approval()` は `TaskDecision` 型（v97.2.0 で導入）を返す。
`TaskDecision` のバリアントは `Approve` / `Reject(String)` の 2 種類。
`match decision` を `|> stage Route` 内で使い、承認時と却下時で別々の処理（SAP 書き込み・S3 監査ログ）を実行するパターン。

**注**: pipeline 定義内の `po_id` / `context` はデモ目的のプレースホルダー変数。
実際の使用時は pipeline パラメータとして宣言（例: `pipeline route_by_approval_result(po_id: String, context: String) !SapOData !Approval !S3`）するか、前段 stage で束縛する。

## Goals

1. `infra/e2e-demo/sap-odata/pipeline_workflow.fav` を新規作成する
2. `route_by_approval_result` pipeline を定義する（`!SapOData !Approval !S3` エフェクト）
3. `fav/src/driver.rs` に `mod v97400_tests`（2 テスト）を追加する

## Pipeline 定義

```favnir
-- infra/e2e-demo/sap-odata/pipeline_workflow.fav
-- 承認フロー結果による条件分岐 pipeline（v97.4.0）

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

## Success Criteria

- `cargo test` で 4,221 tests, 0 failures
- `cargo clippy --locked -- -D warnings` pass
- `./target/debug/fav fmt --check self/compiler.fav` pass
- `./target/debug/fav fmt --check self/checker.fav` pass
- `versions/current.md` が `v97.4.0`（4,221 tests）に更新されていること

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `infra/e2e-demo/sap-odata/pipeline_workflow.fav` | 新規作成（`route_by_approval_result` pipeline） |
| `fav/src/driver.rs` | `mod v97400_tests`（2 テスト）追加 |
| `CHANGELOG.md` | `[v97.4.0]` エントリ追加（先頭） |

**注**: `pipeline_workflow.fav` はデモ / ドキュメント目的のファイル。実際のコンパイル実行はしない。
