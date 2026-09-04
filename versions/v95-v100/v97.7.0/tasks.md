# Tasks: v97.7.0 — `MockWorkflowClient`（承認フローのオフラインテスト）

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/current.md` の最新安定版が `v97.6.0` であることを確認する
- [x] `fav/src/driver.rs` に `mod v97600_tests` が存在することを確認する（v97.6.0 完了済みの証拠）
- [x] `cargo test 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,225 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `97.0.0` であることを確認する（パッチ版のため Cargo.toml version は宣言版 97.0.0 のまま）

## T1: `runes/sap-odata/mock.fav` に `MockWorkflowClient` を追加

- [x] `use sap_odata.workflow` を mock.fav の use 宣言に追加する（`ApprovalClient` / `TaskDecision` に必要）
- [x] `MockWorkflowClient` レコード型を追加する（`auto_approve: Bool` / `reject_reason: Option<String>`）
- [x] `impl ApprovalClient for MockWorkflowClient` を追加する
  - [x] `request_approval`: `auto_approve` を match して `Approve` / `Reject(reason)` を返す

## T2: `runes/ctx/ctx.fav` に `Ctx.mock_workflow` を追加

- [x] `use sap_odata.mock` を ctx.fav の use 宣言に追加する（`MockWorkflowClient` に必要）
- [x] `Ctx.mock` の直後に `Ctx.mock_workflow(workflow: MockWorkflowClient) -> AppCtx` を追加する

## T3: `fav/src/driver.rs` に `mod v97700_tests` を追加

- [x] `mod v97600_tests` の直後に `#[cfg(test)] mod v97700_tests { ... }` を追加する
- [x] `mock_fav_has_mock_workflow_client` テストを追加する
- [x] `ctx_fav_has_mock_workflow` テストを追加する

## T4: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,227 tests, 0 failures であることを確認する
  （注: --test-threads=8 では並列ファイルアクセスで一時的失敗が見られたが、シングルスレッドで全 pass 確認）

## T5: `CHANGELOG.md` に v97.7.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に `[v97.7.0]` エントリを追加する

## T6: `versions/current.md` 更新

- [x] `最終更新:` ヘッダーを `v97.7.0` に更新する
- [x] 最新安定版を `v97.7.0` に更新する（テスト数 4,227）

## T-last: CI 事前確認（T4 の `cargo test` 全 pass 確認後・T5/T6 完了後に実施すること。`cargo test` 再実行は不要。Clippy / fmt のみ確認する）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
