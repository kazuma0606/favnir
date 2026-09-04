# Spec: v92.2.0 — `.select` / `.expand` / `.filter` チェーン実装

Status: 未着手

---

## Background

v92.1.0 で汎用 `QueryBuilder<T>` 型と `query<T>()` コンストラクタを定義した。
現時点では全フィールドが `Option.none()` で初期化されるのみで、フィールドを変更する手段がない。

v92.2.0 は `QueryBuilder<T>` に `.select` / `.expand` / `.filter` の各変換関数を追加し、
レコード更新構文（`{ builder | field: value }`）を使った Fluent スタイルのチェーンを実現する。

---

## Goals

1. `runes/sap-odata/query_builder.fav` に `with_select` / `with_expand` / `with_filter` を追加する
2. `driver.rs` に `mod v92200_tests`（2 件）を追加する

---

## Syntax / API Examples

```favnir
-- runes/sap-odata/query_builder.fav に追加（v92.2.0）

-- select チェーン: フィールドリストを SelectClause に変換して set する
public fn with_select<T>(builder: QueryBuilder<T>, fields: List<String>) -> QueryBuilder<T> {
    { builder | select_clause: Option.some(SelectClause { fields: fields }) }
}

-- expand チェーン: ナビゲーションプロパティを ExpandClause に変換して set する
public fn with_expand<T>(builder: QueryBuilder<T>, nav_props: List<String>) -> QueryBuilder<T> {
    { builder | expand_clause: Option.some(ExpandClause { navigation_properties: nav_props }) }
}

-- filter チェーン: FilterExpr をそのまま set する
public fn with_filter<T>(builder: QueryBuilder<T>, expr: FilterExpr<T>) -> QueryBuilder<T> {
    { builder | filter_expr: Option.some(expr) }
}
```

### 使用例

```favnir
bind q <- query<BusinessPartner>()
bind q <- with_select(q, ["BusinessPartner", "BusinessPartnerName"])
bind q <- with_expand(q, ["to_BusinessPartnerAddress"])
bind q <- with_filter(q, Eq("Country", "JP"))
bind bps <- ctx.sap.business_partners_query(q)
```

---

## Files to Modify / Create

| ファイル | 変更内容 |
|---|---|
| `runes/sap-odata/query_builder.fav` | `with_select` / `with_expand` / `with_filter` の 3 関数を追加 |
| `fav/src/driver.rs` | `mod v92200_tests` を追加（2 件） |
| `runes/sap-odata/sap_odata.fav` | **変更なし**。v92.1.0 同様、Rune ローダーが `query_builder.fav` を自動解決する |

---

## Success Criteria

- `cargo test` 全 pass: **4,100 tests, 0 failures**（4,097 + 3）
- `query_builder.fav` に `public fn with_select` が含まれる
- `query_builder.fav` に `public fn with_expand` が含まれる
- `query_builder.fav` に `public fn with_filter` が含まれる
- `mod v92200_tests` 内の 3 テストが pass する:
  - `with_select_function_defined`: `query_builder.fav` に `with_select` が含まれる
  - `with_expand_function_defined`: `query_builder.fav` に `with_expand` が含まれる
  - `with_filter_function_defined`: `query_builder.fav` に `with_filter` が含まれる

---

## Note

> **テスト数**: ロードマップ記載の計画値は 4089（4087+2）だが、v92.1.0 の実測が 4,097 のため、本バージョンは 4,097 + 3 = **4,100** が目標（`with_expand` テストを計画値 +1 件追加）。

> **`public` 修飾子**: ロードマップのコード例では `fn with_select<T>()` と非公開形式だが、外部パイプラインからの利用を想定し `public fn` として定義する。

> **`SelectClause` フィールド名**: `query.fav` の定義は `SelectClause<T> = { fields: List<String> }`、`ExpandClause<T> = { navigation_properties: List<String> }` であることを確認済み。

> **`with_expand` のテスト**: ロードマップは `with_select` と `with_filter` の 2 件のみを完了条件に記載しているが、`with_expand` は `navigation_properties` フィールドを持つ固有の実装パターンであるため、テストを 1 件追加して合計 3 件とする。
