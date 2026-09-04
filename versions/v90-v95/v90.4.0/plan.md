# Plan: v90.4.0 — `Ctx.build` に SAP 設定注入を統合

## 依存関係

```
Step 1（既存コード確認）
    ↓
Step 2（SapODataClient + impl 追加）
    ↓
Step 3（Ctx.build 追加）
    ↓
Step 4（driver.rs テスト追加）
    ↓
Step 5（cargo test）
    ↓
Step 6（CHANGELOG 更新）
    ↓
Step 7（CI 事前確認）
```

## Steps

### Step 1: 既存コードの確認

- `runes/sap-odata/client.fav` の現状を確認する（`odata_get` / `odata_list` / `basic_auth_header` 等）
- `runes/sap-odata/sap_odata.fav` の関数シグネチャ（`business_partners` 等）を確認する（`SapODataClient.impl` で委譲先として使用）
- `runes/ctx/ctx.fav` の現状を確認する（`Ctx.build` 追加場所の特定）
- 現テスト数が 4048 であることを確認する

### Step 2: `SapODataClient` を `runes/sap-odata/client.fav` に追加

- `SapODataClient = { config: SapConfig }` レコード型を定義する
- `impl SapClient for SapODataClient` ブロックを追加する（5 メソッド）
  - 各メソッドは `runes/sap-odata/sap_odata.fav` の対応関数に `ctx.config` を渡して委譲する
- コメントスタイルは `--`（`client.fav` の既存スタイルに合わせる）

### Step 3: `Ctx.build` を `runes/ctx/ctx.fav` に追加

- `ctx.fav` 末尾に `Ctx.build` 関数を追加する
- `sap_config_from_env()` を呼び出して `SapODataClient` を生成し `AppCtx.sap` に設定する
- コメントスタイルは `//`（`ctx.fav` の既存スタイルに合わせる）

### Step 4: `driver.rs` に `mod v90400_tests` を追加

- `mod v90300_tests` の直後に `#[cfg(test)] mod v90400_tests { ... }` を追加する
- `ctx_build_integrates_sap`: `ctx.fav` に `Ctx.build` と `sap` の両方が含まれることを確認
- `sap_odata_client_impl_exists`: `client.fav` に `impl SapClient for SapODataClient` が含まれることを確認

### Step 5: `cargo test` で全 pass 確認

- `cargo test 2>&1 | grep "test result"` で 4050 tests, 0 failures を確認する

### Step 6: `CHANGELOG.md` に v90.4.0 エントリを追加

- `## [v90.3.0]` の前に v90.4.0 エントリを追加する
- `SapODataClient` / `Ctx.build` / `4050` が含まれることを確認する

### Step 7: CI 事前確認

- `cargo clippy --locked -- -D warnings` が pass することを確認する
- `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
