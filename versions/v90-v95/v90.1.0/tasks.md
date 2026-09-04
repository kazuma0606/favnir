# Tasks: v90.1.0 — `SapClient` interface 定義

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,041 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v90000_tests` が存在することを確認する（v90.0.0 完了済みの証拠）
- [x] `fav/Cargo.toml` の version が `90.0.0` であることを確認する
- [x] `runes/sap-odata/types.fav` が存在することを確認する

## T1: 既存型の確認

- [x] `runes/sap-odata/business_partner.fav` に `BusinessPartner` / `BusinessPartnerFilter` が定義されていることを確認する
- [x] `runes/sap-odata/sales_order.fav` に `SalesOrder` / `SalesOrderFilter` が定義されていることを確認する
- [x] `runes/sap-odata/material.fav` に `Material` / `MaterialFilter` が定義されていることを確認する
- [x] `runes/sap-odata/journal_entry.fav` に `JournalEntry` / `JournalFilter` が定義されていることを確認する

## T2: `SapClient` interface を `types.fav` に追加

- [x] `runes/sap-odata/types.fav` の末尾に `SapClient` interface を追加する
- [x] `business_partners` / `business_partner_by_id` / `sales_orders` / `materials` / `journal_entries` の 5 メソッドが含まれることを確認する
- [x] `interface SapClient {` の形式（`public` なし）で定義されていることを確認する（既存の `HttpClient` / `DbCtx` 等と同じパターン）

## T3: `mod v90100_tests` を `driver.rs` に追加

- [x] `mod v90000_tests { ... }` の直後に `#[cfg(test)] mod v90100_tests { ... }` を追加する
- [x] `sap_client_interface_defined` テストを実装する（`types.fav` に `SapClient` が含まれることを確認）
- [x] `sap_client_has_business_partners_method` テストを実装する（`types.fav` に `business_partners` が含まれることを確認）

## T4: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,043 tests, 0 failures であることを確認する

## T5: `CHANGELOG.md` に v90.1.0 エントリを追加

- [x] `CHANGELOG.md` の先頭（`## [v90.0.0]` の前）に v90.1.0 エントリを追加する
- [x] `v90.1.0`・`SapClient interface`・テスト数 `4,043` が含まれることを確認する

> 上記テスト全 pass 後、CI 事前確認（T-last）に進む。

## Note

> 実装完了後は本ファイルの Status を `COMPLETE` に変更し、T0〜T-last の全チェックボックスを `[x]` にすること。

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
