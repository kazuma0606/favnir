# Spec: v87.4.0 — `create_sales_order()` + `NewSalesOrder`

## Background

v87.3.0 で `sales_order_by_id()` 単一取得が完成した。
本バージョンでは受注伝票の新規作成 `create_sales_order()` を実装する。
POST 前に `x-csrf-token` を取得しリクエストヘッダーに付与するパターンは
`create_business_partner()`（v86.4.0）と同様。

## Goals

- `runes/sap-odata/sales_order.fav` に `NewSalesOrderItem` / `NewSalesOrder` 型と `create_sales_order()` 関数を追加する
- `runes/sap-odata/sap_odata.fav` に re-export を追加する
- Rust テスト 2 件で型・関数の存在を確認する

## Syntax / API

```favnir
-- runes/sap-odata/sales_order.fav に追加（v87.4.0）

public type NewSalesOrderItem = {
    material_id: String,
    quantity:    Float,
    unit:        String
}

public type NewSalesOrder = {
    customer_id: String,
    sales_org:   String,
    currency:    String,
    items:       List<NewSalesOrderItem>
}

-- POST 前に x-csrf-token を取得しリクエストヘッダーに付与する
public fn create_sales_order(
    cfg:   SapConfig,
    order: NewSalesOrder
) -> Result<SalesOrder, String> {
    Result.err("not implemented")
}
```

```favnir
-- runes/sap-odata/sap_odata.fav に追加
-- Note: SapConfig は既存の `use sap_odata.types` より利用可能

public type NewSalesOrderItem = sales_order.NewSalesOrderItem
public type NewSalesOrder     = sales_order.NewSalesOrder
public fn create_sales_order(cfg: SapConfig, order: NewSalesOrder) -> Result<SalesOrder, String> {
    sales_order.create_sales_order(cfg, order)
}
```

## Success Criteria

1. `runes/sap-odata/sales_order.fav` に `NewSalesOrder` 型が定義されている
2. `create_sales_order()` 関数が定義されている
3. `runes/sap-odata/sap_odata.fav` に re-export が追加されている
4. Rust テスト 2 件が pass: `create_sales_order_function_exists` / `new_sales_order_type_exists`
5. `cargo test` 全 pass（3981 + 2 = 3983 tests）

## Files to Modify

- `runes/sap-odata/sales_order.fav` — `NewSalesOrderItem` / `NewSalesOrder` 型・`create_sales_order()` 関数を追加
- `runes/sap-odata/sap_odata.fav` — re-export を追加（`NewSalesOrderItem` / `NewSalesOrder` 型 + `create_sales_order()` ラッパー）
- `fav/src/driver.rs` — `mod v87400_tests` 追加
