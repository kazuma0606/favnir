# Plan: v97.1.0 — `WorkflowInstance` 型 + `ctx.sap.workflow_start()`

## 実装ステップ

1. **`runes/sap-odata/workflow.fav` 新規作成**
   - `WorkflowStatus` ADT（`Running` / `Completed` / `Canceled` / `Suspended`）
   - `WorkflowInstance` レコード型（`instance_id` / `definition` / `status` / `started_at`）
   - `workflow_start(definition: String, context: String) -> WorkflowInstance` スタブ関数

2. **`fav/src/driver.rs` に `mod v97100_tests` 追加**
   - `mod v97000_tests` の直後に追加
   - テスト 1: `workflow_fav_exists` — `runes/sap-odata/workflow.fav` が存在することを確認
   - テスト 2: `workflow_fav_has_workflow_instance` — `workflow.fav` に `WorkflowInstance` が含まれることを確認

3. **`cargo test` で 4,215 tests, 0 failures を確認**

4. **CI 事前確認**
   - `cargo clippy --locked -- -D warnings` が pass することを確認する
   - `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
   - `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

5. **`CHANGELOG.md` に `[v97.1.0]` エントリを追加**

6. **`versions/current.md` を v97.1.0 に更新**

## 注意事項

- `runes/sap-odata/workflow.fav` はスタブ実装（`workflow_start` の戻り値は固定）
- `WorkflowStatus` の ADT バリアントは `public type` で公開する
- driver.rs テストのファイルパス: `std::fs::read_to_string("../runes/sap-odata/workflow.fav")`
  （CWD は `fav/` なので `../runes/` が正しいパス）
- `use super::*` は不要（std::fs のみ使用）
