# Tasks: v95.4.0 — イベント駆動 pipeline

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/current.md` の最新安定版が `v95.3.0` であることを確認する
- [x] `runes/sap-odata/event_mesh.fav` が存在することを確認する（v95.3.0 完了済みの証拠）
- [x] `runes/ctx/ctx.fav` の `AppCtx` に `sap_event` フィールドが含まれることを確認する（v95.3.0 完了済みの証拠）
- [x] `fav/src/driver.rs` に `mod v95300_tests` が存在することを確認する（v95.3.0 完了済みの証拠）
- [x] `fav/src/effect_catalog.rs` が存在し `SAP_EVENT` 定数が含まれることを確認する（v95.3.0 完了済みの証拠）
- [x] `cargo test 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,170 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `95.0.0` であることを確認する
  （スプリント内バージョンでは Cargo.toml を更新しない）

## T1: `pipeline_realtime.fav` 新規作成

- [x] `infra/e2e-demo/sap-odata/` ディレクトリが存在することを確認する
- [x] `infra/e2e-demo/sap-odata/pipeline_realtime.fav` を新規作成する
- [x] `import rune "sap-odata"` / `import rune "s3"` を記述する
- [x] `pipeline sync_on_event !SapEvent !S3 { ... }` を実装する
  - `stage Subscribe`: `ctx.sap_event.subscribe(topic)` でトピック購読
  - `stage Process`: `ctx.sap_event.receive()` → JSON デコード → `ctx.s3.put_object(...)` で S3 書き込み

## T2: `driver.rs` にテストを追加

- [x] `mod v95300_tests` の直後に `#[cfg(test)] mod v95400_tests { ... }` を追加する
- [x] `pipeline_realtime_fav_exists` テストを追加する（ファイルが存在する）
  （パス: `std::path::Path::new("../infra/e2e-demo/sap-odata/pipeline_realtime.fav")`）
- [x] `pipeline_realtime_uses_sap_event` テストを追加する
  （`sap_event` / `SapEvent` / `Subscribe` / `Process` の 4 文字列が含まれることを assert）
  （パス: `std::fs::read_to_string("../infra/e2e-demo/sap-odata/pipeline_realtime.fav")`）

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,172 tests, 0 failures であることを確認する

## T4: CHANGELOG / current.md 更新

- [x] `CHANGELOG.md` の先頭に `[v95.4.0]` エントリを追加する
  - `pipeline_realtime.fav` 新規作成（イベント駆動 pipeline デモ）
  - テスト数: 4,172（+2）
- [x] `versions/current.md` の最新安定版を `v95.4.0` に更新する

## T5: tasks.md 更新

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする

## T-last: CI 事前確認（T3 の `cargo test` 全 pass 確認後に実施すること）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
