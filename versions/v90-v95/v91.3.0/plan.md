# Plan: v91.3.0 — `FilterExpr<T>` 型定義

## 実装ステップ

### Step 1: `runes/sap-odata/query.fav` に `FilterExpr<T>` ADT を追記

`ExpandClause<T>` 定義ブロックの後ろに以下を追加する。

```favnir
-- フィルタ条件を表す ADT（OData $filter に対応）（v91.3.0）
-- T はファントム型パラメータ: 対象エンティティ型をコンパイル時に特定するために使う
public type FilterExpr<T> =
    | Eq(String, String)
    | Gt(String, String)
    | Lt(String, String)
    | And(FilterExpr<T>, FilterExpr<T>)
    | Or(FilterExpr<T>, FilterExpr<T>)
```

### Step 2: `runes/sap-odata/query.fav` に `filter_to_odata_string<T>` を追記

`FilterExpr<T>` 定義の直後に以下を追加する。

```favnir
-- FilterExpr を OData $filter 文字列に変換する
public fn filter_to_odata_string<T>(expr: FilterExpr<T>) -> String {
    match expr {
        | Eq(field, value) -> field ++ " eq '" ++ value ++ "'"
        | Gt(field, value) -> field ++ " gt " ++ value
        | Lt(field, value) -> field ++ " lt " ++ value
        | And(l, r) -> "(" ++ filter_to_odata_string(l) ++ " and " ++ filter_to_odata_string(r) ++ ")"
        | Or(l, r)  -> "(" ++ filter_to_odata_string(l) ++ " or "  ++ filter_to_odata_string(r) ++ ")"
    }
}
```

### Step 3: `fav/src/driver.rs` に `mod v91300_tests` を追加

`mod v91200_tests { ... }` の直後に以下を追加する。

```rust
#[cfg(test)]
mod v91300_tests {
    // use super::* は不要（std::fs のみ使用）
    #[test]
    fn filter_expr_type_defined() {
        let content = std::fs::read_to_string("../runes/sap-odata/query.fav")
            .expect("runes/sap-odata/query.fav should exist");
        assert!(
            content.contains("FilterExpr"),
            "query.fav should define FilterExpr type"
        );
    }
    #[test]
    fn filter_to_odata_string_defined() {
        let content = std::fs::read_to_string("../runes/sap-odata/query.fav")
            .expect("runes/sap-odata/query.fav should exist");
        assert!(
            content.contains("filter_to_odata_string"),
            "query.fav should define filter_to_odata_string function"
        );
    }
}
```

### Step 4: `cargo test` で全 pass 確認

```bash
cargo test 2>&1 | grep "passed"
# 期待値: 4,072 tests, 0 failures
```

### Note: CHANGELOG について

v91.3.0 は中間スプリントのため、CHANGELOG.md への記録は **v92.0.0 宣言時にまとめて行う**。

### Step 5: CI 事前確認

```bash
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```
