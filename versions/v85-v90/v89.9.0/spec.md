# Spec: v89.9.0 — 安定化・コードフリーズ

## Background

v89.1〜v89.8 で SAP Integration の全機能（型定義・Rune 実装・4 シナリオ E2E・ドキュメント・OSS 整備・パフォーマンス計測）が完成した。
本バージョンは新機能を追加せず、全機能の通し確認と最終安定化を行うコードフリーズスプリントである。

v90.0.0 の SAP Integration 1.0 宣言に向けた最終チェックポイントとして機能する。

## Goals

1. `cargo test` 全 pass 確認（4,035 tests → 継続 pass）
2. 全 4 シナリオが `pipeline.fav` に揃っていることを確認するテストを追加
3. Rune Registry デプロイ確認（`import rune "sap-odata"` が解決可能なこと）を記録するテストを追加
4. バグ修正のみ受け入れ（新機能追加なし）

## Success Criteria（Rust テストで担保）

- `sap_all_four_scenarios_in_pipeline`:
  `infra/e2e-demo/sap-odata/pipeline.fav` に 4 シナリオ全て（`sync_business_partners` / `daily_sales_report` / `check_stock_vs_orders` / `outstanding_payables`）が含まれる
- `sap_integration_rune_registry_deployed`:
  `runes/sap-odata/` ディレクトリが存在することを確認（Rune Registry へのデプロイ可能性の担保）
- `cargo test` で 4,037 tests, 0 failures（4,035 + 2）

## テスト詳細

```rust
#[cfg(test)]
mod v89900_tests {
    // use super::* は不要（std::fs / std::path::Path のみ使用）
    #[test]
    fn sap_all_four_scenarios_in_pipeline() {
        let content = std::fs::read_to_string("../infra/e2e-demo/sap-odata/pipeline.fav")
            .expect("infra/e2e-demo/sap-odata/pipeline.fav should exist");
        assert!(content.contains("sync_business_partners"),  "Scenario 1 missing");
        assert!(content.contains("daily_sales_report"),      "Scenario 2 missing");
        assert!(content.contains("check_stock_vs_orders"),   "Scenario 3 missing");
        assert!(content.contains("outstanding_payables"),    "Scenario 4 missing");
    }

    #[test]
    fn sap_integration_rune_registry_deployed() {
        assert!(
            std::path::Path::new("../runes/sap-odata").exists(),
            "runes/sap-odata/ should exist (Rune Registry deployment artifact)"
        );
    }
}
```

## Files to Create / Modify

| ファイル | 変更種別 |
|---|---|
| `fav/src/driver.rs` | `mod v89900_tests` 追加 |

**前提確認**:
- `infra/e2e-demo/sap-odata/pipeline.fav` は v89.3.0 で 4 シナリオ全て実装済み
- `runes/sap-odata/` は v89.1.0 以降に作成済み（Rune 実装の成果物）
- Cargo.toml のバージョンは v90.0.0 宣言まで `89.0.0` のまま維持する
- テストは `fav/` を cwd として実行されるため `"../runes/sap-odata"` は `runes/sap-odata` に解決される

**Note**: CHANGELOG / MILESTONE 更新は v90.0.0 宣言バージョンでまとめて実施する（本バージョンは対象外）。
