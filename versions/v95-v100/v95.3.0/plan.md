# Plan: v95.3.0 — SAP Event Mesh 接続基盤

## 実装順序

### Step 0: ベースライン確認

`cargo test 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,168 であることを確認する。

依存: なし

### Step 1: `event_mesh.fav` 新規作成

`runes/sap-odata/event_mesh.fav` を新規作成する。

内容:
- `SapEventMessage` 型定義（topic: String, payload: String, timestamp: String）
- `SapEventClient` interface（subscribe / receive / publish）

```favnir
-- SAP Event Mesh 接続型定義（v95.3.0〜）
-- AMQP 1.0 プロトコル経由で SAP Event Mesh に接続するための interface 定義
-- ctx パターンに従い AppCtx.sap_event フィールドとして注入される

public type SapEventMessage = {
    topic:     String,
    payload:   String,
    timestamp: String
}

public interface SapEventClient {
    fn subscribe(ctx: SapEventClient, topic: String) -> Result<Unit, String>
    fn receive(ctx: SapEventClient) -> Result<SapEventMessage, String>
    fn publish(ctx: SapEventClient, topic: String, payload: String) -> Result<Unit, String>
}
```

依存: なし

### Step 2: `ctx.fav` 修正 — `AppCtx` に `sap_event` フィールドを追加

`runes/ctx/ctx.fav` の `AppCtx` 型に `sap_event: SapEventClient` フィールドを追加する。

変更箇所:
- `AppCtx` 型定義に `sap_event: SapEventClient` を追加
- ファイルに `use sap_odata.event_mesh` を追加（他の `use` 文と同形式 `use sap_odata.xxx`）

注意:
- `ctx.fav` は `//` コメント形式を使用している（`--` ではなく）
- `Ctx.build()` / `Ctx.mock()` の変更は不要（vm.rs プリミティブが実体を提供するため）

依存: Step 1（`event_mesh.fav` が存在すること）

### Step 3: `effect_catalog.rs` 新規作成

`fav/src/effect_catalog.rs` を新規作成し `"SapEvent"` カタログエントリを追加する。

Rust の `Effect` enum は v35.4.0 で削除済みのため、新規ファイルは定数カタログとして実装する。

```rust
/// SAP Platform Era エフェクトマーカーカタログ（v95.3.0〜）
/// pipeline シグネチャの !SapEvent マーカーを文字列定数として定義する。
pub const SAP_EVENT: &str = "SapEvent";
```

合わせて `fav/src/lib.rs`（または `main.rs`）に `mod effect_catalog;` を追加する。
既存の `mod` 宣言群の中に追加する（アルファベット順または関連グループの末尾）。

依存: なし（Step 1, 2 と並行可能）

### Step 4: `driver.rs` にテストを追加

`fav/src/driver.rs` に `#[cfg(test)] mod v95300_tests` を追加する。

テスト 2 件:
1. `event_mesh_fav_exists` — `runes/sap-odata/event_mesh.fav` が存在する
2. `sap_event_client_interface_defined` — `event_mesh.fav` に `SapEventClient` と `SapEventMessage` が含まれる

依存: Step 1

### Step 5: `cargo test` で全 pass 確認

4,170 tests, 0 failures を確認する。

### Step 6: CHANGELOG / current.md / tasks.md 更新

- `CHANGELOG.md` の先頭に `[v95.3.0]` エントリを追加する
- `versions/current.md` の最新安定版を `v95.3.0` に更新する
- `tasks.md` を COMPLETE に更新する
