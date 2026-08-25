# Tasks: v88.9.0 — 安定化・コードフリーズ

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,013 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v88800_tests` が存在することを確認する（v88.8.0 完了済みの証拠）
- [x] `fav/Cargo.toml` の version が `88.0.0` であることを確認する（v89.0.0 宣言バージョンまでバンプしない設計のため、88.0.0 が正しい）

## T1: 安定化確認（目視チェック）

- [x] `runes/sap-odata/material.fav` に `material_by_id` が存在することを確認する
- [x] `runes/sap-odata/purchase_order.fav` に `purchase_orders` / `purchase_order_by_id` / `create_purchase_order` が存在することを確認する
- [x] `runes/sap-odata/stock.fav` に `detect_stock_shortage` / `format_stock_alerts` が存在することを確認する
- [x] `infra/e2e-demo/sap-odata/pipeline.fav` に `check_stock_vs_orders` が存在することを確認する（Scenario 3）
- [x] `infra/e2e-demo/sap-odata/terraform/main.tf` に `favnir-sap-e2e-demo` が存在することを確認する
- [x] バグが発見された場合はここで修正する（新機能追加は禁止）— バグなし

## T2: `driver.rs` に `mod v88900_tests` を追加

- [x] `mod v88800_tests { ... }` の直後に `#[cfg(test)] mod v88900_tests { ... }` を追加する
- [x] `sap_procurement_material_and_po_covered` テストを実装する（`material.fav` + `purchase_order.fav` の関数存在確認）
- [x] `sap_procurement_scenario3_pipeline_exists` テストを実装する（`pipeline.fav` に `check_stock_vs_orders` を確認）

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,015 tests, 0 failures であることを確認する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## spec-reviewer 指摘対応

- [MED] spec.md の Verification Scope に `stock.fav` 行を追加（`detect_stock_shortage` / `format_stock_alerts`）
- [MED] Lambda Terraform 確認欄に「v88.8.0 の `sap_e2e_demo_terraform_exists` で担保済み（本バージョンは目視のみ）」を明記
- [LOW] spec.md / plan.md の Note に「Cargo.toml は v89.0.0 まで 88.0.0 のまま」を明記
