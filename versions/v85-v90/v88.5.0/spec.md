# Spec: v88.5.0 — `create_purchase_order()` POST 実装

## Background

v88.4.0 で `purchase_orders()` / `purchase_order_by_id()` クエリを実装した。
本バージョンでは発注伝票の新規作成関数 `create_purchase_order()` を追加し、
SAP Procurement 1.0 スプリントの CREATE 操作を完成させる。

## Goals

1. `runes/sap-odata/purchase_order.fav` に `NewPurchaseOrderItem` 型を追加する
2. `runes/sap-odata/purchase_order.fav` に `NewPurchaseOrder` 型を追加する
3. `runes/sap-odata/purchase_order.fav` に `create_purchase_order()` スタブを追加する
4. `runes/sap-odata/sap_odata.fav` に re-export を追加する（手作業確認のみ・Rust テスト対象外、v88.2.0 からの方針）
5. Rust テスト 2 件で関数・型の存在を担保する

## API / Syntax Examples

```favnir
-- runes/sap-odata/purchase_order.fav に追加

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

## Success Criteria（Rust テストで担保）

- `runes/sap-odata/purchase_order.fav` に以下を含む:
  - `"public fn create_purchase_order("` — 発注伝票新規作成関数
  - `"NewPurchaseOrder"` — 作成用入力型
- `cargo test` で 4,007 tests, 0 failures
- Rust テスト 2 件:
  - `create_purchase_order_function_exists`（`purchase_order.fav` に `"public fn create_purchase_order("` を確認）
  - `new_purchase_order_type_exists`（`purchase_order.fav` に `"NewPurchaseOrder"` および `"NewPurchaseOrderItem"` の両方が存在することを確認）

## 手作業確認項目（Rust テスト対象外）

- `sap_odata.fav` に以下が追加されているか:
  - `public type NewPurchaseOrderItem = purchase_order.NewPurchaseOrderItem`
  - `public type NewPurchaseOrder = purchase_order.NewPurchaseOrder`
  - `public fn create_purchase_order(...)` ラッパー
  - Note: `sap_odata.fav` の re-export は薄いラッパーであり、スコープ外テスト方針を維持（v88.2.0 からの方針）

## Files to Modify

| ファイル | 変更種別 |
|---|---|
| `runes/sap-odata/purchase_order.fav` | 追記（`NewPurchaseOrderItem` / `NewPurchaseOrder` 型 + `create_purchase_order()` 関数） |
| `runes/sap-odata/sap_odata.fav` | 追記（re-export）※手作業確認のみ、Rust テストなし |
| `fav/src/driver.rs` | `mod v88500_tests` 追加 |

**Note**: CHANGELOG / MILESTONE / site MDX 更新は v89.0.0 宣言バージョンでまとめて実施する（本バージョンは対象外）
