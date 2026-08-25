# Spec: v88.6.0 — シナリオ 3: 在庫 × 受注クロスチェック

## Background

v88.5.0 で `create_purchase_order()` POST を追加し SAP Procurement 1.0 の CRUD が揃った。
本バージョンでは業務シナリオ 3「在庫 × 受注クロスチェック」の E2E 実装を行う。
`pipeline.fav` にシナリオ 3 関数 `check_stock_vs_orders` を追加し、
`runes/sap-odata/stock.fav` を新規作成して `StockAlert` 型スタブを定義する。
`detect_stock_shortage()` の実装は v88.7.0 で完成させる。

## Goals

1. `runes/sap-odata/stock.fav` を新規作成し、`StockSeverity` enum と `StockAlert` 型を定義する
2. `infra/e2e-demo/sap-odata/pipeline.fav` にシナリオ 3 関数 `check_stock_vs_orders` を追加する
3. Rust テスト 2 件で pipeline の関数と `StockAlert` 型の存在を担保する

## API / Syntax Examples

```favnir
-- runes/sap-odata/stock.fav（新規作成）

public type StockSeverity = Critical | Warning | Info

public type StockAlert = {
    material_id:   String,
    description:   String,
    severity:      StockSeverity,
    open_quantity: Float,
    message:       String
}
```

```favnir
-- infra/e2e-demo/sap-odata/pipeline.fav に追加（シナリオ 3）

-- シナリオ 3: 在庫 × 受注クロスチェック（v88.6.0）
fn check_stock_vs_orders(ctx: AppCtx) -> Result<List<StockAlert>, String> {
    bind cfg       <- sap_odata.sap_config_from_env()
    bind orders    <- sap_odata.sales_orders(cfg, SalesOrderFilter {
        status:         Option.some(SalesOrderStatus.Open),
        customer_id:    Option.none(),
        created_after:  Option.none(),
        created_before: Option.none(),
        sales_org:      Option.none(),
        top:            Option.none()
    })
    bind materials <- sap_odata.materials(cfg, MaterialFilter {
        material_type: Option.some(MaterialType.FinishedProduct),
        plant:         Option.none(),
        top:           Option.none()
    })
    bind alerts    <- sap_odata.detect_stock_shortage(orders, materials)
    Result.ok(alerts)
}
```

## Success Criteria（Rust テストで担保）

- `infra/e2e-demo/sap-odata/pipeline.fav` に `"check_stock_vs_orders"` を含む
- `runes/sap-odata/stock.fav` に `"StockAlert"` を含む
- `cargo test` で 4,009 tests, 0 failures（T0 で確認したベースライン 4,007 + 2）
- Rust テスト 2 件:
  - `sap_e2e_pipeline_contains_check_stock_vs_orders`（`pipeline.fav` に `"check_stock_vs_orders"` を確認）
  - `stock_alert_type_exists`（`stock.fav` に `"StockAlert"` を確認）

## Files to Modify

| ファイル | 変更種別 |
|---|---|
| `runes/sap-odata/stock.fav` | 新規作成（`StockSeverity` / `StockAlert` 型スタブ） |
| `infra/e2e-demo/sap-odata/pipeline.fav` | 追記（シナリオ 3 `check_stock_vs_orders` 関数） |
| `fav/src/driver.rs` | `mod v88600_tests` 追加 |

**Note**: `detect_stock_shortage()` の実装は v88.7.0 で `stock.fav` に**追記**する（v88.7.0 ロードマップは「新規作成」と記載していたが v88.6.0 で先行作成のため「追記」に修正済み）。
**Note**: CHANGELOG / MILESTONE / site MDX 更新は v89.0.0 宣言バージョンでまとめて実施する（本バージョンは対象外）
