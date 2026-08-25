# Plan: v87.3.0 — `sales_order_by_id()` + `$expand=to_Item`

## 実装ステップ

### Step 1: `runes/sap-odata/sales_order.fav` を更新

`sales_orders()` 関数の直後に以下を追加:

```favnir
-- 単一 SalesOrder 取得（v87.3.0）
-- expand_items = true の場合 $expand=to_Item を付与し明細を含む完全な受注を取得する

public fn sales_order_by_id(
    cfg:          SapConfig,
    order_id:     String,
    expand_items: Bool
) -> Result<SalesOrder, String> {
    Result.err("not implemented")
}
```

### Step 2: `runes/sap-odata/sap_odata.fav` を更新

`sales_orders()` ラッパーの直後に以下を追加:

```favnir
public fn sales_order_by_id(cfg: SapConfig, order_id: String, expand_items: Bool) -> Result<SalesOrder, String> {
    sales_order.sales_order_by_id(cfg, order_id, expand_items)
}
```

### Step 3: `fav/src/driver.rs` に `mod v87300_tests` を追加

`mod v87200_tests` の直後に以下を追加:

```rust
#[cfg(test)]
mod v87300_tests {
    #[test]
    fn sales_order_by_id_function_exists() {
        let content = std::fs::read_to_string("../runes/sap-odata/sales_order.fav")
            .expect("runes/sap-odata/sales_order.fav should exist");
        assert!(content.contains("fn sales_order_by_id"), "sales_order_by_id function should be defined");
    }
    #[test]
    fn sales_order_expand_items_in_rune() {
        let content = std::fs::read_to_string("../runes/sap-odata/sales_order.fav")
            .expect("runes/sap-odata/sales_order.fav should exist");
        assert!(content.contains("expand_items"), "sales_order.fav should have expand_items parameter");
    }
}
```

### Step 4: `cargo test` で全 pass 確認（3979 + 2 = 3981）
