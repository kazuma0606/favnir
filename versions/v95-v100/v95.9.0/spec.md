# Spec: v95.9.0 — 安定化・コードフリーズ

## Background

v95.1.0〜v95.8.0（SAP Real-time Sprint 1）の全成果物を確認・安定化する。
新機能の追加は行わず、スプリント全体の整合性チェックと CI 全通過を確認する。

スプリント成果物の要約:
- v95.1.0: `DeltaResult<T>` / `DeletedEntity` 型（OData $delta）
- v95.2.0: `ctx.sap.delta_fetch<T>()` （差分取得インターフェース）
- v95.3.0: `SapEventClient` interface + `effect_catalog.rs`
- v95.4.0: `pipeline_realtime.fav`（イベント駆動 pipeline デモ）
- v95.5.0: `NewSalesOrderWithItems` / `create_sales_order_deep`（Deep Insert）
- v95.6.0: `FunctionImportParam` / `function_import<T>` / `action_import`（RPC）
- v95.7.0: `BatchItemResult<T>` / `PartialSuccess<T>` / `batch_with_partial`（部分失敗）
- v95.8.0: `SapMockServer` / `cmd_sap_mock` / `fav sap-mock` コマンド

## Goals

1. スプリント総括テスト 2 件を `driver.rs` に追加する
   - `sprint1_sap_mock_registered`: `main.rs` に `sap-mock` コマンドが登録されている
   - `sprint1_rpc_fav_complete`: `rpc.fav` に `FunctionImportParam` / `function_import` / `action_import` が含まれる
2. CI 全通過（clippy / self-fmt）を確認する

## Files to Modify

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/driver.rs` | 修正 | `mod v95900_tests`（2 件）追加 |

## Success Criteria

- `cargo test` で 4,184 tests, 0 failures
- `cargo clippy --locked -- -D warnings` が pass する
- `./target/debug/fav fmt --check self/compiler.fav` が pass する
- `./target/debug/fav fmt --check self/checker.fav` が pass する
