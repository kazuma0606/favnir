# Plan: v88.4.0 — `purchase_orders()` / `purchase_order_by_id()` クエリ

## 実装ステップ

### Step 1: `runes/sap-odata/purchase_order.fav` を新規作成

```favnir
-- 発注伝票クエリ関数（v88.4.0）
use sap_odata.types

public type PurchaseOrderFilter = {
    vendor_id:     Option<String>,
    status:        Option<PurchaseOrderStatus>,
    created_after: Option<String>,
    plant:         Option<String>,
    top:           Option<Int>
}

-- 発注伝票一覧取得（v88.4.0）
public fn purchase_orders(cfg: SapConfig, filter: PurchaseOrderFilter) -> Result<List<PurchaseOrder>, String> {
    Result.err("not implemented")
}

-- 発注伝票単件取得（v88.4.0）
-- expand_items: true のとき $expand=Items を付与する（v88.3.0 の PurchaseOrder.items フィールドを展開）
public fn purchase_order_by_id(cfg: SapConfig, po_number: String, expand_items: Bool) -> Result<PurchaseOrder, String> {
    Result.err("not implemented")
}
```

### Step 2: `runes/sap-odata/sap_odata.fav` を更新（re-export）

`use sap_odata.material` の直後に追加:

```favnir
use sap_odata.purchase_order
```

`material_by_id()` ラッパーの直後に追加:

```favnir
public type PurchaseOrderFilter = purchase_order.PurchaseOrderFilter
public fn purchase_orders(cfg: SapConfig, filter: PurchaseOrderFilter) -> Result<List<PurchaseOrder>, String> {
    purchase_order.purchase_orders(cfg, filter)
}
public fn purchase_order_by_id(cfg: SapConfig, po_number: String, expand_items: Bool) -> Result<PurchaseOrder, String> {
    purchase_order.purchase_order_by_id(cfg, po_number, expand_items)
}
```

Note: T2 は手作業確認（Rust テストの対象外）

### Step 3: `fav/src/driver.rs` に `mod v88400_tests` を追加

`mod v88300_tests { ... }` の直後に追加:

```rust
#[cfg(test)]
mod v88400_tests {
    #[test]
    fn purchase_orders_function_exists() {
        let content = std::fs::read_to_string("../runes/sap-odata/purchase_order.fav")
            .expect("runes/sap-odata/purchase_order.fav should exist");
        assert!(content.contains("public fn purchase_orders("), "purchase_orders function should be defined in purchase_order.fav");
    }
    #[test]
    fn purchase_order_filter_type_exists() {
        let content = std::fs::read_to_string("../runes/sap-odata/purchase_order.fav")
            .expect("runes/sap-odata/purchase_order.fav should exist");
        assert!(content.contains("PurchaseOrderFilter"), "PurchaseOrderFilter should be defined in purchase_order.fav");
    }
}
```

### Step 4: `cargo test` で全 pass 確認

4,003 + 2 = 4,005 tests, 0 failures を確認する。

---

**Note**: CHANGELOG / MILESTONE / site MDX 更新は v89.0.0 宣言バージョンでまとめて実施するため、本バージョンでは省略する。
