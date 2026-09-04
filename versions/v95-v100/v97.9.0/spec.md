# Spec: v97.9.0 — 安定化・コードフリーズ

## Background

v97.1.0〜v97.8.0 で SAP Workflow Sprint の全機能実装が完了した。
本バージョンは宣言版 v98.0.0 の前段として、全テスト通過・CI チェック・スプリント整合性を確認する
安定化バージョン。新機能は追加しない。

## Goals

1. `fav/src/driver.rs` に `mod v97900_tests` を追加する（2 テスト）
   - v97.x スプリント全体の整合性を横断的に確認する安定化テスト
2. 全 CI チェックを通過することを確認する

## 安定化テスト（2 件）

```rust
// sap-workflow.mdx が IFlowClient に言及していること
// （v97.5.0 iflow.fav ↔ v97.8.0 MDX の整合性確認）
fn sap_workflow_mdx_has_iflow_client()

// mock.fav が impl ApprovalClient for MockWorkflowClient を含むこと
// （v97.3.0 ApprovalClient ↔ v97.7.0 MockWorkflowClient の整合性確認）
fn mock_fav_has_impl_approval_client()
```

## Success Criteria

- `mod v97900_tests` の全テストが pass する
- `cargo test` で 4,231 tests, 0 failures（+2）
- `cargo clippy --locked -- -D warnings` が pass する
- `./target/debug/fav fmt --check self/compiler.fav` が pass する
- `./target/debug/fav fmt --check self/checker.fav` が pass する

## Error Codes

新規エラーコードなし。

## Files to Modify

| ファイル | 種別 | 内容 |
|---|---|---|
| `fav/src/driver.rs` | 追記 | `mod v97900_tests`（2 テスト） |
| `CHANGELOG.md` | 追記 | v97.9.0 エントリ |
| `versions/current.md` | 更新 | 最新安定版を v97.9.0 に変更 |
