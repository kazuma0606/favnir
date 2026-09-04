# Tasks: v90.5.0 — `runes/sap-odata/sap_odata.fav` を `ctx.sap.*` スタイルに対応

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,050 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v90400_tests` が存在することを確認する（v90.4.0 完了済みの証拠）
- [x] `runes/ctx/ctx.fav` に `Ctx.build` が含まれることを確認する（v90.4.0 完了済みの証拠）
- [x] `runes/sap-odata/sap_odata.fav` の `business_partners` が `cfg: SapConfig` スタイルであることを確認する（本バージョンで更新するため）

## T1: 現状確認

- [x] `runes/sap-odata/sap_odata.fav` の対象 4 関数（`business_partners` / `sales_orders` / `materials` / `journal_entries`）のシグネチャを確認する
- [x] `sap_odata.fav` の `public fn` の件数を確認し、他に `cfg: SapConfig` を取る公開関数が残っていないことを確認する

## T2: `sap_odata.fav` の 4 関数を更新

- [x] `business_partners` を更新する
  - [x] 旧関数を `business_partners_cfg` にリネームし `-- deprecated` コメントを追加する
  - [x] 新関数 `business_partners(ctx: AppCtx, filter)` を追加し `ctx.sap.business_partners(filter)` に委譲する
- [x] `sales_orders` を更新する
  - [x] 旧関数を `sales_orders_cfg` にリネームし `-- deprecated` コメントを追加する
  - [x] 新関数 `sales_orders(ctx: AppCtx, filter)` を追加し `ctx.sap.sales_orders(filter)` に委譲する
- [x] `materials` を更新する
  - [x] 旧関数を `materials_cfg` にリネームし `-- deprecated` コメントを追加する
  - [x] 新関数 `materials(ctx: AppCtx, filter)` を追加し `ctx.sap.materials(filter)` に委譲する
- [x] `journal_entries` を更新する
  - [x] 旧関数を `journal_entries_cfg` にリネームし `-- deprecated` コメントを追加する
  - [x] 新関数 `journal_entries(ctx: AppCtx, filter)` を追加し `ctx.sap.journal_entries(filter)` に委譲する
- [x] コメントスタイルが `--` であることを確認する（sap_odata.fav 既存スタイルに統一）

## T3: `mod v90500_tests` を `driver.rs` に追加

- [x] `mod v90400_tests { ... }` の直後に `#[cfg(test)] mod v90500_tests { ... }` を追加する
- [x] `sap_odata_fav_uses_app_ctx` テストを実装する（`sap_odata.fav` に `ctx: AppCtx` が含まれることを確認）
- [x] `sap_odata_fav_delegates_to_ctx_sap` テストを実装する（`sap_odata.fav` に `ctx.sap.business_partners` が含まれることを確認）

## T4: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,052 tests, 0 failures であることを確認する

## T5: `CHANGELOG.md` に v90.5.0 エントリを追加

- [x] `CHANGELOG.md` の先頭（`## [v90.4.0]` の前）に v90.5.0 エントリを追加する
- [x] `v90.5.0`・`ctx: AppCtx`・`ctx.sap.*`・`deprecated`・テスト数 `4,052` が含まれることを確認する
> 本バージョンは `changelog_has_v90_5_0` Rust テストを含まないため T4 後の追加で問題ない。

> 上記テスト全 pass 後、CI 事前確認（T-last）に進む。

## Note

> 実装完了後は本ファイルの Status を `COMPLETE` に変更し、T0〜T-last の全チェックボックスを `[x]` にすること。

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
