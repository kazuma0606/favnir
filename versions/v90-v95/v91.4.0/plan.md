# Plan: v91.4.0 — `SalesOrderQuery` + クエリオプション統合

## 実装ステップ

### Step 0: 循環参照チェック

1. `runes/sap-odata/sales_order.fav` に `use sap_odata.query` が存在しないことを確認する
2. `runes/sap-odata/types.fav` の import 構成を確認し、`query.fav` → `sales_order.fav` → `types.fav` の循環が発生しないか判断する
3. 循環が発生する場合は以下の代替案を選択:
   - **代替案 A**: `SalesOrderQuery` を `query.fav` ではなく `types.fav` の末尾に定義する（循環を断ち切る）
   - **代替案 B**: `SapClient` への `sales_orders_query` 追加を v91.5.0 へ延期する

### Step 1: `runes/sap-odata/query.fav` に import を追記（または代替案 A を選択）

循環参照なしの場合: ファイル先頭に `use sap_odata.sales_order` を追記する。

```
use sap_odata.sales_order
```

### Step 2: `SalesOrderQuery` 型を追記

`FilterExpr<T>` 定義ブロックの後ろに追加する。

```favnir
-- 受注クエリオプション（$filter / $select / $expand / $top / $skip の統合）（v91.4.0）
public type SalesOrderQuery = {
    filter: Option<FilterExpr<SalesOrder>>,
    select: Option<SelectClause<SalesOrder>>,
    expand: Option<ExpandClause<SalesOrder>>,
    top:    Option<Int>,
    skip:   Option<Int>
}
```

### Step 3: `sales_order_query()` ビルダーを追記

`SalesOrderQuery` 定義の直後に追加する。

```favnir
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
```

### Step 4: `SapClient` interface に `sales_orders_query` を追加（循環 dep なしの場合）

`runes/sap-odata/types.fav` の `SapClient` interface に以下を追記する。

```favnir
    fn sales_orders_query(ctx: SapClient, query: SalesOrderQuery) -> Result<List<SalesOrder>, String>
```

### Step 5: `SapODataClient` と `MockSapClient` にスタブ追加（循環 dep なしの場合）

**client.fav** (`SapODataClient`):
```favnir
    fn sales_orders_query(ctx: SapODataClient, query: SalesOrderQuery)
        -> Result<List<SalesOrder>, String> {
        Result.err("not implemented")
    }
```

**mock.fav** (`MockSapClient`):
```favnir
    fn sales_orders_query(ctx: MockSapClient, query: SalesOrderQuery)
        -> Result<List<SalesOrder>, String> {
        Result.err("not implemented")
    }
```

### Step 6: `fav/src/driver.rs` に `mod v91400_tests` を追加

`mod v91300_tests { ... }` の直後に追加する。

```rust
#[cfg(test)]
mod v91400_tests {
    // use super::* は不要（std::fs のみ使用）
    #[test]
    fn sales_order_query_type_defined() {
        let content = std::fs::read_to_string("../runes/sap-odata/query.fav")
            .expect("runes/sap-odata/query.fav should exist");
        assert!(
            content.contains("public type SalesOrderQuery"),
            "query.fav should define public type SalesOrderQuery"
        );
    }
    #[test]
    fn sales_order_query_builder_defined() {
        let content = std::fs::read_to_string("../runes/sap-odata/query.fav")
            .expect("runes/sap-odata/query.fav should exist");
        assert!(
            content.contains("public fn sales_order_query"),
            "query.fav should define public fn sales_order_query"
        );
    }
}
```

### Step 7: `cargo test` で全 pass 確認

```bash
cargo test 2>&1 | grep "passed"
# 期待値: 4,075 tests, 0 failures
```

### Note: CHANGELOG について

v91.4.0 は中間スプリントのため、CHANGELOG.md への記録は **v92.0.0 宣言時にまとめて行う**。

### Step 8: CI 事前確認

```bash
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```
