# Tasks: v90.4.0 — `Ctx.build` に SAP 設定注入を統合

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,048 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v90300_tests` が存在することを確認する（v90.3.0 完了済みの証拠）
- [x] `runes/sap-odata/mock.fav` に `impl SapClient for MockSapClient` が含まれることを確認する（v90.3.0 完了済みの証拠）
- [x] `runes/sap-odata/client.fav` に `SapODataClient` が存在しないことを確認する（本バージョンで追加するため）

## T1: 既存コードの確認

- [x] `runes/sap-odata/client.fav` の関数一覧を確認する（`odata_get` / `odata_list` 等）
- [x] `runes/sap-odata/sap_odata.fav` の `business_partners` 等の関数シグネチャを確認する（委譲先）
- [x] `runes/ctx/ctx.fav` の現状を確認する（`Ctx.build` 追加場所の特定）

## T2: `SapODataClient` を `runes/sap-odata/client.fav` に追加

- [x] `SapODataClient = { config: SapConfig }` レコード型を追加する
- [x] `impl SapClient for SapODataClient { ... }` ブロックを追加する（5 メソッド）
  - [x] `business_partners`: `business_partner.business_partners(ctx.config, filter)` に委譲する
  - [x] `business_partner_by_id`: `business_partner.business_partner_by_id(ctx.config, id, false)` に委譲する
  - [x] `sales_orders`: `sales_order.sales_orders(ctx.config, filter)` に委譲する
  - [x] `materials`: `material.materials(ctx.config, filter)` に委譲する
  - [x] `journal_entries`: `journal_entry.journal_entries(ctx.config, filter)` に委譲する
- [x] コメントスタイルが `--` であることを確認する（`client.fav` 既存スタイルに統一）

## T3: `Ctx.build` を `runes/ctx/ctx.fav` に追加

- [x] `ctx.fav` に `public fn Ctx.build() -> Result<AppCtx, String>` を追加する
- [x] `sap_config_from_env()` 呼び出しで `SapODataClient` を生成し `sap` フィールドに設定する
- [x] コメントスタイルが `//` であることを確認する（`ctx.fav` 既存スタイルに統一）

## T4: `mod v90400_tests` を `driver.rs` に追加

- [x] `mod v90300_tests { ... }` の直後に `#[cfg(test)] mod v90400_tests { ... }` を追加する
- [x] `ctx_build_integrates_sap` テストを実装する（`ctx.fav` に `Ctx.build` と `sap` が含まれることを確認）
- [x] `sap_odata_client_impl_exists` テストを実装する（`client.fav` に `impl SapClient for SapODataClient` が含まれることを確認）

## T5: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,050 tests, 0 failures であることを確認する

## T6: `CHANGELOG.md` に v90.4.0 エントリを追加

- [x] `CHANGELOG.md` の先頭（`## [v90.3.0]` の前）に v90.4.0 エントリを追加する
- [x] `v90.4.0`・`SapODataClient`・`Ctx.build`・テスト数 `4,050` が含まれることを確認する
> 本バージョンは `changelog_has_v90_4_0` Rust テストを含まないため T5 後の追加で問題ない。

> 上記テスト全 pass 後、CI 事前確認（T-last）に進む。

## Note

> 実装完了後は本ファイルの Status を `COMPLETE` に変更し、T0〜T-last の全チェックボックスを `[x]` にすること。

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
