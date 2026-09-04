# Spec: v95.4.0 — イベント駆動 pipeline

## Background

v95.3.0 で `SapEventClient` interface と `AppCtx.sap_event` フィールドを追加した。
本バージョンでは `ctx.sap_event.*` を活用したイベント駆動 pipeline のデモ実装を
`infra/e2e-demo/sap-odata/pipeline_realtime.fav` として追加する。

pipeline シグネチャの `!SapEvent` は ctx interface マーカーであり、`effect` 宣言は行わない（ctx パターン統一方針）。

## Goals

1. `infra/e2e-demo/sap-odata/pipeline_realtime.fav` を新規作成する
2. `ctx.sap_event.subscribe` / `ctx.sap_event.receive` を使ったイベント駆動 pipeline を実装する
3. `fav/src/driver.rs` に `mod v95400_tests`（2 件）を追加する

## Syntax / API Examples

```favnir
use sap_odata.event_mesh
use sap_odata.business_partner

-- SAP Event Mesh からリアルタイムにビジネスパートナー変更を受け取り S3 に書き込む
-- !SapEvent: ctx.sap_event.* を使用することを示すマーカー
-- !S3: ctx.s3.* を使用することを示すマーカー
pipeline sync_on_event !SapEvent !S3 {
    stage Subscribe {
        bind _ <- ctx.sap_event.subscribe("sap/s4/BusinessPartner/Changed")
    }
    |> stage Process {
        bind msg  <- ctx.sap_event.receive()
        bind bp   <- Json.decode<BusinessPartner>(msg.payload)
        bind json <- Json.encode(bp)
        bind _    <- ctx.s3.put_object("favnir-sap-sync", bp.partner_id, json)
    }
}
```

## Files to Modify

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `infra/e2e-demo/sap-odata/pipeline_realtime.fav` | 新規作成 | イベント駆動 pipeline デモ |
| `fav/src/driver.rs` | 修正 | `mod v95400_tests`（2 件）追加 |

## Success Criteria

- `infra/e2e-demo/sap-odata/pipeline_realtime.fav` が存在する
- `pipeline_realtime.fav` に `sap_event` が含まれる
- `pipeline_realtime.fav` に `SapEvent` マーカーが含まれる
- `pipeline_realtime.fav` に `stage Subscribe` と `stage Process` の両方が含まれる
- `cargo test` で 4,172 tests, 0 failures

## Files to Modify

前提: `runes/sap-odata/event_mesh.fav`（v95.3.0 作成済み）、`fav/src/effect_catalog.rs`（v95.3.0 作成済み）

## Out of Scope（次バージョン以降）

- `MockSapEventClient` の実装（v95.5.0 以降で実施）
- `!SapEvent` マーカーの Rust 側 lineage 解析対応（後続バージョンで実施）
- `subscribe` / `receive` の実際の AMQP 1.0 接続実装（後続バージョンで実施）
- `checker.fav` の effect match 対応: ロードマップ v95.3.0 の「再評価 v95.4.0」指示に対し、
  checker.fav に effect 関連の match が存在しないことを確認済みのため、本バージョンでは対応しないことを決定。
