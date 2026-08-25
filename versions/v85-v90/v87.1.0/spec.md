# Spec: v87.1.0 — `SalesOrder` / `SalesOrderItem` 型定義

## Background

v87.0.0 で SAP Master Data 1.0（BusinessPartner CRUD）が完成した。
Sprint 3（SAP Sales 1.0）の開始として、受注伝票エンティティを Favnir の型として定義する。

SAP S/4HANA の `SalesOrder` は、顧客との取引を表す基本エンティティ。
`SalesOrderItem` はその明細行（品目・数量・金額）を保持する。

## Goals

- `runes/sap-odata/sales_order.fav` に `SalesOrder` / `SalesOrderItem` / `SalesOrderStatus` 型を定義する
- Rust テスト 2 件で型の存在を確認する
- Note: `sap_odata.fav` への re-export は v87.2.0（関数追加時）に実施する

## Syntax / API

```favnir
-- runes/sap-odata/sales_order.fav

type SalesOrderStatus = Open | InProcess | Completed | Cancelled

type SalesOrderItem = {
    item_number:  Int,
    material_id:  String,
    description:  String,
    quantity:     Float,
    unit:         String,
    net_amount:   Float,
    currency:     String
}

type SalesOrder = {
    order_id:      String,
    customer_id:   String,
    status:        SalesOrderStatus,
    total_amount:  Float,
    currency:      String,
    sales_org:     String,
    created_at:    String,
    items:         Option<List<SalesOrderItem>>
}
```

## Success Criteria

1. `runes/sap-odata/sales_order.fav` が存在する
2. `SalesOrder` 型（`order_id` フィールドを含む）が定義されている
3. `SalesOrderItem` 型（`item_number` フィールドを含む）が定義されている
4. Rust テスト 2 件が pass: `sales_order_type_defined_in_rune` / `sales_order_item_type_defined_in_rune`
5. `cargo test` 全 pass（3975 + 2 = 3977 tests）

## Files to Modify

- `runes/sap-odata/sales_order.fav` — 新規作成
- `fav/src/driver.rs` — `mod v87100_tests` 追加
