# Plan: v89.3.0 — シナリオ 4: 購買→支払サイクル照合

## 実装ステップ

### Step 1: `infra/e2e-demo/sap-odata/pipeline.fav` にシナリオ 4 を追記

`check_stock_vs_orders` 関数の直後に追加:

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

### Step 2: `fav/src/driver.rs` に `mod v89300_tests` を追加

`mod v89200_tests { ... }` の直後に追加:

```rust
#[cfg(test)]
mod v89300_tests {
    #[test]
    fn sap_e2e_pipeline_contains_outstanding_payables() {
        let content = std::fs::read_to_string(
            "../infra/e2e-demo/sap-odata/pipeline.fav",
        )
        .expect("infra/e2e-demo/sap-odata/pipeline.fav should exist");
        assert!(
            content.contains("outstanding_payables"),
            "pipeline.fav should define outstanding_payables (Scenario 4)"
        );
    }

    #[test]
    fn sap_e2e_pipeline_has_all_four_scenarios() {
        let content = std::fs::read_to_string(
            "../infra/e2e-demo/sap-odata/pipeline.fav",
        )
        .expect("infra/e2e-demo/sap-odata/pipeline.fav should exist");
        assert!(content.contains("sync_business_partners"), "pipeline.fav should have Scenario 1");
        assert!(content.contains("daily_sales_report"), "pipeline.fav should have Scenario 2");
        assert!(content.contains("check_stock_vs_orders"), "pipeline.fav should have Scenario 3");
        assert!(content.contains("outstanding_payables"), "pipeline.fav should have Scenario 4");
    }
}
```

### Step 3: `cargo test` で全 pass 確認

4,023 + 2 = 4,025 tests, 0 failures を確認する。

### Step 4: CI 事前確認

```bash
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```

---

**Note**: CHANGELOG / MILESTONE / site MDX 更新は v90.0.0 宣言バージョンでまとめて実施するため、本バージョンでは省略する。
**Note**: Cargo.toml のバージョンは v90.0.0 宣言まで `89.0.0` のまま維持する。
