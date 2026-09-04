# Tasks: v98.0.0 — SAP Workflow 1.0 宣言

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/current.md` の最新安定版が `v97.9.0` であることを確認する
- [x] `fav/src/driver.rs` に `mod v97900_tests` が存在することを確認する（v97.9.0 完了済みの証拠）
- [x] `cargo test 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,231 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `97.0.0` であることを確認する（宣言版でバージョンを 98.0.0 に上げる）

## T1: `fav/Cargo.toml` バージョン更新

- [x] `version = "97.0.0"` → `version = "98.0.0"` に変更する

## T2: `CHANGELOG.md` に v98.0.0 エントリを追加

- [x] `CHANGELOG.md` の先頭に `[v98.0.0]` エントリを追加する
  - SAP Workflow 1.0 宣言
  - v97.1.0〜v97.9.0 Sprint 成果サマリ（!Approval 型・IFlowClient・MockWorkflowClient・ガイドドキュメント・安定化）
  - 4 テスト追加（v98000_tests）

## T3: `MILESTONE.md` に v98.0.0 エントリを追加

- [x] 最新エントリとして v98.0.0 を追加する
  - 「承認フローが型になった。それが、Favnir SAP Workflow 1.0 である。」宣言文
  - v97.1.0〜v97.9.0 の達成内容リスト（`!Approval` 型・`TaskDecision`・`IFlowClient`・`MockWorkflowClient`・`Ctx.mock_workflow`・ガイドドキュメント・安定化）

## T4: `README.md` に v98.0 セクションを追加

- [x] `## v98.0 — SAP Workflow 1.0` セクションを v97.0 セクションの直前に追加する
  - 宣言文
  - 主要成果（!Approval エフェクト・IFlowClient・MockWorkflowClient・ctx.approval.*()パターン・sap-workflow ガイド）

## T5: `fav/src/driver.rs` 更新

- [x] 既存の `"97.0.0"` 文字列を `"98.0.0"` に更新する（テスト名も `cargo_toml_version_is_98_0_0` に変更）
- [x] `mod v97900_tests` の直後に `mod v98000_tests`（4 テスト）を追加する:
  - `cargo_toml_version_is_98_0_0`
  - `changelog_has_v98_0_0`
  - `milestone_has_sap_workflow`
  - `readme_mentions_sap_workflow`

## T6: `cargo clean`（★クリーンアップ）

- [x] `cd fav && cargo clean` を実行する
- [x] `fav/tmp/hello.fav` が存在することを確認する（cargo clean 後も残っているはずだが念のため）

## T7: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,235 tests, 0 failures であることを確認する

## T8: `versions/current.md` 更新

- [x] `最終更新:` ヘッダーを `v98.0.0` に更新する
- [x] 最新安定版を `v98.0.0` に更新する（テスト数 4,235）
- [x] マイルストーン表に `| v98.0 — SAP Workflow 1.0 | **完了** | v97.1〜v97.9 完了後（2026-09-02） |` を追加する

## T-last: CI 事前確認（T7 の `cargo test` 全 pass 確認後・T8 完了後に実施すること。`cargo test` 再実行は不要。Clippy / fmt のみ確認する）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
