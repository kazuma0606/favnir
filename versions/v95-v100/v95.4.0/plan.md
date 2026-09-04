# Plan: v95.4.0 — イベント駆動 pipeline

## 実装順序

### Step 0: ベースライン確認

`cargo test 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,170 であることを確認する。

### Step 1: `pipeline_realtime.fav` 新規作成

`infra/e2e-demo/sap-odata/pipeline_realtime.fav` を新規作成する。

内容:
- `use sap_odata.event_mesh` / `use sap_odata.business_partner` のインポート
- `pipeline sync_on_event !SapEvent !S3 { ... }` — 2 ステージ構成
  - `stage Subscribe`: `ctx.sap_event.subscribe(topic)` でトピック購読
  - `stage Process`: `ctx.sap_event.receive()` でメッセージ受信 → JSON デコード → S3 書き込み

既存の `infra/e2e-demo/sap-odata/` ディレクトリが存在することを事前確認する。

依存: なし（`event_mesh.fav` は v95.3.0 で作成済み）

### Step 2: `driver.rs` にテストを追加

`fav/src/driver.rs` に `#[cfg(test)] mod v95400_tests` を追加する。

テスト 2 件:
1. `pipeline_realtime_fav_exists` — `infra/e2e-demo/sap-odata/pipeline_realtime.fav` が存在する
2. `pipeline_realtime_uses_sap_event` — `pipeline_realtime.fav` に `sap_event` と `SapEvent` が含まれる

依存: Step 1

### Step 3: `cargo test` で全 pass 確認

4,172 tests, 0 failures を確認する。

### Step 4: CHANGELOG / current.md / tasks.md 更新

- `CHANGELOG.md` の先頭に `[v95.4.0]` エントリを追加する
- `versions/current.md` の最新安定版を `v95.4.0` に更新する
- `tasks.md` を COMPLETE に更新する
