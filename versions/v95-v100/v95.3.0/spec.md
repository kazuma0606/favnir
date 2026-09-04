# Spec: v95.3.0 — SAP Event Mesh 接続基盤

## Background

SAP Event Mesh は AMQP 1.0 プロトコルを介してイベント駆動型のシステム連携を実現する SAP BTP のメッセージングサービス。
v95.1.0〜v95.2.0 で確立したポーリング型（OData `$delta`）に加え、プッシュ型（Event Mesh）の接続基盤を追加する。

ctx パターンに従い `SapEventClient` interface として実装し、`AppCtx` に `sap_event: SapEventClient` フィールドを追加する。
`effect SapEvent { ... }` 宣言は行わない（ctx パターン統一方針）。

pipeline シグネチャのエフェクトマーカー（`!SapEvent`）は ctx interface の「型アノテーション」として機能し、
Rust 側の `effect_catalog.rs` に `SapEvent` エントリを追加することで記録する。
なお、Rust の `Effect` enum は v35.4.0 で削除済み（body call 推論に移行）のため、
`effect_catalog.rs` は新規作成する定数カタログファイルとして扱う。

## Goals

1. `SapEventMessage` 型（topic / payload / timestamp）を定義する
2. `SapEventClient` interface（subscribe / receive / publish）を定義する
3. `runes/sap-odata/event_mesh.fav` を新規作成する
4. `runes/ctx/ctx.fav` の `AppCtx` に `sap_event: SapEventClient` フィールドを追加する
5. `fav/src/effect_catalog.rs` を新規作成し `"SapEvent"` をカタログに追加する
6. `fav/src/lib.rs`（または `main.rs`）に `mod effect_catalog;` を追加する

## Syntax / API Examples

```favnir
-- SapEventMessage: Event Mesh から受信したメッセージ
type SapEventMessage = {
    topic:     String,
    payload:   String,
    timestamp: String
}

-- SapEventClient interface（AppCtx のフィールドとして注入される）
public interface SapEventClient {
    fn subscribe(ctx: SapEventClient, topic: String) -> Result<Unit, String>
    fn receive(ctx: SapEventClient) -> Result<SapEventMessage, String>
    fn publish(ctx: SapEventClient, topic: String, payload: String) -> Result<Unit, String>
}

-- 使用例（v95.4.0 のイベント駆動 pipeline で実装予定）
bind _ <- ctx.sap_event.subscribe("sap/s4/BusinessPartner/Changed")
bind msg <- ctx.sap_event.receive()
bind bp  <- Json.decode<BusinessPartner>(msg.payload)
```

```rust
// fav/src/effect_catalog.rs（新規作成）
// SAP Platform Era で導入するエフェクトマーカーのカタログ
pub const SAP_EVENT: &str = "SapEvent";
```

## Files to Modify

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `runes/sap-odata/event_mesh.fav` | 新規作成 | `SapEventMessage` 型 + `SapEventClient` interface |
| `runes/ctx/ctx.fav` | 修正 | `AppCtx` に `sap_event: SapEventClient` フィールド追加 |
| `fav/src/effect_catalog.rs` | 新規作成 | `SAP_EVENT` 定数（`"SapEvent"`） |
| `fav/src/lib.rs` または `main.rs` | 修正 | `mod effect_catalog;` を追加 |
| `fav/src/driver.rs` | 修正 | `mod v95300_tests`（2 件）追加 |

## Success Criteria

- `runes/sap-odata/event_mesh.fav` が存在する
- `event_mesh.fav` に `SapEventMessage` 型定義が含まれる
- `event_mesh.fav` に `SapEventClient` interface が含まれる
- `runes/ctx/ctx.fav` の `AppCtx` に `sap_event` フィールドが含まれる
- `fav/src/effect_catalog.rs` が存在し `"SapEvent"` が含まれる
- `cargo test` で 4,170 tests, 0 failures

## Out of Scope（次バージョン以降）

- `MockSapEventClient` 実装（v95.4.0 で実施）
- イベント駆動 pipeline の実際のステージ実装（v95.4.0 で実施）
- `checker.fav` の exhaustive match 更新（checker.fav に effect match がないため不要と判断、再評価は v95.4.0 で実施）
