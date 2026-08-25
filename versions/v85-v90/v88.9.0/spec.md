# Spec: v88.9.0 — 安定化・コードフリーズ

## Background

v88.1.0〜v88.8.0 で SAP Procurement 関連の全型定義・関数スタブ・E2E デモ基盤が揃った。
本バージョンは v88.1〜v88.8 の全機能を通しで確認する安定化スプリント。
バグ修正のみ受け入れ（新機能追加なし）。

## Goals

1. `cargo test` 全 pass の確認（4,013 → 4,015）
2. Material / PurchaseOrder CRUD の型・関数カバレッジを Rust テストで担保
3. 在庫クロスチェックパイプライン（Scenario 3）の E2E 存在確認
4. Lambda デモ Terraform の構成確認（v88.8.0 の成果物を通しで確認）

## Verification Scope（コードフリーズ確認対象）

| 機能 | ファイル | 確認内容 |
|---|---|---|
| Material CRUD | `runes/sap-odata/material.fav` | `material_by_id` 存在 |
| PurchaseOrder CRUD | `runes/sap-odata/purchase_order.fav` | `purchase_orders` 存在 |
| 在庫クロスチェック型 | `runes/sap-odata/stock.fav` | `detect_stock_shortage` / `format_stock_alerts` 存在 |
| 在庫クロスチェック E2E | `infra/e2e-demo/sap-odata/pipeline.fav` | `check_stock_vs_orders` 存在 |
| Lambda 基盤 | `infra/e2e-demo/sap-odata/terraform/main.tf` | v88.8.0 の `sap_e2e_demo_terraform_exists` テストで担保済み（本バージョンでは目視確認のみ） |

**Note**: Cargo.toml のバージョンは v89.0.0 宣言バージョンまで `88.0.0` のまま維持する（本バージョンではバンプしない）。

## Success Criteria（Rust テストで担保）

- `sap_procurement_material_and_po_covered`:
  `runes/sap-odata/material.fav` に `"material_by_id"` を含み、
  `runes/sap-odata/purchase_order.fav` に `"purchase_orders"` を含む
- `sap_procurement_scenario3_pipeline_exists`:
  `infra/e2e-demo/sap-odata/pipeline.fav` に `"check_stock_vs_orders"` を含む
- `cargo test` で 4,015 tests, 0 failures（4,013 + 2）

## Files to Modify

| ファイル | 変更種別 |
|---|---|
| `fav/src/driver.rs` | `mod v88900_tests` 追加 |

**Note**: CHANGELOG / MILESTONE / site MDX 更新は v89.0.0 宣言バージョンでまとめて実施する（本バージョンは対象外）
