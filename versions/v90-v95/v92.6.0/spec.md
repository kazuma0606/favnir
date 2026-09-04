# Spec: v92.6.0 — QueryBuilder<T> E2E テストパイプライン

Status: COMPLETE

---

## Background

v92.1.0〜v92.5.0 で `QueryBuilder<T>` 型・fluent チェーン関数・`Page<T>` 型・`fetch_all_pages` スタブ・W060 N+1 lint ルールを構築した。
v92.6.0 は `QueryBuilder<T>` を実際に使った E2E デモパイプラインを `infra/e2e-demo/sap-odata/pipeline_query.fav` として追加し、API の使い方を具体的に示す。

---

## Goals

1. `infra/e2e-demo/sap-odata/pipeline_query.fav` を新規作成する
2. `driver.rs` に `mod v92600_tests`（2 件）を追加する

---

## pipeline_query.fav の内容

既存の `pipeline.fav`（4 シナリオ）に追加するファイルとして、`QueryBuilder<T>` パターンを示す新規シナリオを記述する。

```favnir
-- infra/e2e-demo/sap-odata/pipeline_query.fav — QueryBuilder<T> パターン E2E デモ（v92.6.0）
-- query_builder.fav の公開 API（query / with_filter / with_select / fetch_all_pages）を使用する。
-- fetch_all_pages は v92.4.0 スタブ（Result.err）のため、本デモはパターン検証を目的とする。

import rune "sap-odata"

-- シナリオ 5: QueryBuilder<T> を使ったページネーション取得（v92.6.0）
-- with_filter / with_select チェーンで型安全に OData クエリを組み立て、
-- fetch_all_pages で全ページを自動取得するパターンを示す。
-- NOTE: fetch_all_pages は現在スタブ（v92.4.0）。実装完了後に本デモが実際に動作する。
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

### 設計メモ

- `bind q <- ...` 再束縛は E0018 違反のため `q1` / `q2` / `q3` を使う
- `fetch_all_pages` は v92.4.0 スタブ（`Result.err("not yet implemented")`）。fetcher 引数もスタブとして `fn(c, b) { Result.err("...") }` を渡す
- `ctx.s3.put_object(bucket, key, content)` の第3引数は `String`（`List.length` は `Int` のため `Json.encode` を使う）
- `ctx.sap.business_partners_page` メソッドは未実装（v92.4.0 で延期）。v93.0.0 以降で追加予定

---

## Files to Modify / Create

| ファイル | 変更内容 |
|---|---|
| `infra/e2e-demo/sap-odata/pipeline_query.fav` | 新規作成（QueryBuilder<T> パターンデモ） |
| `fav/src/driver.rs` | `mod v92600_tests` を追加（2 件） |

---

## Success Criteria

- `cargo test` 全 pass: **4,109 tests, 0 failures**（4,107 + 2）
- `infra/e2e-demo/sap-odata/pipeline_query.fav` が存在する
- `pipeline_query.fav` に `fetch_all_pages` が含まれる
- `mod v92600_tests` 内の 2 テストが pass する:
  - `pipeline_query_fav_exists`: ファイルが存在する
  - `pipeline_query_uses_fetch_all_pages`: `fetch_all_pages` が含まれる

---

## Note

> **テスト数**: ロードマップ計画値は 4097（4095+2）だが、v92.5.0 の実測が 4,107 のため、本バージョンは 4,107 + 2 = **4,109** が目標。

> **`fetch_all_pages` スタブ**: v92.4.0 で `Result.err("not yet implemented")` として実装済み。pipeline_query.fav はパターンのデモであり、実際の動作は `fetch_all_pages` の完全実装（v93.x.0 予定）後に有効になる。

> **CHANGELOG 更新**: v93.0.0 宣言時にまとめて行う（本バージョンでは不要）。
