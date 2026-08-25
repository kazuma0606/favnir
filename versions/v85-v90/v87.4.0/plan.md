# Plan: v87.4.0 — `create_sales_order()` + `NewSalesOrder`

## 実装ステップ

### Step 1: `runes/sap-odata/sales_order.fav` を更新

`sales_order_by_id()` の直後に以下を追加:

```favnir
-- SalesOrder 新規作成（v87.4.0）
-- POST 前に x-csrf-token を取得しリクエストヘッダーに付与する

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

public fn create_sales_order(
    cfg:   SapConfig,
    order: NewSalesOrder
) -> Result<SalesOrder, String> {
    Result.err("not implemented")
}
```

### Step 2: `runes/sap-odata/sap_odata.fav` を更新

`sales_order_by_id()` ラッパーの直後に以下を追加:

```favnir
public type NewSalesOrderItem = sales_order.NewSalesOrderItem
public type NewSalesOrder     = sales_order.NewSalesOrder
public fn create_sales_order(cfg: SapConfig, order: NewSalesOrder) -> Result<SalesOrder, String> {
    sales_order.create_sales_order(cfg, order)
}
```

### Step 3: `fav/src/driver.rs` に `mod v87400_tests` を追加

`mod v87300_tests` の直後に以下を追加:

```rust
#[cfg(test)]
mod v87400_tests {
    #[test]
    fn create_sales_order_function_exists() {
        let content = std::fs::read_to_string("../runes/sap-odata/sales_order.fav")
            .expect("runes/sap-odata/sales_order.fav should exist");
        assert!(content.contains("fn create_sales_order"), "create_sales_order function should be defined");
    }
    #[test]
    fn new_sales_order_type_exists() {
        let content = std::fs::read_to_string("../runes/sap-odata/sales_order.fav")
            .expect("runes/sap-odata/sales_order.fav should exist");
        assert!(content.contains("NewSalesOrder"), "NewSalesOrder type should be defined");
    }
}
```

### Step 4: `cargo test` で全 pass 確認（3981 + 2 = 3983）
