# Spec: v88.4.0 — `purchase_orders()` / `purchase_order_by_id()` クエリ

## Background

v88.3.0 で `PurchaseOrder` / `PurchaseOrderItem` / `PurchaseOrderStatus` 型を `types.fav` に定義した。
本バージョンでは新規ファイル `runes/sap-odata/purchase_order.fav` を作成し、
発注伝票の一覧取得（`purchase_orders()`）と単件取得（`purchase_order_by_id()`）を実装する。

## Goals

1. `runes/sap-odata/purchase_order.fav` を新規作成する
2. `PurchaseOrderFilter` 型を定義する（5 フィールド）
3. `purchase_orders()` / `purchase_order_by_id()` のスタブ関数を追加する
4. `runes/sap-odata/sap_odata.fav` に re-export を追加する（手作業確認）
5. Rust テスト 2 件で関数・型の存在を担保する

## API / Syntax Examples

```favnir
-- runes/sap-odata/purchase_order.fav（新規作成）

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
public fn purchase_order_by_id(cfg: SapConfig, po_number: String, expand_items: Bool) -> Result<PurchaseOrder, String> {
    Result.err("not implemented")
}
```

## Success Criteria（Rust テストで担保）

- `runes/sap-odata/purchase_order.fav` に以下を含む:
  - `"public fn purchase_orders("` — 発注伝票一覧取得関数
  - `"PurchaseOrderFilter"` — 発注フィルタ型
- `cargo test` で 4,005 tests, 0 failures
- Rust テスト 2 件（ロードマップ定義に準拠）:
  - `purchase_orders_function_exists`（`purchase_order.fav` に `"public fn purchase_orders("` を確認）
  - `purchase_order_filter_type_exists`（`purchase_order.fav` に `"PurchaseOrderFilter"` を確認）

## 手作業確認項目（Rust テスト対象外）

- `purchase_order.fav` に `"public fn purchase_order_by_id("` が存在するか
  - Note: `purchase_order_by_id` はロードマップ定義のテスト 2 件に含まれないため手作業確認とする
- `sap_odata.fav` に以下が追加されているか:
  - `use sap_odata.purchase_order`
  - `public type PurchaseOrderFilter = purchase_order.PurchaseOrderFilter`
  - `public fn purchase_orders(...)` ラッパー
  - `public fn purchase_order_by_id(...)` ラッパー
  - Note: `sap_odata.fav` の re-export は薄いラッパーである。`purchase_order.fav` の Rust テストは `sap_odata.fav` の re-export 存在を検証しない。これを認識した上で、v88.2.0 からの方針の一貫性維持のためスコープ外とする

## Files to Modify

| ファイル | 変更種別 |
|---|---|
| `runes/sap-odata/purchase_order.fav` | 新規作成（`PurchaseOrderFilter` + クエリ関数 2 件） |
| `runes/sap-odata/sap_odata.fav` | 追記（`purchase_order` re-export）※手作業確認のみ、Rust テストなし |
| `fav/src/driver.rs` | `mod v88400_tests` 追加 |
