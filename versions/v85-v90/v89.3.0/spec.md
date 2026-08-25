# Spec: v89.3.0 — シナリオ 4: 購買→支払サイクル照合

## Background

v89.2.0 で `OutstandingPayable` 型と `match_unposted_orders()` スタブが揃った。
本バージョンでは E2E パイプライン（`infra/e2e-demo/sap-odata/pipeline.fav`）に
シナリオ 4「購買→支払サイクル照合」を追加し、全 4 業務シナリオを完成させる。

### 現行シナリオ一覧

| # | 関数名 | 追加バージョン |
|---|---|---|
| 1 | `sync_business_partners` | v86.6.0 |
| 2 | `daily_sales_report` | v87.5.0 |
| 3 | `check_stock_vs_orders` | v88.6.0 |
| 4 | `outstanding_payables` | **v89.3.0（本バージョン）** |

## Goals

1. `infra/e2e-demo/sap-odata/pipeline.fav` にシナリオ 4 を追記する
   - 関数: `outstanding_payables(ctx: AppCtx) -> Result<List<OutstandingPayable>, String>`
2. `fav/src/driver.rs` に `mod v89300_tests` を追加する（2 件）

## API / Syntax Examples

```favnir
-- シナリオ 4: 購買→支払サイクル照合（v89.3.0）

fn outstanding_payables(ctx: AppCtx) -> Result<List<OutstandingPayable>, String> {
    bind cfg      <- sap_odata.sap_config_from_env()
    bind pos      <- sap_odata.purchase_orders(cfg, PurchaseOrderFilter {
        status:        Option.some(PurchaseOrderStatus.PartiallyDelivered),
        vendor_id:     Option.none(),
        created_after: Option.none(),
        plant:         Option.none(),
        top:           Option.none()
    })
    bind journals <- sap_odata.journal_entries(cfg, JournalFilter {
        fiscal_year:       Option.some(2026),
        posting_date_from: Option.none(),
        company_code:      Option.none(),
        reference:         Option.none(),
        top:               Option.none()
    })
    bind unpaid   <- sap_odata.match_unposted_orders(pos, journals)
    bind json     <- Json.encode(unpaid)
    bind _        <- ctx.s3.put_object("favnir-sap-demo", "payables/outstanding.json", json)
    Result.ok(unpaid)
}
```

## Success Criteria（Rust テストで担保）

- `sap_e2e_pipeline_contains_outstanding_payables`:
  `infra/e2e-demo/sap-odata/pipeline.fav` に `"outstanding_payables"` を含む
- `sap_e2e_pipeline_has_all_four_scenarios`:
  `infra/e2e-demo/sap-odata/pipeline.fav` に以下の 4 関数名をすべて含む
  - `"sync_business_partners"`（シナリオ 1）
  - `"daily_sales_report"`（シナリオ 2）
  - `"check_stock_vs_orders"`（シナリオ 3）
  - `"outstanding_payables"`（シナリオ 4）
- `cargo test` で 4,025 tests, 0 failures（4,023 + 2）

## Files to Modify

| ファイル | 変更種別 |
|---|---|
| `infra/e2e-demo/sap-odata/pipeline.fav` | 追記（シナリオ 4 関数追加） |
| `fav/src/driver.rs` | `mod v89300_tests` 追加 |

**前提確認**:
- `PurchaseOrderStatus.PartiallyDelivered` バリアントは v88.3.0 で `types.fav` に定義済み
- `OutstandingPayable` 型・`match_unposted_orders` 関数は v89.2.0 COMPLETE 済み
- `pipeline.fav` は Rust テストで文字列存在確認のみ（型チェック対象外）

**v89.5.0 との関係**: 本バージョンで追記した `outstanding_payables` は v89.5.0 でそのまま維持する。v89.5.0 では Lambda デプロイスクリプトの整備・4 シナリオの統合実行確認を行うが、`pipeline.fav` 本体の書き直しは行わない。

**Note**: CHANGELOG / MILESTONE / site MDX 更新は v90.0.0 宣言バージョンでまとめて実施する（本バージョンは対象外）
**Note**: Cargo.toml のバージョンは v90.0.0 宣言まで `89.0.0` のまま維持する。
