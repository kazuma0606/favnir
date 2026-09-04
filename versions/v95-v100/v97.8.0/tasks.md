# Tasks: v97.8.0 — サイトドキュメント（Workflow / Approval パターンガイド）

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/current.md` の最新安定版が `v97.7.0` であることを確認する
- [x] `fav/src/driver.rs` に `mod v97700_tests` が存在することを確認する（v97.7.0 完了済みの証拠）
- [x] `cargo test 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,227 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `97.0.0` であることを確認する（パッチ版のため Cargo.toml version は宣言版 97.0.0 のまま）

## T1: `site/content/docs/guides/sap-workflow.mdx` を新規作成

- [x] フロントマター（title / order: 11 / category / description）を記述する
- [x] 全体像セクション（v97.1〜v97.7 機能一覧テーブル）を記述する
- [x] 承認フローの型設計セクション（`TaskDecision` / `ApprovalClient`）を記述する
- [x] `!Approval` エフェクト型の使い方セクションを記述する
- [x] iFlow connector の設定セクション（`IFlowClient` / `iflow_send`）を記述する
- [x] E2E デモのウォークスルーセクション（`workflow_demo/` / `bash run.sh`）を記述する
- [x] テスト戦略セクション（`MockWorkflowClient` / `Ctx.mock_workflow`）を記述する

## T2: `fav/src/driver.rs` に `mod v97800_tests` を追加

- [x] `mod v97700_tests` の直後に `#[cfg(test)] mod v97800_tests { ... }` を追加する
- [x] `sap_workflow_mdx_exists` テストを追加する（ファイルの存在確認）
- [x] `sap_workflow_mdx_has_approval_client` テストを追加する（`ApprovalClient` の内容確認）

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,229 tests, 0 failures であることを確認する

## T4: `CHANGELOG.md` に v97.8.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に `[v97.8.0]` エントリを追加する

## T5: `versions/current.md` 更新

- [x] `最終更新:` ヘッダーを `v97.8.0` に更新する
- [x] 最新安定版を `v97.8.0` に更新する（テスト数 4,229）

## T-last: CI 事前確認（T3 の `cargo test` 全 pass 確認後・T4/T5 完了後に実施すること。`cargo test` 再実行は不要。Clippy / fmt のみ確認する）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
