# Spec: v95.5.0 — Deep Insert

## Background

OData v4 の Deep Insert は、親エンティティと子エンティティを 1 回の POST リクエストで同時に作成する機能。
通常の `create_sales_order`（v87.4.0）は `NewSalesOrder.items` を含む形だが、
Deep Insert では OData の `$entity` ネスト構造を明示的に使い、1 リクエストで SalesOrder + Items を作成する。

`runes/sap-odata/sales_order.fav` に `NewSalesOrderWithItems` 型と `create_sales_order_deep` スタブを追加する。

## Goals

1. `NewSalesOrderWithItems` 型を `runes/sap-odata/sales_order.fav` に追加する
2. `create_sales_order_deep` 関数スタブを `runes/sap-odata/sales_order.fav` に追加する
3. `fav/src/driver.rs` に `mod v95500_tests`（2 件）を追加する

## Syntax / API Examples

```favnir
-- Deep Insert 用の入力型（v95.5.0）
-- NewSalesOrder（v87.4.0）と異なり sales_org を持たず、Deep Insert 専用の簡略構造
type NewSalesOrderWithItems = {
    customer_id: String,
    currency:    String,
    items:       List<NewSalesOrderItem>   -- ネストされた Items（Deep Insert）
}

-- 1 リクエストで SalesOrder + Items を作成（Deep Insert）
-- NOTE: v95.5.0 の実装シグネチャは既存スタイルに合わせ cfg: SapConfig を第一引数に取る形式。
--       ctx パターン（ctx.sap.create_sales_order_deep(...)）への移行は Out of Scope（後続バージョンで実施）。
-- @internal: 未実装スタブ。
bind order <- create_sales_order_deep(cfg, NewSalesOrderWithItems {
    customer_id: "C001",
    currency:    "JPY",
    items: [
        NewSalesOrderItem { material_id: "MAT001", quantity: 10.0, unit: "EA" }
    ]
})
```

## Files to Modify

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `runes/sap-odata/sales_order.fav` | 修正 | `NewSalesOrderWithItems` 型 + `create_sales_order_deep` スタブ追加 |
| `fav/src/driver.rs` | 修正 | `mod v95500_tests`（2 件）追加 |

前提: `NewSalesOrderItem` は v87.4.0 で `runes/sap-odata/sales_order.fav` に定義済み

## Success Criteria

- `runes/sap-odata/sales_order.fav` に `NewSalesOrderWithItems` 型が含まれる
- `runes/sap-odata/sales_order.fav` に `create_sales_order_deep` 関数が含まれる
- `cargo test` で 4,174 tests, 0 failures

## Out of Scope（次バージョン以降）

- `create_sales_order_deep` の実際の OData Deep Insert HTTP 実装（後続バージョンで実施）
- `SapClient` interface への `create_sales_order_deep` 追加（後続バージョンで実施）
- `sap_odata.fav` への re-export 追加（後続バージョンで実施）
