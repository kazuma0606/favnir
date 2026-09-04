# Tasks: v95.3.0 — SAP Event Mesh 接続基盤

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/current.md` の最新安定版が `v95.2.0` であることを確認する
- [x] `runes/sap-odata/delta.fav` が存在することを確認する（v95.1.0 完了済みの証拠）
- [x] `runes/sap-odata/client.fav` に `delta_fetch` が含まれることを確認する（v95.2.0 完了済みの証拠）
- [x] `fav/src/driver.rs` に `mod v95200_tests` が存在することを確認する（v95.2.0 完了済みの証拠）
- [x] `cargo test 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,168 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `95.0.0` であることを確認する
  （スプリント内バージョンでは Cargo.toml を更新しない）

## T1: `event_mesh.fav` 新規作成

- [x] `runes/sap-odata/event_mesh.fav` を新規作成する
- [x] `SapEventMessage` 型（topic: String, payload: String, timestamp: String）を定義する
- [x] `SapEventClient` interface（subscribe / receive / publish）を定義する

## T2: `ctx.fav` 修正

- [x] `runes/ctx/ctx.fav` に `use sap_odata.event_mesh` を追加する
- [x] `AppCtx` 型に `sap_event: SapEventClient` フィールドを追加する
- [x] `Ctx.build()` / `Ctx.mock()` の変更が不要であることを確認する

## T3: `effect_catalog.rs` 新規作成

- [x] `fav/src/effect_catalog.rs` を新規作成する（`SAP_EVENT: &str = "SapEvent"` 定数）
- [x] `fav/src/main.rs` に `mod effect_catalog;` を追加する

## T4: `driver.rs` にテストを追加

- [x] `mod v95200_tests` の直後に `#[cfg(test)] mod v95300_tests { ... }` を追加する
- [x] `event_mesh_fav_exists` テストを追加する（`runes/sap-odata/event_mesh.fav` が存在する）
- [x] `sap_event_client_interface_defined` テストを追加する（`event_mesh.fav` に `SapEventClient` / `SapEventMessage` が含まれる）

## T5: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,170 tests, 0 failures であることを確認する

## T6: CHANGELOG / current.md 更新

- [x] `CHANGELOG.md` の先頭に `[v95.3.0]` エントリを追加する
  - `event_mesh.fav` 新規作成（SapEventMessage / SapEventClient）
  - `ctx.fav` AppCtx に `sap_event: SapEventClient` 追加
  - `effect_catalog.rs` 新規作成（SAP_EVENT 定数）
  - テスト数: 4,170（+2）
- [x] `versions/current.md` の最新安定版を `v95.3.0` に更新する

## T7: tasks.md 更新

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする

## T-last: CI 事前確認（T5 の `cargo test` 全 pass 確認後に実施すること）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
