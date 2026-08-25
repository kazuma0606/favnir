# Plan: v89.0.0 — SAP Procurement 1.0 宣言

## 実装ステップ

### Step 1: CHANGELOG.md 更新（テスト先行のため最初に実施）

`changelog_has_v89_0_0` テストが通るよう、CHANGELOG.md の先頭に追加:

```markdown
## [v89.0.0] — 2026-08-24 — SAP Procurement 1.0 宣言

> 「在庫と発注が型になった。
>  Material と PurchaseOrder を並べれば、不足が見える。」

### Added
- `fav/src/driver.rs` — `mod v89000_tests`（テスト 4 件）を追加
- `MILESTONE.md` — SAP Procurement 1.0 マイルストーンを追加
- 合計テスト数: **4,019**（+4）

### Changed
- `fav/Cargo.toml` — version を `88.0.0` → `89.0.0` に更新
- `versions/current.md` — v89.0.0 に更新
```

### Step 2: MILESTONE.md に SAP Procurement 1.0 を追加

MILESTONE.md の先頭（v88.0.0 エントリの前）に追加:

```markdown
## v89.0.0（2026-08-24）— SAP Procurement 1.0 宣言

> 「在庫と発注が型になった。
>  Material と PurchaseOrder を並べれば、不足が見える。」

**SAP Procurement 1.0** の宣言バージョン。v88.1.0〜v88.9.0 で実装した
SAP Integration Era の第 4 スプリントの完成を宣言した。テスト数: 4,019。

**SAP Procurement 1.0（v88.1〜v88.9）達成内容:**
- **型定義**: `Material` / `MaterialType` / `MaterialFilter`
- **型定義**: `PurchaseOrder` / `PurchaseOrderItem` / `PurchaseOrderStatus` / `PurchaseOrderFilter`
- **型定義**: `NewPurchaseOrder` / `NewPurchaseOrderItem`
- **型定義**: `StockSeverity` / `StockAlert`
- **品目マスタ**: `material_by_id(cfg, material_id)` — 単一品目取得
- **品目リスト**: `materials(cfg, MaterialFilter)` — 品目絞り込み検索
- **発注検索**: `purchase_orders(cfg, PurchaseOrderFilter)` — フィルタ検索
- **発注取得**: `purchase_order_by_id(cfg, po_number, expand_items)` — 明細展開対応
- **発注作成**: `create_purchase_order(cfg, NewPurchaseOrder)` — 発注伝票作成
- **在庫チェック**: `detect_stock_shortage(orders, materials)` — 受注 × 品目クロスチェック
- **E2E パイプライン**: Scenario 3（`check_stock_vs_orders`）
- **Lambda 基盤**: `infra/e2e-demo/sap-odata/terraform/`（main.tf / ssm.tf / variables.tf）
- **実行スクリプト**: `infra/e2e-demo/sap-odata/scripts/run.sh`
```

### Step 3: `cargo clean` 実施

```bash
cargo clean
```

`fav/tmp/hello.fav` が残ることを確認する（`cargo clean` は `target/` のみ削除）。

### Step 4: `fav/Cargo.toml` バージョン更新

`version = "88.0.0"` → `version = "89.0.0"` に変更。

### Step 5: `driver.rs` 内の `"88.0.0"` を `"89.0.0"` に一括更新

```bash
sed -i 's/88\.0\.0/89.0.0/g' /c/Users/yoshi/favnir/fav/src/driver.rs
```

driver.rs 内の `"88.0.0"` 文字列（約40箇所）を一括置換する。
絶対パスを使用（相対パスは CWD 依存で失敗する可能性があるため）。

### Step 6: `driver.rs` に `mod v89000_tests` を追加

`mod v88900_tests { ... }` の直後に追加:

```rust
#[cfg(test)]
mod v89000_tests {
    #[test]
    fn cargo_toml_version_is_89_0_0() {
        let content = include_str!("../Cargo.toml");
        assert!(content.contains("version = \"89.0.0\""), "Cargo.toml version should be 89.0.0");
    }
    #[test]
    fn changelog_has_v89_0_0() {
        let content = include_str!("../../CHANGELOG.md");
        assert!(content.contains("[v89.0.0]"), "CHANGELOG.md should have v89.0.0 entry");
    }
    #[test]
    fn milestone_has_sap_procurement() {
        let content = include_str!("../../MILESTONE.md");
        assert!(content.contains("SAP Procurement"), "MILESTONE.md should have SAP Procurement milestone");
    }
    #[test]
    fn sap_odata_rune_has_material_type() {
        let content = std::fs::read_to_string("../runes/sap-odata/sap_odata.fav")
            .expect("runes/sap-odata/sap_odata.fav should exist");
        assert!(content.contains("Material"), "sap_odata.fav should re-export Material type");
    }
}
```

### Step 7: `versions/current.md` 更新

v89.0.0 に更新する。

### Step 8: `README.md` 更新

バージョン表記を v89.0.0 に更新する。

### Step 9: `cargo test` で全 pass 確認

`cargo test` を実行し、4,015 + 4 = 4,019 tests, 0 failures を確認する（`cargo test` はビルドを自動実行するため `cargo build` は不要）。

---

**Note**: CHANGELOG テスト（`changelog_has_v89_0_0`）は `include_str!` マクロを使用するためコンパイル時に埋め込まれる。
CHANGELOG 更新は Step 1 で実施（テストコード追加より先）。
