# Tasks: v91.4.0 — `SalesOrderQuery` + クエリオプション統合

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,073 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v91300_tests` が存在することを確認する（v91.3.0 完了済みの証拠）
- [x] `runes/sap-odata/query.fav` に `FilterExpr` が含まれることを確認する
- [x] `runes/sap-odata/sales_order.fav` に `use sap_odata.query` が存在しないことを確認する（循環参照チェック A）
- [x] `runes/sap-odata/types.fav` の import を確認し、`query.fav` → `sales_order.fav` → `types.fav` の循環が発生しないことを確認する（循環参照チェック B）
  → **循環 dep 検出**: `types.fav` が `query.fav` を import すると循環になるため、T4（SapClient 拡張）を v91.5.0 へ延期（代替案 B）
- [ ] `fav/tmp/hello.fav` が存在することを確認する

## T1: `runes/sap-odata/query.fav` に `use sap_odata.sales_order` を追加

- [x] ファイル先頭に `use sap_odata.sales_order` を追記する（循環 dep なしの場合）
- [x] 循環 dep がある場合は代替案（types.fav への定義移動 or SapClient 拡張を延期）を選択し、以降のタスクを調整する

## T2: `runes/sap-odata/query.fav` に `SalesOrderQuery` 型を追記

- [x] `FilterExpr<T>` 定義の後に `public type SalesOrderQuery = { filter, select, expand, top, skip }` を追加する

## T3: `runes/sap-odata/query.fav` に `sales_order_query()` ビルダーを追記

- [x] `public fn sales_order_query() -> SalesOrderQuery { SalesOrderQuery { ... Option.none() ... } }` を追加する

## T4: `SapClient` interface に `sales_orders_query` を追加（循環 dep なしの場合）

> **SKIP（v91.5.0 へ延期）**: `types.fav` → `query.fav` → `sales_order.fav` → `types.fav` の循環 dep が検出されたため、
> `SapClient.sales_orders_query` の追加は v91.5.0 で対処する（代替案 B を選択）。

- [x] `runes/sap-odata/types.fav` の `SapClient` interface に `sales_orders_query` メソッドを追記する → SKIP（延期）
- [x] `runes/sap-odata/client.fav` の `SapODataClient` impl に `sales_orders_query` スタブを追記する → SKIP（延期）
- [x] `runes/sap-odata/mock.fav` の `MockSapClient` impl に `sales_orders_query` スタブを追記する → SKIP（延期）

## T5: `driver.rs` に `mod v91400_tests` を追加

- [x] `mod v91300_tests { ... }` の直後に `#[cfg(test)] mod v91400_tests { ... }` を追加する
- [x] `sales_order_query_type_defined` テストを実装する（`query.fav` に `"public type SalesOrderQuery"` が含まれることを確認）
- [x] `sales_order_query_builder_defined` テストを実装する（`query.fav` に `"public fn sales_order_query"` が含まれることを確認）

## T6: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "passed"` を実行し、4,075 tests, 0 failures であることを確認する

> 上記テスト全 pass 後、CI 事前確認（T-last）に進む。

## Note

> 実装完了後は本ファイルの Status を `COMPLETE` に変更し、T0〜T-last の全チェックボックスを `[x]` にすること。

> **CHANGELOG について**: v91.4.0 は中間スプリントのため、CHANGELOG.md への記録は **v92.0.0 宣言時にまとめて行う**。

> **ロードマップのテスト数**: ロードマップ本文（4069 + 2 = 4071）は計画値。実測は 4,073 ベース（→ 4,075）。
> ロードマップ一覧表・推移表の修正は v92.0.0 宣言時に実施する。

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
