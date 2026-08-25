# Spec: v88.3.0 — `PurchaseOrder` / `PurchaseOrderItem` 型定義

## Background

v88.2.0 で `material_by_id()` と `MaterialType` enum の完全化が完了した。
本バージョンでは SAP Procurement 1.0 スプリントの第一歩として、
発注伝票（PurchaseOrder）と発注明細（PurchaseOrderItem）の Favnir 型を定義する。

## Goals

1. `runes/sap-odata/types.fav` に `PurchaseOrderStatus` enum を追加する
2. `runes/sap-odata/types.fav` に `PurchaseOrderItem` 型を追加する
3. `runes/sap-odata/types.fav` に `PurchaseOrder` 型を追加する
4. Rust テスト 2 件で型の存在を担保する

## API / Syntax Examples

```favnir
-- runes/sap-odata/types.fav に追加（SalesOrder 型の後）

public type PurchaseOrderStatus = Open | PartiallyDelivered | Completed | Cancelled

public type PurchaseOrderItem = {
    item_number: Int,
    material_id: String,
    quantity:    Float,
    unit:        String,
    net_price:   Float,
    currency:    String,
    plant:       String
}

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

## Success Criteria（Rust テストで担保）

- `runes/sap-odata/types.fav` に以下を含む:
  - `"PurchaseOrderStatus"` — 発注ステータス enum
  - `"PurchaseOrder"` — 発注伝票型
  - `"PurchaseOrderItem"` — 発注明細型
- `cargo test` で 4,003 tests, 0 failures
- Rust テスト 2 件:
  - `purchase_order_type_defined_in_rune`（`types.fav` に `"PurchaseOrder"` / `"PurchaseOrderStatus"` を確認）
  - `purchase_order_item_type_defined_in_rune`（`types.fav` に `"PurchaseOrderItem"` を確認）

## Files to Modify

| ファイル | 変更種別 |
|---|---|
| `runes/sap-odata/types.fav` | 追記（`PurchaseOrderStatus` / `PurchaseOrderItem` / `PurchaseOrder` 型追加）※`public type` は既存 `SalesOrder` 定義と同じ修飾子スタイルを踏襲 |
| `fav/src/driver.rs` | `mod v88300_tests` 追加 |
