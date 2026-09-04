# Spec: v92.0.0 — SAP OData Query 1.0 宣言 ★クリーンアップ

Status: 未着手

---

## Background

v91.1.0〜v91.9.0 で SAP OData Query 基盤を段階的に整備した：

| バージョン | 内容 |
|---|---|
| v91.1.0 | `SelectClause<T>` / `select_fields<T>` |
| v91.2.0 | `ExpandClause<T>` / `expand_nav<T>` |
| v91.3.0 | `FilterExpr<T>` ADT / `filter_to_odata_string<T>` |
| v91.4.0 | `SalesOrderQuery` / `sales_order_query()` |
| v91.5.0 | `BusinessPartnerQuery` / `business_partner_query()` |
| v91.6.0 | `MaterialQuery` / `PurchaseOrderQuery` / 各ビルダー |
| v91.7.0 | `JournalEntryQuery`（`fiscal_year: Option<Int>`） / `journal_entry_query()` |
| v91.8.0 | `ODataQueryBuilder<T, Q>` / `build_url` / `SapQueryClient` interface |
| v91.9.0 | 安定化・スモークテスト（`odata_query_smoke_all_query_types` / `odata_filter_expr_serializable`） |

v92.0.0 はこれらの成果を **SAP OData Query 1.0** として宣言するクリーンアップバージョン。

---

## 宣言文

> 「`SapQueryClient` を通じて `sales_orders_query(q)` と書けば、
>  `$filter`・`$select`・`$expand` を型で組み立てた OData クエリが発行できる。
>  誤フィールド指定はコンパイル時に検出される。
>  それが、Favnir SAP OData Query 1.0 である。」

> **Note**: `ctx.sap.sales_orders_query(q)` 構文（`AppCtx` の `SapQueryClient` 統合）は
> 今後のスプリントで対応予定。現バージョンでは `SapQueryClient` として明示的に型注釈した変数経由で利用可能。

---

## Goals

1. `CHANGELOG.md` に v91.1.0〜v92.0.0 の全エントリを追加する
2. `fav/Cargo.toml` のバージョンを `92.0.0` に更新する
3. `MILESTONE.md` に v92.0.0 SAP OData Query 1.0 宣言を追加する
4. `README.md` に OData Query 機能の言及を追加する
5. `versions/current.md` を v92.0.0 に更新する
6. `driver.rs` 内の `"91.0.0"` 文字列を `"92.0.0"` に一括置換する（44 箇所、sed で一括）
7. `driver.rs` に `mod v92000_tests`（4 件）を追加する
8. `cargo clean` でビルド成果物を削除する

---

## Files to Modify / Create

| ファイル | 変更内容 |
|---|---|
| `CHANGELOG.md` | v91.1.0〜v92.0.0 のエントリを先頭に追加 |
| `fav/Cargo.toml` | `version = "92.0.0"` |
| `MILESTONE.md` | v92.0.0 SAP OData Query 1.0 宣言を先頭に追加 |
| `README.md` | OData Query 言及を追加 |
| `versions/current.md` | v92.0.0 に更新 |
| `fav/src/driver.rs` | `cargo_toml_version_is_91_0_0` を更新、`mod v92000_tests` を追加 |

---

## Success Criteria

- `cargo test` 全 pass: **4,094 tests, 0 failures**（4,090 + 4）
- `fav/Cargo.toml` のバージョンが `92.0.0` である
- `CHANGELOG.md` に `v92.0.0` エントリが含まれる
- `MILESTONE.md` に `SAP OData Query 1.0` が含まれる
- `README.md` に `OData Query` または `SapQueryClient` への言及が含まれる
- `mod v92000_tests` 内の 4 テストが pass する:
  - `cargo_toml_version_is_92_0_0`
  - `changelog_has_v92_0_0`
  - `milestone_has_sap_odata_query`
  - `readme_mentions_odata_query`
- `cargo clean` が完了している

---

## Note

> **ロードマップのテスト数**: ロードマップ記載の完了条件（4085 + 4 = 4089）は計画値。実測は 4,090 ベース → 4,094。ロードマップ推移表の実測値反映もこのバージョンで実施する（v91.9.0 引き継ぎ事項）。

> **cargo clean 後の hello.fav**: `cargo clean` は `target/` のみ削除するため `fav/tmp/hello.fav` は影響を受けない。ただし `./target/debug/fav` が消えるため、CI 事前確認は `cargo clean` より前に実施する。
