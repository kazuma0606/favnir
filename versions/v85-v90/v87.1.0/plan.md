# Plan: v87.1.0 — `SalesOrder` / `SalesOrderItem` 型定義

## 実装ステップ

### Step 1: `runes/sap-odata/sales_order.fav` を新規作成

`SalesOrderStatus` enum + `SalesOrderItem` レコード型 + `SalesOrder` レコード型を定義する。
BusinessPartner パターン（`business_partner.fav`）と同構造で作成する。

### Step 2: `fav/src/driver.rs` に `mod v87100_tests` を追加

`mod v87000_tests` の直後に以下を追加:

```rust
#[cfg(test)]
mod v87100_tests {
    #[test]
    fn sales_order_type_defined_in_rune() {
        let content = std::fs::read_to_string("../runes/sap-odata/sales_order.fav")
            .expect("runes/sap-odata/sales_order.fav should exist");
        assert!(content.contains("order_id"), "SalesOrder should have order_id field");
    }
    #[test]
    fn sales_order_item_type_defined_in_rune() {
        let content = std::fs::read_to_string("../runes/sap-odata/sales_order.fav")
            .expect("runes/sap-odata/sales_order.fav should exist");
        assert!(content.contains("item_number"), "SalesOrderItem should have item_number field");
    }
}
```

### Step 3: `cargo test` で全 pass 確認（3975 + 2 = 3977）
