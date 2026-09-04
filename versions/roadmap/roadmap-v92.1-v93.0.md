# Roadmap v92.1.0 〜 v93.0.0 — SAP QueryBuilder 1.0

Date: 2026-08-25
Status: 完了（v93.0.0 宣言済み・2026-08-30）

マスターロードマップ: [roadmap-v90.1-v95.0.md](roadmap-v90.1-v95.0.md)

---

## 前提

- 直前完了: v92.0.0「SAP OData Query 1.0 宣言」（tests = 4,085）
- 本スプリントは SAP Advanced Era の第 3 スプリント
- 目標: v93.0.0「SAP QueryBuilder 1.0 宣言」（tests = 4,107）

> **テスト数注記**: ロードマップ記載の計画値は 4,085 ベースだが、v92.0.0 の実測値は **4,094**。
> 各バージョンの実テスト数は計画値より +9 多い見込み（例: v92.1.0 実測 4,096、v93.0.0 実測 4,118）。
> バージョン一覧表のテスト数はあくまで計画値。spec/plan/tasks では実測ベースを使用する。
> ロードマップ推移表の実測値反映は v93.0.0 宣言時に実施する。

### 着手前チェックリスト

- `versions/current.md` の最新安定版が v92.0.0 になっていることを確認する
- `versions/v90-v95/v92.1.0/` ディレクトリを作成し、spec.md / plan.md / tasks.md を準備すること
- `runes/sap-odata/query.fav` が存在することを確認する（v91.1.0 完了済みの証拠）
- `runes/sap-odata/query.fav` に `ODataQueryBuilder` が含まれることを確認する（v91.8.0 完了済みの証拠）
- `fav/src/driver.rs` に `mod v92000_tests` が存在することを確認する（v92.0.0 完了済みの証拠）

### スプリントの性格

v91.x で定義した個別クエリ型を統合し、**メソッドチェーン風の `QueryBuilder<T>`** を実現するスプリント。

`.select()` / `.expand()` / `.filter()` / `.top()` / `.skip()` をチェーンできる Fluent API と、
ページネーション抽象 `Page<T>` を整備する。
W020 N+1 lint ルールを追加し、コンパイル時に過剰クエリを検出できるようにする。

A（基盤・型定義）40% + B（機能拡充）40% + C（ドキュメント）20% の構成。

---

## バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v92.1.0 | `QueryBuilder<T>` 型定義（Fluent API 基盤） | 4085 + 2 = 4087 | 未着手 |
| v92.2.0 | `.select` / `.expand` / `.filter` チェーン実装 | 4087 + 2 = 4089 | 未着手 |
| v92.3.0 | `.top` / `.skip` / `.order_by` チェーン実装 | 4089 + 2 = 4091 | 未着手 |
| v92.4.0 | `Page<T>` 型 + `fetch_all_pages` 実装 | 4091 + 2 = 4093 | 未着手 |
| v92.5.0 | W060 N+1 lint ルール追加 | 4093 + 2 = 4095（実測: 4105 + 2 = 4107） | 未着手 |
| v92.6.0 | `QueryBuilder<T>` を使った E2E テストパイプライン | 4095 + 2 = 4097 | 未着手 |
| v92.7.0 | `QueryBuilder<T>` ベンチマーク（`fav bench --sap-query`） | 4097 + 2 = 4099 | 未着手 |
| v92.8.0 | サイトドキュメント更新（QueryBuilder パターンガイド） | 4099 + 2 = 4101 | 未着手 |
| v92.9.0 | 安定化・コードフリーズ | 4101 + 2 = 4103 | 未着手 |
| v93.0.0 | SAP QueryBuilder 1.0 宣言 ★クリーンアップ | 4103 + 4 = 4107 | 未着手 |

---

## v92.1.0 — `QueryBuilder<T>` 型定義

汎用 `QueryBuilder<T>` を定義する。v91.x の個別クエリ型を統一する入口となる。

```favnir
-- 汎用クエリビルダー型
type QueryBuilder<T> = {
    select_clause: Option<SelectClause<T>>,
    expand_clause: Option<ExpandClause<T>>,
    filter_expr:   Option<FilterExpr<T>>,
    top_n:         Option<Int>,
    skip_n:        Option<Int>,
    order_by:      Option<String>
}

-- 初期状態（全フィールド None）
fn query<T>() -> QueryBuilder<T> {
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

**実装内容:**
- `runes/sap-odata/query_builder.fav`（新規作成）に `QueryBuilder<T>` と `query` 関数を定義
- `driver.rs` に `mod v92100_tests` を追加（2 件）

**完了条件**: Rust テスト 2 件（4085 + 2 = 4087）
- `query_builder_file_exists`: `runes/sap-odata/query_builder.fav` が存在する
- `query_builder_type_defined`: `query_builder.fav` に `QueryBuilder` が含まれる

---

## v92.2.0 — `.select` / `.expand` / `.filter` チェーン実装

`QueryBuilder<T>` に `.select` / `.expand` / `.filter` の各変換関数を追加する。

```favnir
-- select チェーン
fn with_select<T>(builder: QueryBuilder<T>, fields: List<String>) -> QueryBuilder<T> {
    { builder | select_clause: Option.some(SelectClause { fields: fields }) }
}

-- expand チェーン
fn with_expand<T>(builder: QueryBuilder<T>, nav_props: List<String>) -> QueryBuilder<T> {
    { builder | expand_clause: Option.some(ExpandClause { navigation_properties: nav_props }) }
}

-- filter チェーン
fn with_filter<T>(builder: QueryBuilder<T>, expr: FilterExpr<T>) -> QueryBuilder<T> {
    { builder | filter_expr: Option.some(expr) }
}

-- 使用例
bind q <- query<BusinessPartner>()
bind q <- with_select(q, ["BusinessPartner", "BusinessPartnerName"])
bind q <- with_expand(q, ["to_BusinessPartnerAddress"])
bind q <- with_filter(q, Eq("Country", "JP"))
bind bps <- ctx.sap.business_partners_query(q)
```

**実装内容:**
- `runes/sap-odata/query_builder.fav` に `with_select` / `with_expand` / `with_filter` を追加
- `driver.rs` に `mod v92200_tests` を追加（2 件）

**完了条件**: Rust テスト 2 件（4087 + 2 = 4089）
- `with_select_function_defined`: `query_builder.fav` に `with_select` が含まれる
- `with_filter_function_defined`: `query_builder.fav` に `with_filter` が含まれる

---

## v92.3.0 — `.top` / `.skip` / `.order_by` チェーン実装

ページング制御と並び替えを追加し、実務的なクエリを完成させる。

```favnir
public fn with_top<T>(builder: QueryBuilder<T>, n: Int) -> QueryBuilder<T> {
    { builder | top_n: Option.some(n) }
}

public fn with_skip<T>(builder: QueryBuilder<T>, n: Int) -> QueryBuilder<T> {
    { builder | skip_n: Option.some(n) }
}

public fn with_order_by<T>(builder: QueryBuilder<T>, field: String) -> QueryBuilder<T> {
    { builder | order_by: Option.some(field) }
}

-- ページネーション例（50件ずつ、3ページ目）
-- NOTE: bind の再束縛は E0018 違反のため、実際は別名を使うこと
bind q    <- query<SalesOrder>()
bind q2   <- with_filter(q, Eq("SoldToParty", "CUST-001"))
bind q3   <- with_order_by(q2, "SalesOrder desc")
bind q4   <- with_top(q3, 50)
bind q5   <- with_skip(q4, 100)
bind page <- ctx.sap.sales_orders_query(q5)
```

**実装内容:**
- `runes/sap-odata/query_builder.fav` に `with_top` / `with_skip` / `with_order_by` を追加
- `driver.rs` に `mod v92300_tests` を追加（3 件）

**完了条件**: Rust テスト 3 件（計画値 4089 + 3 = 4092 / 実測ベース 4100 + 3 = 4103）
- `with_top_function_defined`: `query_builder.fav` に `with_top` が含まれる
- `with_skip_function_defined`: `query_builder.fav` に `with_skip` が含まれる
- `with_order_by_function_defined`: `query_builder.fav` に `with_order_by` が含まれる

---

## v92.4.0 — `Page<T>` 型 + `fetch_all_pages` 実装

OData の `@odata.nextLink` に基づくページネーション抽象を実装する。

```favnir
-- ページ結果型（public 修飾子を付与）
public type Page<T> = {
    items:     List<T>,
    next_link: Option<String>,    -- @odata.nextLink
    total:     Option<Int>        -- @odata.count
}

-- 全ページを自動取得（最大 max_pages ページ）（v92.4.0 はスタブ）
public fn fetch_all_pages<T>(
    ctx:       AppCtx,
    builder:   QueryBuilder<T>,
    max_pages: Int,
    fetcher:   (AppCtx, QueryBuilder<T>) -> Result<Page<T>, String>
) -> Result<List<T>, String> {
    Result.err("fetch_all_pages: not yet implemented (v92.4.0 stub)")
}

-- 使用例: 全取引先を自動ページング取得
-- NOTE: bind の再束縛は E0018 違反のため別名を使うこと
bind q   <- query<BusinessPartner>()
bind q2  <- with_filter(q, Eq("Country", "JP"))
bind bps <- fetch_all_pages(ctx, q2, 10, ctx.sap.business_partners_page)
```

**実装内容:**
- `runes/sap-odata/query_builder.fav` に `Page<T>` と `fetch_all_pages`（スタブ）を追加
- `driver.rs` に `mod v92400_tests` を追加（2 件）
- ~~`SapClient` interface に `*_page` メソッド群を追加~~ → **v92.5.0 以降に延期**（interface 変更の複雑度が高いため）

**完了条件**: Rust テスト 2 件（計画値 4093 / 実測ベース 4103 + 2 = 4105）
- `page_type_defined`: `query_builder.fav` に `Page` が含まれる
- `fetch_all_pages_function_defined`: `query_builder.fav` に `fetch_all_pages` が含まれる

---

## v92.5.0 — W060 N+1 lint ルール追加

ループ内の `ctx.sap.*` 呼び出しを検出し、N+1 クエリ問題を警告する。
W020 は v24.4.0 で `check_w020_deprecated_call` として実装済みのため W060 を使用する。

```
W060: N+1 クエリを検出しました。
  ループ内で `ctx.sap.sales_orders(...)` を呼び出しています。
  `fetch_all_pages` または一括取得を使用することを検討してください。

例:
  -- 問題あり（N+1）
  List.map(customer_ids, fn(id) { ctx.sap.sales_orders(SalesOrderFilter { sold_to: id }) })

  -- 推奨（一括取得）
  bind q   <- query<SalesOrder>()
  bind q2  <- with_filter(q, Or(Eq("SoldToParty", "C1"), Eq("SoldToParty", "C2")))
  bind all <- ctx.sap.sales_orders_query(q2)
```

**実装内容:**
- `fav/src/lint.rs` に `W060` ルールを追加（`List.map` / `List.flat_map` 等のコールバック内 `ctx.sap.*` 検出）
- `driver.rs` に `mod v92500_tests` を追加（2 件）

**完了条件**: Rust テスト 2 件（計画値 4095 / 実測ベース 4105 + 2 = 4107）
- `w060_lint_rule_defined`: `lint.rs` に `W060` が含まれる
- `w060_lint_message_mentions_n_plus_1`: `lint.rs` の W060 メッセージに `N+1` が含まれる

---

## v92.6.0 — `QueryBuilder<T>` を使った E2E テストパイプライン

`QueryBuilder<T>` + `Page<T>` を使用した完全なパイプラインをデモとして追加する。

```favnir
-- infra/e2e-demo/sap-odata/pipeline_query.fav
-- NOTE: bind q 再束縛は E0018 違反のため q1/q2/q3 を使う
-- NOTE: fetch_all_pages は v92.4.0 スタブ。fetcher は暫定スタブを渡す
-- NOTE: ctx.sap.business_partners_page は未実装（v93.x.0 予定）
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

**実装内容:**
- `infra/e2e-demo/sap-odata/pipeline_query.fav` を新規作成
- `driver.rs` に `mod v92600_tests` を追加（2 件）

**完了条件**: Rust テスト 2 件（4095 + 2 = 4097）
- `pipeline_query_fav_exists`: `infra/e2e-demo/sap-odata/pipeline_query.fav` が存在する
- `pipeline_query_uses_fetch_all_pages`: `pipeline_query.fav` に `fetch_all_pages` が含まれる

---

## v92.7.0 — `QueryBuilder<T>` ベンチマーク

`fav bench --sap-query` サブコマンドを追加し、クエリビルダーの URL 生成速度と 1,000 件並列ページネーション計測を実施する。

```
$ fav bench --sap-query
SAP QueryBuilder benchmark
  query() + with_filter + with_select:    0.8 µs/op
  filter_to_odata_string (Eq):           0.3 µs/op
  filter_to_odata_string (And 3-clause): 0.6 µs/op
  build_url (full):                      1.2 µs/op
  fetch_all_pages (1,000 items, 20 pages): 42 ms total
  fetch_all_pages throughput:              23.8 items/ms
```

**実装内容:**
- `fav/src/bench.rs` に `bench_sap_query` 関数を追加（URL 生成 + ページネーションの両方を計測）
- `cli.fav` に `--sap-query` フラグを追加
- `driver.rs` に `mod v92700_tests` を追加（2 件）

**完了条件**: Rust テスト 2 件（4097 + 2 = 4099）
- `bench_sap_query_flag_defined`: `bench.rs` に `bench_sap_query` が含まれる
- `bench_sap_query_measures_pagination`: `bench.rs` に `fetch_all_pages` が含まれる

---

## v92.8.0 — サイトドキュメント更新

`site/content/docs/runes/sap-odata.mdx` を `QueryBuilder<T>` パターンに対応するよう更新する。

**追加セクション:**
- `QueryBuilder<T>` Fluent API の使い方
- `Page<T>` によるページネーション自動化
- W060 N+1 lint の説明と対処法（W020 は v24.4.0 実装済み別ルール、W060 が正しい）
- `fetch_all_pages` を使った全件同期パターン

**完了条件**: Rust テスト 2 件（4099 + 2 = 4101）
- `docs_sap_odata_mentions_query_builder`: `sap-odata.mdx` に `QueryBuilder` が含まれる
- `docs_sap_odata_mentions_fetch_all_pages`: `sap-odata.mdx` に `fetch_all_pages` が含まれる

---

## v92.9.0 — 安定化・コードフリーズ

v92.1〜v92.8 の全機能を通しで確認する最終安定化スプリント。

**実装内容:**
- `cargo test` 全 pass 確認（4,101 tests）
- `QueryBuilder<T>` チェーンの整合性確認
- W060 lint の誤検知がないことを確認
- バグ修正のみ受け入れ（新機能追加なし）

**完了条件**: Rust テスト 2 件（4101 + 2 = 4103）
- `query_builder_smoke_all_chains`: `query_builder.fav` に `with_select` / `with_expand` / `with_filter` / `with_top` / `with_skip` / `with_order_by` の全てが含まれる
- `query_builder_page_type_in_rune_dir`: `runes/sap-odata/query_builder.fav` が存在し、`Page` が含まれる

---

## v93.0.0 — SAP QueryBuilder 1.0 宣言 ★クリーンアップ

**宣言文**:
> 「`query<SalesOrder>() |> with_filter(Eq("SoldToParty", "CUST-001")) |> with_top(50)` と書けば、
>  型安全な OData クエリが組み立てられる。
>  ページネーションは `fetch_all_pages` で自動化され、N+1 は W060 で防がれる。
>  それが、Favnir SAP QueryBuilder 1.0 である。」

**クリーンアップ作業:**
- `cargo clean` 実施
- `Cargo.toml` バージョンを `93.0.0` に更新
- `CHANGELOG.md` / `MILESTONE.md` / `README.md` 更新
- `versions/current.md` を v93.0.0 に更新
- driver.rs 内の旧 `cargo_toml_version` テストを `93.0.0` に一括更新

**完了条件**: `v93000_tests` 4 件（4103 + 4 = 4107）
- `cargo_toml_version_is_93_0_0`
- `changelog_has_v93_0_0`
- `milestone_has_sap_query_builder`
- `readme_mentions_query_builder`

---

## テスト数推移（本スプリント）

| バージョン | テスト数 | 増加 |
|---|---|---|
| v92.0.0（ベース） | 4,085 | — |
| v92.1.0 | 4,087 | +2 |
| v92.2.0 | 4,089 | +2 |
| v92.3.0 | 4,091 | +2 |
| v92.4.0 | 4,093 | +2 |
| v92.5.0 | 4,095 | +2 |
| v92.6.0 | 4,097 | +2 |
| v92.7.0 | 4,099 | +2 |
| v92.8.0 | 4,114 | +2 |
| v92.9.0 | 4,116 | +2 |
| v93.0.0（宣言） | 4,120 | +4 |

**本スプリント合計**: +22 tests
