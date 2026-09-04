# Spec: v91.2.0 — `ExpandClause<T>` 型定義

## Background

v91.1.0 で `SelectClause<T>`（OData `$select`）を定義し、`runes/sap-odata/query.fav` を新規作成した。
v91.2.0 では OData `$expand`（ナビゲーションプロパティ展開）に対応する `ExpandClause<T>` を同ファイルに追加する。

受注に明細（`to_Item`）や取引先（`to_Partner`）を展開するなど、SAP OData の実務クエリで頻出する
ナビゲーション展開を型安全に表現できるようにする。

## Goals

- `runes/sap-odata/query.fav` に `ExpandClause<T>` 型を追加する
- `expand_nav<T>` ヘルパー関数を追加する
- Rust テスト 2 件を追加する（4,067 → 4,069）

> **注意 — テスト数について**: ロードマップ計画値は 4,065 + 2 = 4,067 だが、
> v91.1.0 の実測完了値が 4,067 のため、本 spec の目標は **4,069** とする。

## Syntax / API

```favnir
-- ナビゲーション展開を表す型（OData $expand に対応）
-- navigation_properties: 展開するナビゲーションプロパティ名のリスト
public type ExpandClause<T> = {
    navigation_properties: List<String>
}

-- ナビゲーションプロパティリストから ExpandClause を生成するヘルパー
public fn expand_nav<T>(navigation_properties: List<String>) -> ExpandClause<T> {
    ExpandClause { navigation_properties: navigation_properties }
}

-- 使用例: 受注に明細と取引先を展開
bind expand <- expand_nav<SalesOrder>(["to_Item", "to_Partner"])
-- expand.navigation_properties == ["to_Item", "to_Partner"]

-- ctx.sap を介した利用（v91.4.0 以降に統合）
bind orders <- ctx.sap.sales_orders_with_expand(filter, expand)
```

## Success Criteria

- `query.fav` に `ExpandClause` が含まれる（`expand_clause_type_defined` テストで確認）
- `query.fav` に `expand_nav` が含まれる（`expand_nav_function_defined` テストで確認）
- `cargo test` が 4,069 tests, 0 failures で通過する

## Error Codes

- なし（新規型定義のみ、チェッカー変更なし）

## Files to Modify

| ファイル | 操作 | 内容 |
|---|---|---|
| `runes/sap-odata/query.fav` | 追記 | `ExpandClause<T>` 型 + `expand_nav<T>` ヘルパー |
| `fav/src/driver.rs` | 追記 | `mod v91200_tests` 2 件 |
