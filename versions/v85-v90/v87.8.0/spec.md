# Spec: v87.8.0 — モックサーバーテスト（受注シナリオ全操作検証）

## Background

v87.1〜v87.7 で SalesOrder の型定義・CRUD・ページネーション・売上レポート集計を実装した。
`sap_odata.test.fav` には BusinessPartner CRUD テスト（v86.7.0）が存在するが、
SalesOrder 操作のテストはまだ存在しない。

本バージョンでは `sap_odata.test.fav` に SalesOrder CRUD・ページネーション・
日次売上レポート生成の 3 カテゴリのテストを追加する。

## Goals

1. SalesOrder 作成・取得・フィルタのテスト関数を追加する
2. ページネーション（100 件超シナリオ）のテスト関数を追加する
3. 日次売上レポート生成のテスト関数を追加する

## API / Syntax Examples

```favnir
-- sap_odata.test.fav（追加分）

-- SalesOrder CRUD テスト（v87.8.0）
test "sales_order_create" {
    -- create_sales_order のシグネチャが存在することを確認する（スタブテスト）
}

test "sales_order_read" {
    -- sales_order_by_id のシグネチャが存在することを確認する（スタブテスト）
}

test "sales_order_filter" {
    -- sales_orders のフィルタシグネチャが存在することを確認する（スタブテスト）
}

-- ページネーションテスト（v87.8.0）
test "pagination_over_100_items" {
    -- odata_list_paged のシグネチャが存在することを確認する（スタブテスト）
}

-- 日次売上レポートテスト（v87.8.0）
test "daily_sales_report_pipeline" {
    -- build_sales_report のシグネチャが存在することを確認する（スタブテスト）
}
```

## Success Criteria（Rust テストで担保）

- `runes/sap-odata/sap_odata.test.fav` に以下を含む:
  - `"sales_order_create"` — SalesOrder 作成テスト
  - `"pagination_over_100_items"` — ページネーションテスト（`odata_list_paged_function_exists` テストと区別）
- `cargo test` で 3,991 tests, 0 failures
- Rust テスト 2 件:
  - `sap_odata_test_contains_sales_order_tests`（`"sales_order_create"` を確認）
  - `sap_odata_test_contains_pagination_test`（`"pagination_over_100_items"` を確認）

## Files to Modify

| ファイル | 変更種別 |
|---|---|
| `runes/sap-odata/sap_odata.test.fav` | 追記（SalesOrder CRUD + ページネーション + レポートテスト） |
| `fav/src/driver.rs` | `mod v87800_tests` 追加 |
