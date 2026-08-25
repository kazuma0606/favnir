# Plan: v87.9.0 — 安定化・コードフリーズ

## 実装ステップ

### Step 1: `fav/src/driver.rs` に `mod v87900_tests` を追加

`mod v87800_tests { ... }` の直後に追加:

```rust
#[cfg(test)]
mod v87900_tests {
    #[test]
    fn sap_sales_order_crud_covered() {
        let content = std::fs::read_to_string("../runes/sap-odata/sales_order.fav")
            .expect("runes/sap-odata/sales_order.fav should exist");
        assert!(content.contains("public fn sales_orders("), "sales_orders should be defined");
        assert!(content.contains("public fn sales_order_by_id("), "sales_order_by_id should be defined");
        assert!(content.contains("public fn create_sales_order("), "create_sales_order should be defined");
    }
    #[test]
    fn sap_sales_scenario2_report_pipeline_exists() {
        let content = std::fs::read_to_string("../infra/e2e-demo/sap-odata/pipeline.fav")
            .expect("infra/e2e-demo/sap-odata/pipeline.fav should exist");
        assert!(content.contains("sap_odata.build_sales_report("), "pipeline.fav should call sap_odata.build_sales_report");
    }
}
```

### Step 2: `cargo test` で全 pass 確認

3,991 + 2 = 3,993 tests, 0 failures を確認する。

---

**Note**: CHANGELOG / MILESTONE / site MDX 更新は v88.0.0 宣言バージョンでまとめて実施するため、本バージョンでは省略する。
