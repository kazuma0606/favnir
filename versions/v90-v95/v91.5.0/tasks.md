# Tasks: v91.5.0 — `BusinessPartnerQuery` 実装

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,075 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v91400_tests` が存在することを確認する（v91.4.0 完了済みの証拠）
- [x] `runes/sap-odata/query.fav` に `public type SalesOrderQuery` が含まれることを確認する
- [x] `runes/sap-odata/query.fav` に `use sap_odata.sales_order` が含まれることを確認する
- [x] `runes/sap-odata/types.fav` が `query.fav` を import していないことを確認する（循環 dep チェック）
  - `grep "use sap_odata.query" runes/sap-odata/types.fav || echo "OK"`
- [x] `runes/sap-odata/business_partner.fav` の先頭 import を確認し、`query.fav` を import していないことを確認する
- [x] `fav/tmp/hello.fav` が存在することを確認する

## T1: `query.fav` に `use sap_odata.business_partner` を追加

- [x] `runes/sap-odata/query.fav` の `use sap_odata.sales_order` の直後に `use sap_odata.business_partner` を追記する

## T2: `query.fav` に `BusinessPartnerQuery` 型を追記

- [x] `SalesOrderQuery` 定義の後に `public type BusinessPartnerQuery = { filter, select, expand, top, skip }` を追加する（各フィールドは `Option<FilterExpr<BusinessPartner>>` 等）

## T3: `query.fav` に `business_partner_query()` ビルダーを追記

- [x] `sales_order_query()` の直後に `public fn business_partner_query() -> BusinessPartnerQuery { ... Option.none() ... }` を追加する

## T4: SapClient interface 拡張（延期）

> **SKIP（v91.8.0 へ延期）**: `types.fav` → `query.fav` 循環 dep 制約により、`SapClient` interface への `business_partners_query` / `sales_orders_query` 追加は v91.8.0 で循環 dep 解消と合わせて実施する。

- [x] 延期決定を記録する（このチェックは「確認済み」として完了扱い）→ SKIP

## T5: `driver.rs` に `mod v91500_tests` を追加

- [x] `mod v91400_tests { ... }` の直後に `#[cfg(test)] mod v91500_tests { ... }` を追加する
- [x] `business_partner_query_type_defined` テストを実装する（`query.fav` に `"public type BusinessPartnerQuery"` が含まれることを確認）
- [x] `business_partner_query_builder_defined` テストを実装する（`query.fav` に `"public fn business_partner_query"` が含まれることを確認）

## T6: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "passed"` を実行し、4,077 tests, 0 failures であることを確認する

> 上記テスト全 pass 後、CI 事前確認（T-last）に進む。

## Note

> 実装完了後は本ファイルの Status を `COMPLETE` に変更し、T0〜T-last の全チェックボックスを `[x]` にすること（T0 の全項目を含む）。

> **CHANGELOG について**: v91.5.0 は中間スプリントのため、CHANGELOG.md への記録は **v92.0.0 宣言時にまとめて行う**。

> **ロードマップのテスト数**: ロードマップ一覧表（4071 + 2 = 4073）は計画値。実測は 4,075 ベース（→ 4,077）。
> ロードマップ一覧表・推移表の修正は v92.0.0 宣言時に実施する。

> **SapClient 延期**: v91.4.0 からの `sales_orders_query` および本バージョンの `business_partners_query` の SapClient 拡張は v91.8.0 で一括対応予定。

> **MDX ドキュメント更新**: `site/content/docs/runes/sap-odata.mdx` の更新は **v92.0.0 宣言時にまとめて実施する**（中間スプリントのため本バージョンでは不要）。

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
