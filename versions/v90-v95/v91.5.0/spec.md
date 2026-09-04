# Spec: v91.5.0 — `BusinessPartnerQuery` 実装

Status: 未着手

---

## Background

v91.1〜v91.4 で OData クエリ型の基盤（`SelectClause<T>` / `ExpandClause<T>` / `FilterExpr<T>` / `SalesOrderQuery`）を整備した。
v91.5.0 では SAP 最重要エンティティのひとつである **取引先（BusinessPartner）** に特化したクエリ型を実装する。

### v91.4.0 からの引き継ぎ事項

v91.4.0 の T4（`SapClient` interface への `sales_orders_query` 追加）は循環 dep により延期された：

```
query.fav → sales_order.fav → types.fav → (if imports query.fav) → query.fav  ❌
```

`types.fav` は `query.fav` を import できないため、`SapClient` interface に query 型を引数とするメソッドを直接追加できない。
本バージョンでも同じ制約が `business_partners_query` に適用されるため、**SapClient 拡張は引き続き延期**する。

SapClient への `business_partners_query` / `sales_orders_query` 統合は v91.8.0（`ODataQueryBuilder` 実装時）で循環 dep 解消と合わせて対応する。

---

## Goals

1. `runes/sap-odata/query.fav` に `BusinessPartnerQuery` 型を定義する
2. `business_partner_query()` ビルダー関数を追加する
3. Rust テスト 2 件を `driver.rs` に追加する

---

## Syntax / API Examples

```favnir
-- BusinessPartnerQuery 型
public type BusinessPartnerQuery = {
    filter: Option<FilterExpr<BusinessPartner>>,
    select: Option<SelectClause<BusinessPartner>>,
    expand: Option<ExpandClause<BusinessPartner>>,
    top:    Option<Int>,
    skip:   Option<Int>
}

-- デフォルト値を持つビルダー関数
public fn business_partner_query() -> BusinessPartnerQuery {
    BusinessPartnerQuery {
        filter: Option.none(),
        select: Option.none(),
        expand: Option.none(),
        top:    Option.none(),
        skip:   Option.none()
    }
}

-- 使用例: 国コードでフィルタ + 住所ナビを展開
bind q <- business_partner_query()
bind q <- {
    q |
    filter: Option.some(Eq("Country", "JP")),
    expand: Option.some(expand_nav<BusinessPartner>(["to_BusinessPartnerAddress"]))
}
-- ctx.sap.business_partners_query(q) は v91.8.0 以降で対応予定
```

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `runes/sap-odata/query.fav` | `use sap_odata.business_partner` 追加、`BusinessPartnerQuery` 型・`business_partner_query()` 追加 |
| `fav/src/driver.rs` | `mod v91500_tests` 追加（2 件） |

**変更しないファイル（循環 dep 制約）:**
- `runes/sap-odata/types.fav` — `SapClient` interface 拡張は循環 dep のため延期
- `runes/sap-odata/client.fav` — 同上
- `runes/sap-odata/mock.fav` — 同上

---

## Success Criteria

- `cargo test` 全 pass: **4,077 tests, 0 failures**（4,075 + 2）
- `runes/sap-odata/query.fav` に `public type BusinessPartnerQuery` が存在する
- `runes/sap-odata/query.fav` に `public fn business_partner_query` が存在する
- `runes/sap-odata/query.fav` に `use sap_odata.business_partner` が追加されている
- `mod v91500_tests` 内の 2 テストが pass する:
  - `business_partner_query_type_defined`
  - `business_partner_query_builder_defined`

---

## Note

> **CHANGELOG について**: v91.5.0 は中間スプリントのため、CHANGELOG.md への記録は **v92.0.0 宣言時にまとめて行う**。

> **ロードマップのテスト数**: ロードマップ一覧表（4071 + 2 = 4073）は計画値。実測は 4,075 ベース（→ 4,077）。
> ロードマップ一覧表・推移表の修正は v92.0.0 宣言時に実施する。

> **SapClient 延期の記録**: v91.4.0 から引き継いだ `sales_orders_query` および本バージョンの `business_partners_query` の SapClient 拡張は、v91.8.0 で循環 dep 解消と合わせて一括実装する予定。
