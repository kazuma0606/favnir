# Spec: v87.2.0 — `SalesOrderFilter` + `sales_orders()` クエリ

## Background

v87.1.0 で `SalesOrder` / `SalesOrderItem` / `SalesOrderStatus` 型定義が完成した。
本バージョンでは `SalesOrderFilter` 型と `sales_orders()` クエリ関数を実装し、
条件付きの受注リスト取得を可能にする。

BusinessPartner の `BusinessPartnerFilter` + `business_partners()`（v86.2.0）と同パターン。
また `sap_odata.fav` エントリポイントに re-export を追加し、
`import rune "sap-odata"` で `sap_odata.sales_orders()` を呼べるようにする。

## Goals

- `runes/sap-odata/sales_order.fav` に `SalesOrderFilter` 型・`sales_orders()` 関数を追加する
- `use sap_odata.types` import を追加する（`SapConfig` 参照のため）
- `runes/sap-odata/sap_odata.fav` に `sales_order` モジュールの re-export を追加する
- Rust テスト 2 件で型・関数の存在を確認する

## Syntax / API

```favnir
-- runes/sap-odata/sales_order.fav に追加

use sap_odata.types   -- SapConfig のインポート

public type SalesOrderFilter = {
    customer_id:    Option<String>,
    status:         Option<SalesOrderStatus>,
    created_after:  Option<String>,
    created_before: Option<String>,
    sales_org:      Option<String>,
    top:            Option<Int>
}

public fn sales_orders(
    cfg:    SapConfig,
    filter: SalesOrderFilter
) -> Result<List<SalesOrder>, String> {
    Result.err("not implemented")
}
```

```favnir
-- runes/sap-odata/sap_odata.fav に追加
-- Note: SapConfig は既存の `use sap_odata.types` より利用可能（追加不要）

use sap_odata.sales_order

public type SalesOrderStatus  = sales_order.SalesOrderStatus
public type SalesOrderItem    = sales_order.SalesOrderItem
public type SalesOrder        = sales_order.SalesOrder
public type SalesOrderFilter  = sales_order.SalesOrderFilter
public fn sales_orders(cfg: SapConfig, filter: SalesOrderFilter) -> Result<List<SalesOrder>, String> {
    sales_order.sales_orders(cfg, filter)
}
```

## Success Criteria

1. `runes/sap-odata/sales_order.fav` に `public type SalesOrderFilter` が定義されている
2. `public fn sales_orders(cfg: SapConfig, ...)` が定義されている
3. `runes/sap-odata/sap_odata.fav` に `use sap_odata.sales_order` が追加されている
4. Rust テスト 2 件が pass: `sales_orders_function_exists` / `sales_order_filter_type_exists`
5. `cargo test` 全 pass（3977 + 2 = 3979 tests）

## Files to Modify

- `runes/sap-odata/sales_order.fav` — `use sap_odata.types`・`SalesOrderFilter` 型・`sales_orders()` 関数を追加
- `runes/sap-odata/sap_odata.fav` — `use sap_odata.sales_order` + re-export 追加
- `fav/src/driver.rs` — `mod v87200_tests` 追加
