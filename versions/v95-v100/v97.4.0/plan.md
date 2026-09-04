# Plan: v97.4.0 — 条件分岐 pipeline

## 実装ステップ

1. **`infra/e2e-demo/sap-odata/pipeline_workflow.fav` を新規作成**
   - `route_by_approval_result` pipeline（`!SapOData !Approval !S3` エフェクト）
   - `stage Decide`: `ctx.approval.request_approval("発注承認", context)` で `TaskDecision` を取得
   - `|> stage Route`: `match decision` で `Approve` / `Reject(msg)` に分岐
     - `Approve`: `ctx.sap.workflow_start("POApproval", po_id)` + `ctx.s3.put_object("audit", ...)`
     - `Reject(msg)`: `ctx.io.println(...)` + `ctx.s3.put_object("audit", ...)`

2. **`fav/src/driver.rs` に `mod v97400_tests` を追加**
   - `mod v97300_tests` の直後に追加
   - テスト 1: `pipeline_workflow_fav_exists` — ファイルの存在を確認
   - テスト 2: `pipeline_workflow_fav_has_route_by_approval_result` — `route_by_approval_result` が含まれることを確認
   - ファイルパス: `std::fs::read_to_string("../infra/e2e-demo/sap-odata/pipeline_workflow.fav")`
   - `use super::*` は不要

3. **`cargo test` で 4,221 tests, 0 failures を確認**

4. **CI 事前確認**
   - `cargo clippy --locked -- -D warnings` pass
   - `./target/debug/fav fmt --check self/compiler.fav` pass
   - `./target/debug/fav fmt --check self/checker.fav` pass

5. **`CHANGELOG.md` に `[v97.4.0]` エントリを追加**

6. **`versions/current.md` を v97.4.0 に更新**

## 注意事項

- ファイルパス（driver.rs テスト）: CWD は `fav/` なので `../infra/e2e-demo/sap-odata/pipeline_workflow.fav`
- `pipeline_workflow.fav` は Favnir コードとして構文的に正しい形式で書く（`--` コメント使用）
- `match decision` の両ブランチ（`Approve` / `Reject(msg)`）を必ず記述する（全バリアント網羅）
