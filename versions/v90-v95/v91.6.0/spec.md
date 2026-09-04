# Spec: v91.6.0 — `MaterialQuery` / `PurchaseOrderQuery` 実装

Status: 未着手

---

## Background

v91.5.0 で `BusinessPartnerQuery` を実装した。v91.6.0 では SAP 調達系エンティティである **品目（Material）** と **購買発注（PurchaseOrder）** のクエリ型を追加する。

`MaterialQuery` は `expand` を持たない（品目はナビゲーション展開を現スプリントではサポートしない）。
`PurchaseOrderQuery` は `expand` を持つ（明細展開 `to_PurchaseOrderItem` 等が実務で多用される）。

### 循環 dep 制約（v91.4.0〜引き継ぎ）

`types.fav` は `query.fav` を import できないため、`SapClient` interface への `materials_query` / `purchase_orders_query` 追加は **v91.8.0 へ引き続き延期**する。

---

## Goals

1. `runes/sap-odata/query.fav` に `MaterialQuery` 型・`material_query()` ビルダーを追加する
2. `runes/sap-odata/query.fav` に `PurchaseOrderQuery` 型・`purchase_order_query()` ビルダーを追加する
3. Rust テスト 2 件を `driver.rs` に追加する

---

## Syntax / API Examples

```favnir
-- 品目クエリオプション（expand なし）
public type MaterialQuery = {
    filter: Option<FilterExpr<Material>>,
    select: Option<SelectClause<Material>>,
    top:    Option<Int>,
    skip:   Option<Int>
}

public fn material_query() -> MaterialQuery {
    MaterialQuery {
        filter: Option.none(),
        select: Option.none(),
        top:    Option.none(),
        skip:   Option.none()
    }
}

-- 購買発注クエリオプション（expand あり）
public type PurchaseOrderQuery = {
    filter: Option<FilterExpr<PurchaseOrder>>,
    select: Option<SelectClause<PurchaseOrder>>,
    expand: Option<ExpandClause<PurchaseOrder>>,
    top:    Option<Int>,
    skip:   Option<Int>
}

public fn purchase_order_query() -> PurchaseOrderQuery {
    PurchaseOrderQuery {
        filter: Option.none(),
        select: Option.none(),
        expand: Option.none(),
        top:    Option.none(),
        skip:   Option.none()
    }
}

-- 使用例: 購買発注を仕入先でフィルタ + 明細展開
bind q <- purchase_order_query()
bind q <- {
    q |
    filter: Option.some(Eq("Supplier", "VEND-001")),
    expand: Option.some(expand_nav<PurchaseOrder>(["to_PurchaseOrderItem"]))
}
-- ctx.sap.purchase_orders_query(q) は v91.8.0 以降で対応予定
```

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `runes/sap-odata/query.fav` | `use sap_odata.material` / `use sap_odata.purchase_order` 追加、`MaterialQuery` / `PurchaseOrderQuery` 型・ビルダー追加 |
| `fav/src/driver.rs` | `mod v91600_tests` 追加（2 件） |

**変更しないファイル（循環 dep 制約）:**
- `runes/sap-odata/types.fav` — SapClient 拡張は v91.8.0 へ延期
- `runes/sap-odata/client.fav` / `mock.fav` — 同上

---

## Success Criteria

- `cargo test` 全 pass: **4,079 tests, 0 failures**（4,077 + 2）
- `runes/sap-odata/query.fav` に `public type MaterialQuery` が存在する
- `runes/sap-odata/query.fav` に `public fn material_query` が存在する
- `runes/sap-odata/query.fav` に `public type PurchaseOrderQuery` が存在する
- `runes/sap-odata/query.fav` に `public fn purchase_order_query` が存在する
- `mod v91600_tests` 内の 2 テストが pass する:
  - `material_query_type_defined`
  - `purchase_order_query_type_defined`

---

## Note

> **CHANGELOG**: v91.6.0 は中間スプリントのため、CHANGELOG.md への記録は v92.0.0 宣言時にまとめて行う。

> **ロードマップのテスト数**: ロードマップ記載の完了条件（4073 + 2 = 4075）は計画値。実測は 4,077 ベース（→ 4,079）。ロードマップ一覧表・推移表の修正は v92.0.0 宣言時に実施する。

> **MaterialQuery に expand なし**: 品目マスタのナビゲーション展開は現スプリントではスコープ外。将来バージョンで追加予定。
