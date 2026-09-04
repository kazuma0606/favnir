# Spec: v92.4.0 — `Page<T>` 型 + `fetch_all_pages` 実装

Status: 未着手

---

## Background

v92.3.0 で `with_top` / `with_skip` / `with_order_by` を追加し、クエリオプションが揃った。
v92.4.0 は OData の `@odata.nextLink` に基づくページネーション抽象を実装する。

ページング結果型 `Page<T>` と全ページ自動取得関数 `fetch_all_pages<T>` を `query_builder.fav` に追加する。
`fetch_all_pages` は `fetcher` 関数（1 ページ分取得）を受け取り、`nextLink` を辿って全件収集する。

---

## Goals

1. `runes/sap-odata/query_builder.fav` に `Page<T>` 型と `fetch_all_pages<T>` 関数を追加する
2. `driver.rs` に `mod v92400_tests`（2 件）を追加する

---

## Syntax / API Examples

```favnir
-- ページ結果型（v92.4.0）
-- items:     取得した 1 ページ分のエンティティリスト
-- next_link: 次ページの OData URL（@odata.nextLink）。最終ページでは none()
-- total:     総件数（@odata.count）。サーバーが返さない場合は none()
public type Page<T> = {
    items:     List<T>,
    next_link: Option<String>,
    total:     Option<Int>
}

-- 全ページを自動取得する関数（v92.4.0）
-- max_pages: 最大ページ数（無限ループ防止）
-- fetcher:   1 ページ分のデータを取得する関数（AppCtx + QueryBuilder を受け取り Page を返す）
-- NOTE: v92.4.0 は型定義とシグネチャのスタブ実装。再帰ヘルパー（fetch_pages_acc）の
--       完全実装は v92.5.0 以降で行う。
public fn fetch_all_pages<T>(
    ctx:       AppCtx,
    builder:   QueryBuilder<T>,
    max_pages: Int,
    fetcher:   fn(AppCtx, QueryBuilder<T>) -> Result<Page<T>, String>
) -> Result<List<T>, String> {
    Result.err("fetch_all_pages: not yet implemented (v92.4.0 stub)")
}
```

### 使用例（全取引先を自動ページング取得）

```favnir
bind q   <- query<BusinessPartner>()
bind q2  <- with_filter(q, Eq("Country", "JP"))
bind bps <- fetch_all_pages(ctx, q2, 10, ctx.sap.business_partners_page)
```

---

## Files to Modify / Create

| ファイル | 変更内容 |
|---|---|
| `runes/sap-odata/query_builder.fav` | `Page<T>` 型・`fetch_all_pages<T>` 関数（スタブ）を追加 |
| `fav/src/driver.rs` | `mod v92400_tests` を追加（2 件） |
| `runes/sap-odata/query_client.fav` | **変更なし**。`*_page` メソッドの `SapQueryClient` 追加は将来バージョンで実施 |
| `runes/sap-odata/sap_odata.fav` | **変更なし** |

---

## Success Criteria

- `cargo test` 全 pass: **4,105 tests, 0 failures**（4,103 + 2）
- `query_builder.fav` に `public type Page` が含まれる
- `query_builder.fav` に `public fn fetch_all_pages` が含まれる
- `mod v92400_tests` 内の 2 テストが pass する:
  - `page_type_defined`: `query_builder.fav` に `Page` が含まれる
  - `fetch_all_pages_function_defined`: `query_builder.fav` に `fetch_all_pages` が含まれる

---

## Note

> **テスト数**: ロードマップ計画値は 4093（4091+2）だが、v92.3.0 の実測が 4,103 のため、本バージョンは 4,103 + 2 = **4,105** が目標。

> **`public` 修飾子**: ロードマップのコード例は `type`/`fn` 非公開形式だが、外部利用を想定し `public` として定義する。

> **`fetch_all_pages` スタブ**: v92.4.0 は型シグネチャの定義が主目的。再帰ヘルパー `fetch_pages_acc`（nextLink を辿る）の完全実装は v92.5.0 以降。

> **`SapQueryClient *_page` メソッド**: ロードマップは `SapClient interface に *_page メソッド群を追加` と記載しているが、interface 変更（`SapQueryClient` への `sales_orders_page` 等 5 メソッド追加 + `SapODataClient` / `MockSapClient` スタブ）は複雑度が高いため v92.5.0 以降に延期する。ロードマップの v92.4.0 エントリはこの判断を反映して更新済み。

> **`fetcher` 関数型記法**: `fn(AppCtx, QueryBuilder<T>) -> Result<Page<T>, String>` — Favnir の関数型パラメータ記法として `fn(...)` 形式を使用する。ロードマップは `(AppCtx, ...) -> ...` の tuple スタイルで記載しているが、既存 Favnir ファイルでの慣習に従い `fn(...)` 形式を採用する（構文確認が必要な場合は実装時に `Result.err(...)` スタブから始めて段階的に検証する）。

> **`bind` 再束縛（E0018）**: ロードマップの使用例は `bind q <- with_filter(q, ...)` と同名再束縛しているが、E0018 違反。spec の例では別名を使用する。
