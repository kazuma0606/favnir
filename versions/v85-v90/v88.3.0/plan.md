# Plan: v88.3.0 — `PurchaseOrder` / `PurchaseOrderItem` 型定義

## 実装ステップ

### Step 1: `runes/sap-odata/types.fav` に発注伝票型を追加

`SalesOrder` 型の後に追加:

```favnir
-- 発注伝票ステータス（v88.3.0）
public type PurchaseOrderStatus = Open | PartiallyDelivered | Completed | Cancelled

-- 発注明細型（v88.3.0）
public type PurchaseOrderItem = {
    item_number: Int,
    material_id: String,
    quantity:    Float,
    unit:        String,
    net_price:   Float,
    currency:    String,
    plant:       String
}

-- 発注伝票型（v88.3.0）
public type PurchaseOrder = {
    po_number:    String,
    vendor_id:    String,
    status:       PurchaseOrderStatus,
    total_amount: Float,
    currency:     String,
    created_at:   String,
    items:        Option<List<PurchaseOrderItem>>
}
```

### Step 2: `fav/src/driver.rs` に `mod v88300_tests` を追加

`mod v88200_tests { ... }` の直後に追加:

```rust
#[cfg(test)]
mod v88300_tests {
    #[test]
    fn purchase_order_type_defined_in_rune() {
        let content = std::fs::read_to_string("../runes/sap-odata/types.fav")
            .expect("runes/sap-odata/types.fav should exist");
        assert!(content.contains("PurchaseOrder"), "PurchaseOrder should be defined in types.fav");
        assert!(content.contains("PurchaseOrderStatus"), "PurchaseOrderStatus should be defined in types.fav");
    }
    #[test]
    fn purchase_order_item_type_defined_in_rune() {
        let content = std::fs::read_to_string("../runes/sap-odata/types.fav")
            .expect("runes/sap-odata/types.fav should exist");
        assert!(content.contains("PurchaseOrderItem"), "PurchaseOrderItem should be defined in types.fav");
    }
}
```

### Step 3: `cargo test` で全 pass 確認

4,001 + 2 = 4,003 tests, 0 failures を確認する。

---

**Note**: CHANGELOG / MILESTONE / site MDX 更新は v89.0.0 宣言バージョンでまとめて実施するため、本バージョンでは省略する。
