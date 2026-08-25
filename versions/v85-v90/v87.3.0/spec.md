# Spec: v87.3.0 — `sales_order_by_id()` + `$expand=to_Item`

## Background

v87.2.0 で `sales_orders()` リストクエリが完成した。
本バージョンでは単一受注の取得関数 `sales_order_by_id()` を実装する。
`expand_items = true` で `$expand=to_Item` を付与し、明細（SalesOrderItem）を含む
完全な受注を 1 リクエストで取得できるようにする。

`business_partner_by_id()`（v86.3.0）と同パターン。

## Goals

- `runes/sap-odata/sales_order.fav` に `sales_order_by_id()` 関数を追加する
- `expand_items: Bool` フラグで `$expand=to_Item` の付与を制御する
- `runes/sap-odata/sap_odata.fav` に re-export を追加する
- Rust テスト 2 件で関数・`expand_items` の存在を確認する

## Syntax / API

```favnir
-- runes/sap-odata/sales_order.fav に追加（v87.3.0）

-- expand_items = true の場合 $expand=to_Item を付与し明細を含む完全な受注を取得する
public fn sales_order_by_id(
    cfg:          SapConfig,
    order_id:     String,
    expand_items: Bool
) -> Result<SalesOrder, String> {
    Result.err("not implemented")
}
```

```favnir
-- runes/sap-odata/sap_odata.fav に追加

public fn sales_order_by_id(cfg: SapConfig, order_id: String, expand_items: Bool) -> Result<SalesOrder, String> {
    sales_order.sales_order_by_id(cfg, order_id, expand_items)
}
```

## Success Criteria

1. `runes/sap-odata/sales_order.fav` に `fn sales_order_by_id` が定義されている
2. `expand_items` フィールド/パラメータが含まれている
3. `runes/sap-odata/sap_odata.fav` に `sales_order_by_id` ラッパーが追加されている
4. Rust テスト 2 件が pass: `sales_order_by_id_function_exists` / `sales_order_expand_items_in_rune`
5. `cargo test` 全 pass（3979 + 2 = 3981 tests）

## Files to Modify

- `runes/sap-odata/sales_order.fav` — `sales_order_by_id()` 関数を追加
- `runes/sap-odata/sap_odata.fav` — `sales_order_by_id()` re-export を追加
- `fav/src/driver.rs` — `mod v87300_tests` 追加
