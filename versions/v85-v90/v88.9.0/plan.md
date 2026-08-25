# Plan: v88.9.0 — 安定化・コードフリーズ

## 実装ステップ

### Step 1: 安定化確認（コードフリーズ前チェック）

1. `cargo test` を実行し、4,013 tests, 0 failures を確認する
2. 以下のファイルの存在と内容を目視確認する:
   - `runes/sap-odata/material.fav` — `material_by_id` 関数
   - `runes/sap-odata/purchase_order.fav` — `purchase_orders` / `purchase_order_by_id` / `create_purchase_order` 関数
   - `runes/sap-odata/stock.fav` — `detect_stock_shortage` / `format_stock_alerts` 関数
   - `infra/e2e-demo/sap-odata/pipeline.fav` — `check_stock_vs_orders` 関数（Scenario 3）
   - `infra/e2e-demo/sap-odata/terraform/main.tf` — `favnir-sap-e2e-demo` Lambda 構成
3. バグが発見された場合はこのステップで修正する（新機能追加は禁止）

### Step 2: `fav/src/driver.rs` に `mod v88900_tests` を追加

`mod v88800_tests { ... }` の直後に追加:

```rust
#[cfg(test)]
mod v88900_tests {
    #[test]
    fn sap_procurement_material_and_po_covered() {
        let material = std::fs::read_to_string("../runes/sap-odata/material.fav")
            .expect("runes/sap-odata/material.fav should exist");
        assert!(
            material.contains("material_by_id"),
            "material.fav should define material_by_id"
        );
        let po = std::fs::read_to_string("../runes/sap-odata/purchase_order.fav")
            .expect("runes/sap-odata/purchase_order.fav should exist");
        assert!(
            po.contains("purchase_orders"),
            "purchase_order.fav should define purchase_orders"
        );
    }

    #[test]
    fn sap_procurement_scenario3_pipeline_exists() {
        let content = std::fs::read_to_string(
            "../infra/e2e-demo/sap-odata/pipeline.fav",
        )
        .expect("infra/e2e-demo/sap-odata/pipeline.fav should exist");
        assert!(
            content.contains("check_stock_vs_orders"),
            "pipeline.fav should define check_stock_vs_orders (Scenario 3)"
        );
    }
}
```

### Step 3: `cargo test` で全 pass 確認

4,013 + 2 = 4,015 tests, 0 failures を確認する。

---

**Note**: CHANGELOG / MILESTONE / site MDX 更新は v89.0.0 宣言バージョンでまとめて実施するため、本バージョンでは省略する。
**Note**: Cargo.toml のバージョンは v89.0.0 宣言まで `88.0.0` のまま維持する（本バージョンではバンプしない）。
