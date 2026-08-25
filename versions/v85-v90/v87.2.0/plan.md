# Plan: v87.2.0 — `SalesOrderFilter` + `sales_orders()` クエリ

## 実装ステップ

### Step 1: `runes/sap-odata/sales_order.fav` を更新

既存コンテンツの先頭コメントを更新し、以下を追加:

1. `use sap_odata.types`（`SapConfig` 参照のため）
2. `public type SalesOrderFilter` レコード型（6 フィールド）
3. `public fn sales_orders(cfg: SapConfig, filter: SalesOrderFilter) -> Result<List<SalesOrder>, String>` スタブ実装

### Step 2: `runes/sap-odata/sap_odata.fav` を更新

`use sap_odata.business_partner` の直後に以下を追加（BusinessPartner パターンと同順）:

```favnir
use sap_odata.sales_order

public type SalesOrderStatus  = sales_order.SalesOrderStatus
public type SalesOrderItem    = sales_order.SalesOrderItem
public type SalesOrder        = sales_order.SalesOrder
public type SalesOrderFilter  = sales_order.SalesOrderFilter
public fn sales_orders(cfg: SapConfig, filter: SalesOrderFilter) -> Result<List<SalesOrder>, String> {
    sales_order.sales_orders(cfg, filter)
}
```

### Step 3: `fav/src/driver.rs` に `mod v87200_tests` を追加

`mod v87100_tests` の直後に以下を追加:

```rust
#[cfg(test)]
mod v87200_tests {
    #[test]
    fn sales_orders_function_exists() {
        let content = std::fs::read_to_string("../runes/sap-odata/sales_order.fav")
            .expect("runes/sap-odata/sales_order.fav should exist");
        assert!(content.contains("fn sales_orders"), "sales_orders function should be defined");
    }
    #[test]
    fn sales_order_filter_type_exists() {
        let content = std::fs::read_to_string("../runes/sap-odata/sales_order.fav")
            .expect("runes/sap-odata/sales_order.fav should exist");
        assert!(content.contains("SalesOrderFilter"), "SalesOrderFilter type should be defined");
    }
}
```

### Step 4: `cargo test` で全 pass 確認（3977 + 2 = 3979）
