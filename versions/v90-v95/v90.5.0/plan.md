# Plan: v90.5.0 — `runes/sap-odata/sap_odata.fav` を `ctx.sap.*` スタイルに対応

## 依存関係

```
Step 1（現状確認）
    ↓
Step 2（sap_odata.fav 4 関数を更新）
    ↓
Step 3（driver.rs テスト追加）
    ↓
Step 4（cargo test）
    ↓
Step 5（CHANGELOG 更新）
    ↓
Step 6（CI 事前確認）
```

## Steps

### Step 1: 現状確認

- `runes/sap-odata/sap_odata.fav` の `business_partners` / `sales_orders` / `materials` / `journal_entries` の現行シグネチャを確認する
- 現テスト数が 4050 であることを確認する

### Step 2: `sap_odata.fav` の 4 関数を更新

更新対象: `business_partners` / `sales_orders` / `materials` / `journal_entries`

各関数について以下を実施する:
1. 既存の `cfg: SapConfig` 版を `deprecated` コメント付き `*_cfg` 関数名にリネームする（後方互換維持）
2. 新しい `ctx: AppCtx` 版を追加し、本体を `ctx.sap.METHOD(filter)` への委譲に書き換える

例（`business_partners`）:
```favnir
-- deprecated: cfg スタイル（v91.0.0 で削除予定）
public fn business_partners_cfg(cfg: SapConfig, filter: BusinessPartnerFilter)
    -> Result<List<BusinessPartner>, String> {
    business_partner.business_partners(cfg, filter)
}
public fn business_partners(ctx: AppCtx, filter: BusinessPartnerFilter)
    -> Result<List<BusinessPartner>, String> {
    ctx.sap.business_partners(filter)
}
```

コメントスタイルは `--`（sap_odata.fav の既存スタイル）。

### Step 3: `driver.rs` に `mod v90500_tests` を追加

- `mod v90400_tests` の直後に `#[cfg(test)] mod v90500_tests { ... }` を追加する
- `sap_odata_fav_uses_app_ctx`: `sap_odata.fav` に `ctx: AppCtx` が含まれることを確認
- `sap_odata_fav_delegates_to_ctx_sap`: `sap_odata.fav` に `ctx.sap.business_partners` が含まれることを確認

### Step 4: `cargo test` で全 pass 確認

- `cargo test 2>&1 | grep "test result"` で 4052 tests, 0 failures を確認する

### Step 5: `CHANGELOG.md` に v90.5.0 エントリを追加

- `## [v90.4.0]` の前に v90.5.0 エントリを追加する
- `ctx: AppCtx` / `ctx.sap.*` / `deprecated` / `4052` が含まれることを確認する

### Step 6: CI 事前確認

- `cargo clippy --locked -- -D warnings` が pass することを確認する
- `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
