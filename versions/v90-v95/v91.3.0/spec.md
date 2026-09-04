# Spec: v91.3.0 — `FilterExpr<T>` 型定義

## Background

v91.1.0 で `SelectClause<T>`（OData `$select`）、v91.2.0 で `ExpandClause<T>`（OData `$expand`）を定義した。
v91.3.0 では OData `$filter`（フィルタ条件）に対応する `FilterExpr<T>` を `runes/sap-odata/query.fav` に追加する。

フィルタ条件を ADT（代数的データ型）で表現し、`filter_to_odata_string` 関数で OData クエリ文字列に変換できるようにする。
これにより `eq`/`gt`/`lt` 条件や `and`/`or` 結合を型安全に組み立てられる。

## Goals

- `runes/sap-odata/query.fav` に `FilterExpr<T>` ADT を追加する
- `filter_to_odata_string<T>` 変換関数を追加する
- Rust テスト 2 件を追加する（4,070 → 4,072）

> **注意 — テスト数について**: ロードマップ計画値は 4,069 + 2 = 4,071 だが、
> code-reviewer 修正（`select_fields_function_defined` テスト追加）により実測値が 4,070 のため、
> 本 spec の目標は **4,072** とする。

## Syntax / API

```favnir
-- フィルタ条件を表す ADT（OData $filter に対応）（v91.3.0）
-- T はファントム型パラメータ: 対象エンティティ型をコンパイル時に特定するために使う
public type FilterExpr<T> =
    | Eq(String, String)                      -- field eq 'value'
    | Gt(String, String)                      -- field gt value
    | Lt(String, String)                      -- field lt value
    | And(FilterExpr<T>, FilterExpr<T>)       -- (left and right)
    | Or(FilterExpr<T>, FilterExpr<T>)        -- (left or right)

-- FilterExpr を OData $filter 文字列に変換する
-- 使用例: filter_to_odata_string<BusinessPartner>(And(Eq("Country", "JP"), Gt("CreditLimit", "1000")))
--         => "(Country eq 'JP' and CreditLimit gt 1000)"
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

## Success Criteria

- `query.fav` に `FilterExpr` が含まれる（`filter_expr_type_defined` テストで確認）
- `query.fav` に `filter_to_odata_string` が含まれる（`filter_to_odata_string_defined` テストで確認）
- `cargo test` が 4,072 tests, 0 failures で通過する

## Error Codes

- なし（新規型定義のみ、チェッカー変更なし）

## Files to Modify

| ファイル | 操作 | 内容 |
|---|---|---|
| `runes/sap-odata/query.fav` | 追記 | `FilterExpr<T>` ADT + `filter_to_odata_string<T>` 関数 |
| `fav/src/driver.rs` | 追記 | `mod v91300_tests` 2 件 |
