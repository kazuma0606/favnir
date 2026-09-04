# Spec: v92.1.0 — `QueryBuilder<T>` 型定義

Status: 未着手

---

## Background

v92.0.0 で SAP OData Query 1.0 を宣言した。v91.x で定義した個別クエリ型
（`SalesOrderQuery` / `BusinessPartnerQuery` / `MaterialQuery` / `PurchaseOrderQuery` / `JournalEntryQuery`）
は各エンティティに特化しているが、共通構造（select / expand / filter / top / skip）を持つ。

v92.1.0 は、これらを統一する **汎用 `QueryBuilder<T>` 型** を定義する基盤スプリント。
Fluent API（`.with_select()` / `.with_filter()` 等のチェーン）は v92.2.0 以降で追加する。

---

## Goals

1. `runes/sap-odata/query_builder.fav`（新規）に `public type QueryBuilder<T>` を定義する
2. `query<T>()` 初期化関数（全フィールド `Option.none()`）を定義する
3. `driver.rs` に `mod v92100_tests`（2 件）を追加する

---

## Syntax / API Examples

```favnir
-- runes/sap-odata/query_builder.fav（新規）
use sap_odata.query

-- 汎用クエリビルダー型（v92.1.0）
-- T: クエリ対象エンティティ型（ファントム型パラメータ）
-- 全フィールドは Option; Option.none() は「指定なし（デフォルト）」を意味する
public type QueryBuilder<T> = {
    select_clause: Option<SelectClause<T>>,
    expand_clause: Option<ExpandClause<T>>,
    filter_expr:   Option<FilterExpr<T>>,
    top_n:         Option<Int>,
    skip_n:        Option<Int>,
    order_by:      Option<String>
}

-- 全フィールドを none() で初期化した QueryBuilder を生成するコンストラクタ
-- 使用例: bind q <- query<SalesOrder>()
--         bind q <- { q | top_n: Option.some(50) }
public fn query<T>() -> QueryBuilder<T> {
    QueryBuilder {
        select_clause: Option.none(),
        expand_clause: Option.none(),
        filter_expr:   Option.none(),
        top_n:         Option.none(),
        skip_n:        Option.none(),
        order_by:      Option.none()
    }
}
```

---

## Files to Modify / Create

| ファイル | 変更内容 |
|---|---|
| `runes/sap-odata/query_builder.fav` | **新規作成**。`QueryBuilder<T>` 型・`query<T>()` 関数を定義 |
| `fav/src/driver.rs` | `mod v92100_tests` を追加（2 件） |
| `runes/sap-odata/sap_odata.fav` | **変更なし**。Rune ローダーはディレクトリ内 `.fav` ファイルを自動解決するため、`use sap_odata.query_builder` は `query_builder.fav` が存在するだけで使用可能。v93.0.0 以降で re-export 追加を検討する。 |

---

## Success Criteria

- `cargo test` 全 pass: **4,096 tests, 0 failures**（4,094 + 2）
- `runes/sap-odata/query_builder.fav` が存在する
- `query_builder.fav` に `public type QueryBuilder` が含まれる
- `mod v92100_tests` 内の 2 テストが pass する:
  - `query_builder_file_exists`: `query_builder.fav` が存在する
  - `query_builder_type_defined`: `query_builder.fav` に `QueryBuilder` が含まれる

---

## Note

> **ロードマップのテスト数**: ロードマップ記載の完了条件（4085 + 2 = 4087）は計画値。実測は 4,094 ベース（v92.0.0 実測）→ 4,096。ロードマップ一覧表の修正は v93.0.0 宣言時に実施する。

> **`public` 修飾子**: ロードマップのコード例では `fn query<T>()` と非公開形式で記載されているが、外部パイプラインからの利用を想定し `public fn query<T>()` として定義する。

> **import 設計**: `query_builder.fav` は `use sap_odata.query` をインポートして `SelectClause<T>` / `ExpandClause<T>` / `FilterExpr<T>` を参照する。循環 dep は発生しない（`query.fav` は `query_builder.fav` を参照しない）。

> **CHANGELOG**: v92.1.0 は中間スプリントのため、CHANGELOG.md への記録は v93.0.0 宣言時にまとめて行う。
