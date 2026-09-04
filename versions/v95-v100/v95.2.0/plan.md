# Plan: v95.2.0 — `ctx.sap.delta_fetch<T>()`

## 実装ステップ

### Step 1: `types.fav` — `SapClient` interface に `delta_fetch<T>` を追加

`runes/sap-odata/types.fav` の `SapClient` interface（現在 6 メソッド）末尾に追加する。

```favnir
-- 追加するシグネチャ（batch の直後）
fn delta_fetch<T>(ctx: SapClient, entity_set: String, delta_link: Option<String>) -> Result<DeltaResult<T>, String>
```

`DeltaResult<T>` は v95.1.0 で `runes/sap-odata/delta.fav` に定義済み。
`types.fav` に `use sap_odata.delta` を追加して `DeltaResult<T>` を参照可能にする。

**依存**: v95.1.0 の `delta.fav` 完了済み

---

### Step 2: `client.fav` — `use sap_odata.delta` 追加 + `delta_fetch<T>` スタブ実装

`runes/sap-odata/client.fav` に 2 つの変更を加える:

1. ファイル先頭の `use` 宣言群に `use sap_odata.delta` を追加する
2. `impl SapClient for SapODataClient` ブロックに `delta_fetch<T>` スタブを追加する:

```favnir
fn delta_fetch<T>(ctx: SapODataClient, entity_set: String, delta_link: Option<String>)
    -> Result<DeltaResult<T>, String> {
    Result.err("delta_fetch: not yet implemented")
}
```

**依存**: Step 1 完了後（interface 定義が先）

---

### Step 3: `sap_odata.fav` — `delta_fetch<T>` re-export 追加

`runes/sap-odata/sap_odata.fav` の `$delta 型 re-export` セクションに `delta_fetch<T>` ラッパーを追加する。

```favnir
-- $delta セクション（v95.1.0 で追加済みの re-export の後に追記）
-- 注意: ctx.sap.delta_fetch<T>(...) と型パラメータを明示して呼ぶ（型推論が不確かなため）
public fn delta_fetch<T>(ctx: AppCtx, entity_set: String, delta_link: Option<String>) -> Result<DeltaResult<T>, String> {
    ctx.sap.delta_fetch<T>(entity_set, delta_link)
}
```

**依存**: Step 1・Step 2 完了後

---

### Step 4: `driver.rs` に `mod v95200_tests` を追加

`mod v95100_tests { ... }` の直後に追加する。

テスト 1: `sap_client_interface_has_delta_fetch`
```rust
#[test]
fn sap_client_interface_has_delta_fetch() {
    let content = std::fs::read_to_string("../runes/sap-odata/types.fav")
        .expect("types.fav を読み込めない");
    assert!(
        content.contains("delta_fetch"),
        "SapClient interface に delta_fetch が含まれていない"
    );
}
```

テスト 2: `client_fav_implements_delta_fetch`
```rust
#[test]
fn client_fav_implements_delta_fetch() {
    let content = std::fs::read_to_string("../runes/sap-odata/client.fav")
        .expect("client.fav を読み込めない");
    assert!(
        content.contains("delta_fetch"),
        "client.fav に delta_fetch の実装が含まれていない"
    );
}
```

**依存**: Step 1・Step 2 完了後

---

## 実装順序

```
Step 1: types.fav — SapClient interface に delta_fetch<T> 追加（use sap_odata.delta も追加）
    ↓
Step 2: client.fav — use sap_odata.delta 追加 + SapODataClient に delta_fetch<T> スタブ実装
    ↓
Step 3: sap_odata.fav — delta_fetch<T> re-export 追加
    ↓
Step 4: driver.rs — v95200_tests 追加
    ↓
cargo test で 4,168 tests, 0 failures を確認
```

## 変更ファイル一覧

| ファイル | 操作 |
|---|---|
| `runes/sap-odata/types.fav` | 追記（`use sap_odata.delta` + `SapClient` interface に `delta_fetch<T>` シグネチャ） |
| `runes/sap-odata/client.fav` | 追記（`use sap_odata.delta` + `delta_fetch<T>` スタブ） |
| `runes/sap-odata/sap_odata.fav` | 追記（`delta_fetch<T>` re-export） |
| `fav/src/driver.rs` | 追記（`mod v95200_tests`） |
