# Plan: v88.5.0 — `create_purchase_order()` POST 実装

## 実装ステップ

### Step 1: `runes/sap-odata/purchase_order.fav` に型と関数を追加

`purchase_order_by_id()` の直後に追加:

```favnir
-- 発注明細作成用入力型（v88.5.0）
public type NewPurchaseOrderItem = {
    material_id: String,
    quantity:    Float,
    unit:        String,
    plant:       String
}

-- 発注伝票作成用入力型（v88.5.0）
public type NewPurchaseOrder = {
    vendor_id: String,
    currency:  String,
    items:     List<NewPurchaseOrderItem>
}

-- 発注伝票新規作成（v88.5.0）
-- TODO: implement — OData POST /PurchaseOrders
public fn create_purchase_order(cfg: SapConfig, order: NewPurchaseOrder) -> Result<PurchaseOrder, String> {
    Result.err("not implemented")
}
```

### Step 2: `runes/sap-odata/sap_odata.fav` を更新（re-export）

`purchase_order_by_id()` ラッパーの直後に追加:

```favnir
public type NewPurchaseOrderItem = purchase_order.NewPurchaseOrderItem
public type NewPurchaseOrder     = purchase_order.NewPurchaseOrder
public fn create_purchase_order(cfg: SapConfig, order: NewPurchaseOrder) -> Result<PurchaseOrder, String> {
    purchase_order.create_purchase_order(cfg, order)
}
```

Note: T2 は手作業確認（Rust テストの対象外）

### Step 3: `fav/src/driver.rs` に `mod v88500_tests` を追加

`mod v88400_tests { ... }` の直後に追加:

```rust
#[cfg(test)]
mod v88500_tests {
    #[test]
    fn create_purchase_order_function_exists() {
        let content = std::fs::read_to_string("../runes/sap-odata/purchase_order.fav")
            .expect("runes/sap-odata/purchase_order.fav should exist");
        assert!(content.contains("public fn create_purchase_order("), "create_purchase_order function should be defined in purchase_order.fav");
    }
    #[test]
    fn new_purchase_order_type_exists() {
        let content = std::fs::read_to_string("../runes/sap-odata/purchase_order.fav")
            .expect("runes/sap-odata/purchase_order.fav should exist");
        assert!(content.contains("NewPurchaseOrder"), "NewPurchaseOrder should be defined in purchase_order.fav");
        assert!(content.contains("NewPurchaseOrderItem"), "NewPurchaseOrderItem should be defined in purchase_order.fav");
    }
}
```

### Step 4: `cargo test` で全 pass 確認

4,005 + 2 = 4,007 tests, 0 failures を確認する。

---

**Note**: CHANGELOG / MILESTONE / site MDX 更新は v89.0.0 宣言バージョンでまとめて実施するため、本バージョンでは省略する。
