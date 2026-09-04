# Spec: v90.5.0 — `runes/sap-odata/sap_odata.fav` を `ctx.sap.*` スタイルに対応

## Background

v90.4.0 で `Ctx.build()` が完成し、`AppCtx.sap` に `SapODataClient` が注入される。
しかし `sap_odata.fav` の公開関数は依然として `cfg: SapConfig` を第1引数に取る旧スタイルのままである。

```favnir
-- 旧スタイル（現状）
public fn business_partners(cfg: SapConfig, filter: BusinessPartnerFilter)
    -> Result<List<BusinessPartner>, String>
```

ユーザーコードが `ctx.sap.business_partners(filter)` で直接呼び出せるようになった今、
Rune 関数も `ctx: AppCtx` を第1引数に取る新スタイルへ移行することで、
`AppCtx` を受け取るパイプラインから自然に呼び出せるようになる。

後方互換性のため、旧 `cfg: SapConfig` シグネチャは `-- deprecated` コメントを付けて残す（削除は v91.0.0）。

## Goals

1. `sap_odata.fav` の `SapClient` interface に対応する 4 関数（`business_partners` / `sales_orders` / `materials` / `journal_entries`）のシグネチャを `ctx: AppCtx` スタイルに更新する
2. 各関数の本体を `ctx.sap.METHOD(filter)` への委譲に書き換える
3. 旧 `cfg: SapConfig` シグネチャを deprecated コメント付きで維持する
4. Rust テスト 2 件を追加して構造を保証する

## Syntax / API

```favnir
-- 変更前
public fn business_partners(cfg: SapConfig, filter: BusinessPartnerFilter)
    -> Result<List<BusinessPartner>, String> {
    business_partner.business_partners(cfg, filter)
}

-- 変更後（deprecated 旧スタイルを先に残し、新スタイルを直後に追加）
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

同様に `sales_orders` / `materials` / `journal_entries` も更新する。

## Success Criteria

- `sap_odata.fav` に `ctx: AppCtx` が含まれる（新スタイル関数が存在する）
- `sap_odata.fav` に `ctx.sap.business_partners` が含まれる（委譲が正しく記述されている）
- driver.rs の `sap_odata_fav_uses_app_ctx` テストが pass する
- driver.rs の `sap_odata_fav_delegates_to_ctx_sap` テストが pass する
- 事前確認: `sap_odata.fav` の `public fn` が対象 4 関数（business_partners / sales_orders / materials / journal_entries）を含む正しい件数であることを確認する
- `cargo test` で 4,052 tests, 0 failures

## Files to Modify

| ファイル | 操作 |
|---|---|
| `runes/sap-odata/sap_odata.fav` | 4 関数のシグネチャ更新 + deprecated 旧シグネチャ追加 |
| `fav/src/driver.rs` | `mod v90500_tests` 追加 |
| `CHANGELOG.md` | v90.5.0 エントリ追加 |
