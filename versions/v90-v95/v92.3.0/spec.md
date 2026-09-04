# Spec: v92.3.0 — `.top` / `.skip` / `.order_by` チェーン実装

Status: 未着手

---

## Background

v92.2.0 で `with_select` / `with_expand` / `with_filter` を追加し、選択・展開・絞り込みのチェーンが実現した。
v92.3.0 はページング制御（`with_top` / `with_skip`）と並び替え（`with_order_by`）を追加し、
実務的なクエリに必要なすべてのオプションを揃える。

---

## Goals

1. `runes/sap-odata/query_builder.fav` に `with_top` / `with_skip` / `with_order_by` を追加する
2. `driver.rs` に `mod v92300_tests`（3 件）を追加する

---

## Syntax / API Examples

```favnir
-- runes/sap-odata/query_builder.fav に追加（v92.3.0）

-- top チェーン: 取得件数上限を set する
public fn with_top<T>(builder: QueryBuilder<T>, n: Int) -> QueryBuilder<T> {
    { builder | top_n: Option.some(n) }
}

-- skip チェーン: 先頭 n 件をスキップする（ページネーション用）
public fn with_skip<T>(builder: QueryBuilder<T>, n: Int) -> QueryBuilder<T> {
    { builder | skip_n: Option.some(n) }
}

-- order_by チェーン: ソートフィールドを set する（"FieldName asc" / "FieldName desc" 形式）
public fn with_order_by<T>(builder: QueryBuilder<T>, field: String) -> QueryBuilder<T> {
    { builder | order_by: Option.some(field) }
}
```

### 使用例（ページネーション: 50 件ずつ、3 ページ目）

```favnir
bind q    <- query<SalesOrder>()
bind q2   <- with_filter(q, Eq("SoldToParty", "CUST-001"))
bind q3   <- with_order_by(q2, "SalesOrder desc")
bind q4   <- with_top(q3, 50)
bind q5   <- with_skip(q4, 100)
bind page <- ctx.sap.sales_orders_query(q5)
```

---

## Files to Modify / Create

| ファイル | 変更内容 |
|---|---|
| `runes/sap-odata/query_builder.fav` | `with_top` / `with_skip` / `with_order_by` の 3 関数を追加 |
| `fav/src/driver.rs` | `mod v92300_tests` を追加（3 件） |
| `runes/sap-odata/sap_odata.fav` | **変更なし**。Rune ローダーが `query_builder.fav` を自動解決する（v93.0.0 以降で re-export 追加予定） |

---

## Success Criteria

- `cargo test` 全 pass: **4,103 tests, 0 failures**（4,100 + 3）（ロードマップ計画値 4,092 に対し実測ベース +11 のオフセット）
- `query_builder.fav` に `public fn with_top` が含まれる
- `query_builder.fav` に `public fn with_skip` が含まれる
- `query_builder.fav` に `public fn with_order_by` が含まれる
- `mod v92300_tests` 内の 3 テストが pass する:
  - `with_top_function_defined`: `query_builder.fav` に `with_top` が含まれる
  - `with_skip_function_defined`: `query_builder.fav` に `with_skip` が含まれる
  - `with_order_by_function_defined`: `query_builder.fav` に `with_order_by` が含まれる

---

## Note

> **テスト数**: ロードマップ計画値は 4091（4089+2）だが、v92.2.0 の実測が 4,100 のため、本バージョンは 4,100 + 3 = **4,103** が目標。ロードマップの `with_skip` テストは未記載だが、v92.2.0 と同様に 3 関数すべてをテストする（`with_top` / `with_skip` / `with_order_by`）。

> **`public` 修飾子**: ロードマップのコード例は `fn` 非公開形式だが、外部利用を想定し `public fn` として定義する。

> **`bind` 再束縛禁止（E0018）**: ロードマップの使用例は `bind q <- ...` を繰り返しているが、E0018 に違反する。spec の使用例では別名（`q2`/`q3`...）を使用する。

> **`order_by` の型**: `QueryBuilder<T>` の `order_by: Option<String>` フィールドに `"FieldName asc"` / `"FieldName desc"` 形式の文字列を渡す。型安全な `OrderByClause<T>` は将来バージョン予定。
