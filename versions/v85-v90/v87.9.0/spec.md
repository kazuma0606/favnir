# Spec: v87.9.0 — 安定化・コードフリーズ

## Background

v87.1〜v87.8 で SalesOrder の型定義・CRUD・ページネーション・売上レポート集計・テストを実装した。
本バージョンは安定化スプリントであり、v87 スプリント全体の整合性を確認する。
新機能追加は行わず、バグ修正のみ受け入れる。

## Goals

1. v87.1〜v87.8 の全実装を通しで確認する
2. SalesOrder CRUD（`sales_orders` / `sales_order_by_id` / `create_sales_order`）が揃っていることを Rust テストで担保する
3. シナリオ 2（日次売上レポートパイプライン）の `pipeline.fav` が `sap_odata.build_sales_report(` の呼び出しを含むことを Rust テストで担保する

## Success Criteria（Rust テストで担保）

- `runes/sap-odata/sales_order.fav` に SalesOrder CRUD 3 関数がすべて含まれる:
  - `public fn sales_orders(`
  - `public fn sales_order_by_id(`
  - `public fn create_sales_order(`
- `infra/e2e-demo/sap-odata/pipeline.fav` にシナリオ 2 の関数が含まれる:
  - `sap_odata.build_sales_report(`
- `cargo test` で 3,993 tests, 0 failures
- Rust テスト 2 件:
  - `sap_sales_order_crud_covered`（`sales_order.fav` に CRUD 3 関数が揃っていることを確認）
  - `sap_sales_scenario2_report_pipeline_exists`（`pipeline.fav` に `sap_odata.build_sales_report(` が含まれることを確認）

## Files to Modify

| ファイル | 変更種別 |
|---|---|
| `fav/src/driver.rs` | `mod v87900_tests` 追加 |

※ 新機能追加なし。バグ修正が必要な場合のみ該当ファイルを追記する。
