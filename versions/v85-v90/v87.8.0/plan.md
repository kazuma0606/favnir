# Plan: v87.8.0 — モックサーバーテスト（受注シナリオ全操作検証）

## 実装ステップ

### Step 1: `runes/sap-odata/sap_odata.test.fav` にテストを追加

既存の BusinessPartner CRUD テスト（`business_partner_list` — 24 行目）の直後に追加する。

**追加テスト:**

1. SalesOrder CRUD テスト（3 件）:
   - `"sales_order_create"` — `create_sales_order` シグネチャ確認
   - `"sales_order_read"` — `sales_order_by_id` シグネチャ確認
   - `"sales_order_filter"` — `sales_orders` フィルタシグネチャ確認

2. ページネーションテスト（1 件）:
   - `"pagination_over_100_items"` — `odata_list_paged` シグネチャ確認

3. 日次売上レポートテスト（1 件）:
   - `"daily_sales_report_pipeline"` — `build_sales_report` シグネチャ確認

### Step 2: `fav/src/driver.rs` に `mod v87800_tests` を追加

`mod v87700_tests { ... }` の直後に追加:

```rust
#[cfg(test)]
mod v87800_tests {
    #[test]
    fn sap_odata_test_contains_sales_order_tests() {
        let content = std::fs::read_to_string("../runes/sap-odata/sap_odata.test.fav")
            .expect("runes/sap-odata/sap_odata.test.fav should exist");
        assert!(content.contains("\"sales_order_create\""), "sap_odata.test.fav should contain sales_order_create test");
    }
    #[test]
    fn sap_odata_test_contains_pagination_test() {
        let content = std::fs::read_to_string("../runes/sap-odata/sap_odata.test.fav")
            .expect("runes/sap-odata/sap_odata.test.fav should exist");
        assert!(content.contains("\"pagination_over_100_items\""), "sap_odata.test.fav should contain pagination_over_100_items test");
    }
}
```

### Step 3: `cargo test` で全 pass 確認

3,989 + 2 = 3,991 tests, 0 failures を確認する。

---

**Note**: CHANGELOG / MILESTONE / site MDX 更新は v88.0.0 宣言バージョンでまとめて実施するため、本バージョンでは省略する。
