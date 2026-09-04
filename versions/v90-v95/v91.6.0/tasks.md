# Tasks: v91.6.0 — `MaterialQuery` / `PurchaseOrderQuery` 実装

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,077 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v91500_tests` が存在することを確認する（v91.5.0 完了済みの証拠）
- [x] `runes/sap-odata/query.fav` に `public type BusinessPartnerQuery` が含まれることを確認する
- [x] `runes/sap-odata/material.fav` が `query.fav` を import していないことを確認する（循環 dep チェック）
- [x] `runes/sap-odata/purchase_order.fav` が `query.fav` を import していないことを確認する（循環 dep チェック）
- [x] `runes/sap-odata/types.fav` が `query.fav` を import していないことを確認する
- [x] `fav/tmp/hello.fav` が存在することを確認する

## T1: `query.fav` に import を追加

- [x] `use sap_odata.business_partner` の直後に `use sap_odata.material` を追記する
- [x] `use sap_odata.material` の直後に `use sap_odata.purchase_order` を追記する

## T2: `query.fav` に `MaterialQuery` 型を追記

- [x] `business_partner_query()` 定義の後に `public type MaterialQuery = { filter, select, top, skip }` を追加する（`expand` フィールドなし）

## T3: `query.fav` に `material_query()` ビルダーを追記

- [x] `MaterialQuery` 型の直後に `public fn material_query() -> MaterialQuery { ... Option.none() ... }` を追加する

## T4: `query.fav` に `PurchaseOrderQuery` 型を追記

- [x] `material_query()` 定義の後に `public type PurchaseOrderQuery = { filter, select, expand, top, skip }` を追加する（`expand` フィールドあり）

## T5: `query.fav` に `purchase_order_query()` ビルダーを追記

- [x] `PurchaseOrderQuery` 型の直後に `public fn purchase_order_query() -> PurchaseOrderQuery { ... Option.none() ... }` を追加する

## T6: SapClient interface 拡張（延期）

> **SKIP（v91.8.0 へ延期）**: 循環 dep 制約により、`SapClient` への `materials_query` / `purchase_orders_query` 追加は v91.8.0 で一括実施する。

- [x] 延期決定を記録する → SKIP

## T7: `driver.rs` に `mod v91600_tests` を追加

- [x] `mod v91500_tests { ... }` の直後に `#[cfg(test)] mod v91600_tests { ... }` を追加する
- [x] `material_query_type_defined` テストを実装する（`query.fav` に `"public type MaterialQuery"` が含まれることを確認）
- [x] `purchase_order_query_type_defined` テストを実装する（`query.fav` に `"public type PurchaseOrderQuery"` が含まれることを確認）

## T8: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,081 tests, 0 failures であることを確認する

## Note

> 実装完了後は本ファイルの Status を `COMPLETE` に変更し、T0〜T-last の全チェックボックスを `[x]` にすること（T0 の全項目を含む）。

> **CHANGELOG**: v91.6.0 は中間スプリントのため、CHANGELOG.md への記録は v92.0.0 宣言時にまとめて行う。

> **ロードマップのテスト数**: ロードマップ記載の完了条件（4073 + 2 = 4075）は計画値。実測は 4,077 ベース（→ 4,079）。ロードマップ修正は v92.0.0 時に実施。

> **MDX ドキュメント更新**: `site/content/docs/runes/sap-odata.mdx` の更新は v92.0.0 宣言時にまとめて実施する。

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
