# Spec: v91.4.0 — `SalesOrderQuery` + クエリオプション統合

## Background

v91.1.0〜v91.3.0 で `SelectClause<T>`・`ExpandClause<T>`・`FilterExpr<T>` を定義した。
v91.4.0 では受注クエリ専用の複合型 `SalesOrderQuery` を定義し、3 つのクエリ型を統合する。

`SalesOrderQuery` は `filter`・`select`・`expand`・`top`・`skip` を `Option` でまとめた値オブジェクトであり、
`sales_order_query()` ビルダー関数でデフォルト（全 `none`）インスタンスを生成する。

また、ロードマップでは `SapClient` interface に `sales_orders_query` メソッドを追加することが定められている。
ただし `SapClient` は `runes/sap-odata/types.fav` に定義されており、`types.fav` が `query.fav` を import すると
循環依存（`types.fav` → `query.fav` → `sales_order.fav` → `types.fav`）が発生する恐れがある。
実装前に循環参照の有無を確認し、問題があれば `SalesOrderQuery` を `types.fav` 内に定義するか、
`SapClient` 拡張を次スプリントへ延期する。

## Goals

- `runes/sap-odata/query.fav` に `SalesOrderQuery` 型と `sales_order_query()` ビルダーを追加する
- `SapClient` interface（`types.fav`）に `sales_orders_query` メソッドを追加する（循環 dep がない場合）
- 循環 dep がある場合は `SapClient` 拡張を v91.5.0 へ延期し、その旨を tasks.md に記録する
- Rust テスト 2 件を追加する（4,073 → 4,075）

> **注意 — テスト数について**: ロードマップの完了条件テキスト（4069 + 2 = 4071）は計画値。
> code-reviewer 修正等による実測テスト増加により現在のベースは 4,073 のため、目標は **4,075**。
> ロードマップ一覧表・推移表の修正は v92.0.0 宣言時にまとめて行う。

## Syntax / API

```favnir
-- 受注クエリオプション（$filter / $select / $expand / $top / $skip の統合）（v91.4.0）
-- 各フィールドは Option 型; none() は「指定なし（デフォルト）」を意味する
public type SalesOrderQuery = {
    filter: Option<FilterExpr<SalesOrder>>,
    select: Option<SelectClause<SalesOrder>>,
    expand: Option<ExpandClause<SalesOrder>>,
    top:    Option<Int>,
    skip:   Option<Int>
}

-- 全フィールドを none() で初期化したデフォルトクエリを生成するビルダー
public fn sales_order_query() -> SalesOrderQuery {
    SalesOrderQuery {
        filter: Option.none(),
        select: Option.none(),
        expand: Option.none(),
        top:    Option.none(),
        skip:   Option.none()
    }
}

-- 使用例: フィルタ + 上位 50 件
bind q <- sales_order_query()
bind q <- { q | filter: Option.some(Eq("SoldToParty", "CUST-001")), top: Option.some(50) }

-- SapClient 経由の利用（循環 dep なしの場合、v91.4.0 で追加）
-- bind orders <- ctx.sap.sales_orders_query(q)
```

> **Note**: ロードマップのコードサンプルでは型・関数に `public` 修飾子が省略されているが、
> Rune として外部公開するため `public` が正しい（spec が優先する）。

## Success Criteria

- `query.fav` に `SalesOrderQuery` が含まれる（`sales_order_query_type_defined` テストで確認）
- `query.fav` に `sales_order_query` 関数が含まれる（`sales_order_query_builder_defined` テストで確認）
- `cargo test` が 4,075 tests, 0 failures で通過する

## Error Codes

- なし（新規型定義のみ、チェッカー変更なし）

## Files to Modify

| ファイル | 操作 | 内容 |
|---|---|---|
| `runes/sap-odata/query.fav` | 追記 | `use sap_odata.sales_order` import + `SalesOrderQuery` 型 + `sales_order_query()` ビルダー |
| `fav/src/driver.rs` | 追記 | `mod v91400_tests` 2 件 |
| `runes/sap-odata/types.fav` | 追記（条件付き） | `SapClient` に `sales_orders_query(query: SalesOrderQuery)` 追加（循環 dep がない場合のみ） |
| `runes/sap-odata/client.fav` | 追記（条件付き） | `SapODataClient` の `sales_orders_query` impl スタブ |
| `runes/sap-odata/mock.fav` | 追記（条件付き） | `MockSapClient` の `sales_orders_query` stub |

> **循環参照が発見された場合**: `SalesOrderQuery` を `types.fav` の末尾に直接定義し、
> `query.fav` からは除外する（`query.fav` → `sales_order.fav` → `types.fav` の循環を回避）。
> または `SapClient` 拡張を v91.5.0 へ延期する。
