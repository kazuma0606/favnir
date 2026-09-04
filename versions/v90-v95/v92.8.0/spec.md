# Spec: v92.8.0 — サイトドキュメント更新（QueryBuilder<T> パターン）

Status: COMPLETE

---

## Background

v92.1.0〜v92.7.0 で `QueryBuilder<T>` API・`Page<T>` 型・`fetch_all_pages` スタブ・W060 N+1 lint ルール・E2E デモパイプラインを構築した。
v92.8.0 は `site/content/docs/runes/sap-odata.mdx` を更新し、これらの新機能を公式ドキュメントに反映する。

---

## Goals

1. `site/content/docs/runes/sap-odata.mdx` に QueryBuilder<T> 関連セクションを追加する
2. `driver.rs` に `mod v92800_tests`（2 件）を追加する

---

## 追加セクションの内容

### 追加する4セクション

1. **QueryBuilder\<T\> Fluent API** — `query<T>()` + `with_filter` / `with_select` / `with_expand` / `with_top` / `with_skip` / `with_order_by` の使い方
2. **Page\<T\> によるページネーション自動化** — `Page<T>` 型の構造（`items` / `next_link` / `total`）
3. **W060 N+1 lint** — `List.map` / `List.flat_map` コールバック内の `ctx.sap.*` 呼び出し検出と対処法
4. **fetch_all_pages を使った全件同期パターン** — E2E デモ（`pipeline_query.fav`）への参照

### 追加する MDX コンテンツ（概要）

```mdx
---

## QueryBuilder<T> Fluent API（v92.1.0〜）

`query<T>()` でビルダーを生成し、チェーン関数で OData クエリを型安全に組み立てる。

```favnir
import rune "sap-odata"

fn list_jp_partners(ctx: AppCtx) -> Result<List<BusinessPartner>, String> {
    bind q1 <- Result.ok(query<BusinessPartner>())
    bind q2 <- Result.ok(with_filter(q1, Eq("Country", "JP")))
    bind q3 <- Result.ok(with_select(q2, ["BusinessPartner", "BusinessPartnerName"]))
    bind q4 <- Result.ok(with_top(q3, 100))
    ctx.sap.business_partners_query(q4)
}
```

| 関数 | 説明 |
|---|---|
| `query<T>()` | 空の `QueryBuilder<T>` を生成 |
| `with_filter(q, expr)` | `FilterExpr<T>` を設定 |
| `with_select(q, fields)` | `$select` フィールドリストを設定 |
| `with_expand(q, nav_props)` | `$expand` ナビゲーションプロパティを設定 |
| `with_top(q, n)` | `$top` 取得件数上限を設定 |
| `with_skip(q, n)` | `$skip` スキップ件数を設定 |
| `with_order_by(q, field)` | `$orderby` ソートフィールドを設定 |

---

## Page<T> によるページネーション（v92.4.0〜）

`Page<T>` はページ単位の取得結果を表す型。

```favnir
public type Page<T> = {
    items:     List<T>,
    next_link: Option<String>,
    total:     Option<Int>
}
```

`fetch_all_pages` で全ページを自動取得できる（v92.4.0 スタブ、v93.x 以降で完全実装予定）。

---

## W060 N+1 lint（v92.5.0〜）

`List.map` / `List.flat_map` コールバック内で `ctx.sap.*` を呼び出すと W060 警告が発生する。

```favnir
-- W060 検出対象（N+1 パターン）
bind results <- List.map(customer_ids, fn(id) {
    ctx.sap.sales_orders(SalesOrderFilter { sold_to: Option.some(id) })
})

-- 推奨: fetch_all_pages または一括フィルタクエリを使う
bind q1  <- Result.ok(query<SalesOrder>())
bind q2  <- Result.ok(with_filter(q1, Eq("SoldToParty", "CUST-001")))
bind all <- ctx.sap.sales_orders_query(q2)
```

---

## fetch_all_pages パターン（v92.6.0 デモ）

```favnir
fn sync_business_partners_paged(ctx: AppCtx) -> Result<String, String> {
    bind q1  <- Result.ok(query<BusinessPartner>())
    bind q2  <- Result.ok(with_filter(q1, Eq("Country", "JP")))
    bind q3  <- Result.ok(with_select(q2, ["BusinessPartner", "BusinessPartnerName"]))
    bind bps <- fetch_all_pages(ctx, q3, 20, fn(c, b) { Result.err("fetcher: not yet wired") })
    bind enc <- Json.encode(bps)
    bind _   <- ctx.s3.put_object("sap-sync", "business_partners_jp.json", enc)
    Result.ok("synced " ++ Int.to_string(List.length(bps)) ++ " business partners")
}
```
```

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `site/content/docs/runes/sap-odata.mdx` | QueryBuilder<T> / Page<T> / W060 / fetch_all_pages セクションを追加（末尾に追記） |
| `fav/src/driver.rs` | `mod v92800_tests` を追加（2 件） |

---

## Success Criteria

- `cargo test` 全 pass: **4,114 tests, 0 failures**（4,112 + 2）
- `site/content/docs/runes/sap-odata.mdx` に `QueryBuilder` が含まれる
- `site/content/docs/runes/sap-odata.mdx` に `fetch_all_pages` が含まれる
- `mod v92800_tests` 内の 2 テストが pass する:
  - `docs_sap_odata_mentions_query_builder`: MDX に `QueryBuilder` が含まれる
  - `docs_sap_odata_mentions_fetch_all_pages`: MDX に `fetch_all_pages` が含まれる

---

## Note

> **テスト数**: ロードマップ計画値は 4101（4099+2）だが、v92.7.0 の実測が 4,112 のため、本バージョンは 4,112 + 2 = **4,114** が目標。

> **W060 vs W020**: ロードマップの「W020 N+1 lint の説明」は実装上 **W060** が正しい（W020 は v24.4.0 で `check_w020_deprecated_call` として実装済み）。MDX でも W060 として記述する。

> **CHANGELOG 更新**: v93.0.0 宣言時にまとめて行う（本バージョンでは不要）。
