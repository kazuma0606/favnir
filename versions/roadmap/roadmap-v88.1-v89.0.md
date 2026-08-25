# Roadmap v88.1.0 〜 v89.0.0 — SAP Procurement 1.0

Date: 2026-08-22
Status: 未着手（v88.0.0 完了後に開始）

マスターロードマップ: [roadmap-v85.1-v90.0.md](roadmap-v85.1-v90.0.md)

---

## 前提

- 直前完了: v88.0.0「SAP Sales 1.0 宣言」（tests = 3,997）
- 本スプリントは SAP Integration Era の第 4 スプリント
- 目標: v89.0.0「SAP Procurement 1.0 宣言」（tests = 4,019）

### 着手前チェックリスト

- `versions/current.md` の最新安定版が v88.0.0 になっていることを確認する
- `versions/v85-v90/v88.1.0/` ディレクトリを作成し、spec.md / plan.md / tasks.md を準備すること
- `runes/sap-odata/` に SalesOrder 型・関数が実装済みであることを確認する

### スプリントの性格

品目マスタ（`Material`）・発注伝票（`PurchaseOrder`）を型安全に扱えるようにする。
在庫 × 受注クロスチェック（業務シナリオ 3）と E2E デモ Lambda 基盤も整備する。
A（新機能）60% + B（インフラ）40% の構成。

---

## バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v88.1.0 | `Material` 型定義 + `MaterialFilter` + `materials()` | 3997 + 2 = 3999 | 未着手 |
| v88.2.0 | `material_by_id()` + `MaterialType` enum 完全化 | 3999 + 2 = 4001 | 未着手 |
| v88.3.0 | `PurchaseOrder` / `PurchaseOrderItem` 型定義 | 4001 + 2 = 4003 | 未着手 |
| v88.4.0 | `purchase_orders()` / `purchase_order_by_id()` クエリ | 4003 + 2 = 4005 | 未着手 |
| v88.5.0 | `create_purchase_order()` POST 実装 | 4005 + 2 = 4007 | 未着手 |
| v88.6.0 | シナリオ 3: 在庫 × 受注クロスチェック（Material × SalesOrder） | 4007 + 2 = 4009 | 未着手 |
| v88.7.0 | `StockAlert` 型 + `detect_stock_shortage()` | 4009 + 2 = 4011 | 未着手 |
| v88.8.0 | E2E デモ Lambda 基盤（`infra/e2e-demo/sap-odata/terraform/`） | 4011 + 2 = 4013 | 未着手 |
| v88.9.0 | 安定化・コードフリーズ | 4013 + 2 = 4015 | 未着手 |
| v89.0.0 | SAP Procurement 1.0 宣言 ★クリーンアップ | 4015 + 4 = 4019 | 未着手 |

---

## v88.1.0 — `Material` 型定義 + `MaterialFilter` + `materials()`

品目マスタの Favnir 型と一覧取得関数を実装する。

```favnir
type MaterialType = FinishedProduct | RawMaterial | SemiFinished | Trading | Service

type Material = {
    material_id:   String,
    description:   String,
    material_type: MaterialType,
    base_unit:     String,
    weight:        Option<Float>,
    weight_unit:   Option<String>,
    plant:         Option<String>
}

type MaterialFilter = {
    material_type: Option<MaterialType>,
    plant:         Option<String>,
    top:           Option<Int>
}

public fn materials(cfg: SapConfig, filter: MaterialFilter) -> Result<List<Material>, String>
```

**実装ファイル:** `runes/sap-odata/material.fav`（新規作成）

**完了条件**: Rust テスト 2 件（3997 + 2 = 3999）
- `material_type_defined_in_rune`
- `materials_function_exists`

---

## v88.2.0 — `material_by_id()` + `MaterialType` enum 完全化

単一品目の取得関数と MaterialType 全バリアントを確認する。

```favnir
public fn material_by_id(cfg: SapConfig, material_id: String) -> Result<Material, String>
```

**完了条件**: Rust テスト 2 件（3999 + 2 = 4001）
- `material_by_id_function_exists`
- `material_type_enum_has_finished_product`

---

## v88.3.0 — `PurchaseOrder` / `PurchaseOrderItem` 型定義

発注伝票と発注明細の Favnir 型を定義する。

```favnir
type PurchaseOrderStatus = Open | PartiallyDelivered | Completed | Cancelled

type PurchaseOrderItem = {
    item_number: Int,
    material_id: String,
    quantity:    Float,
    unit:        String,
    net_price:   Float,
    currency:    String,
    plant:       String
}

type PurchaseOrder = {
    po_number:    String,
    vendor_id:    String,
    status:       PurchaseOrderStatus,
    total_amount: Float,
    currency:     String,
    created_at:   String,
    items:        Option<List<PurchaseOrderItem>>
}
```

**実装ファイル:** `runes/sap-odata/types.fav`（SalesOrder 型の後に追加）

**完了条件**: Rust テスト 2 件（4001 + 2 = 4003）
- `purchase_order_type_defined_in_rune`
- `purchase_order_item_type_defined_in_rune`

---

## v88.4.0 — `purchase_orders()` / `purchase_order_by_id()` クエリ

発注伝票の一覧取得・単件取得を実装する。

```favnir
type PurchaseOrderFilter = {
    vendor_id:     Option<String>,
    status:        Option<PurchaseOrderStatus>,
    created_after: Option<String>,
    plant:         Option<String>,
    top:           Option<Int>
}

public fn purchase_orders(cfg: SapConfig, filter: PurchaseOrderFilter) -> Result<List<PurchaseOrder>, String>
public fn purchase_order_by_id(cfg: SapConfig, po_number: String, expand_items: Bool) -> Result<PurchaseOrder, String>
```

**実装ファイル:** `runes/sap-odata/purchase_order.fav`（新規作成）

**完了条件**: Rust テスト 2 件（4003 + 2 = 4005）
- `purchase_orders_function_exists`
- `purchase_order_filter_type_exists`

---

## v88.5.0 — `create_purchase_order()` POST

発注伝票の新規作成を実装する。

```favnir
type NewPurchaseOrderItem = {
    material_id: String,
    quantity:    Float,
    unit:        String,
    plant:       String
}

type NewPurchaseOrder = {
    vendor_id: String,
    currency:  String,
    items:     List<NewPurchaseOrderItem>
}

public fn create_purchase_order(cfg: SapConfig, order: NewPurchaseOrder) -> Result<PurchaseOrder, String>
```

**完了条件**: Rust テスト 2 件（4005 + 2 = 4007）
- `create_purchase_order_function_exists`
- `new_purchase_order_type_exists`

---

## v88.6.0 — シナリオ 3: 在庫 × 受注クロスチェック

業務シナリオ 3 の E2E 実装。オープン受注と品目マスタを突き合わせ、在庫不足を検出する。

```favnir
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

**完了条件**: Rust テスト 2 件（4007 + 2 = 4009）
- `sap_e2e_pipeline_contains_check_stock_vs_orders`
- `stock_alert_type_exists`

---

## v88.7.0 — `StockAlert` 型 + `detect_stock_shortage()`

在庫不足アラートの型と純粋計算関数を実装する。

```favnir
type StockSeverity = Critical | Warning | Info

type StockAlert = {
    material_id:   String,
    description:   String,
    severity:      StockSeverity,
    open_quantity: Float,
    message:       String
}

-- 注: detect_stock_shortage は SAP への HTTP アクセスをしない純粋な計算関数のため
--     cfg: SapConfig を取らない（他の Rune 関数との例外的な違い）。
public fn detect_stock_shortage(
    orders:    List<SalesOrder>,
    materials: List<Material>
) -> Result<List<StockAlert>, String>

fn format_stock_alerts(alerts: List<StockAlert>) -> String
```

**実装ファイル:** `runes/sap-odata/stock.fav`（v88.6.0 で新規作成済み、本バージョンでは追記）

**完了条件**: Rust テスト 2 件（4009 + 2 = 4011）
- `detect_stock_shortage_function_exists`
- `format_stock_alerts_function_exists`

---

## v88.8.0 — E2E デモ Lambda 基盤

SAP パイプラインを AWS Lambda で実行するデモ基盤を整備する。

**実装内容:**
- `infra/e2e-demo/sap-odata/terraform/main.tf`（Lambda + IAM + S3 出力バケット）
- `infra/e2e-demo/sap-odata/terraform/ssm.tf`（SSM パラメータ参照）
- `infra/e2e-demo/sap-odata/terraform/variables.tf`
- `infra/e2e-demo/sap-odata/scripts/run.sh`（デモ実行スクリプト）

**完了条件**: Rust テスト 2 件（4011 + 2 = 4013）
- `sap_e2e_demo_terraform_exists`
- `sap_e2e_demo_run_script_exists`

---

## v88.9.0 — 安定化・コードフリーズ

v88.1〜v88.8 の全機能を通しで確認する安定化スプリント。

**実装内容:**
- `cargo test` 全 pass 確認
- Material / PurchaseOrder CRUD の動作確認
- 在庫クロスチェックパイプラインの E2E 確認
- Lambda デモ Terraform のプランニング確認
- バグ修正のみ受け入れ（新機能追加なし）

**完了条件**: Rust テスト 2 件（4013 + 2 = 4015）
- `sap_procurement_material_and_po_covered`
- `sap_procurement_scenario3_pipeline_exists`

---

## v89.0.0 — SAP Procurement 1.0 宣言 ★クリーンアップ

**宣言文**:
> 「在庫と発注が型になった。
>  Material と PurchaseOrder を並べれば、不足が見える。」

**クリーンアップ作業:**
- `cargo clean` 実施
- `Cargo.toml` バージョンを `89.0.0` に更新
- `CHANGELOG.md` / `MILESTONE.md` / `README.md` 更新
- `versions/current.md` を v89.0.0 に更新
- driver.rs 内の旧 `cargo_toml_version` テスト（33 件）を `89.0.0` に一括更新

**完了条件**: `v89000_tests` 4 件（4015 + 4 = 4019）
- `cargo_toml_version_is_89_0_0`
- `changelog_has_v89_0_0`
- `milestone_has_sap_procurement`
- `sap_odata_rune_has_material_type`

---

## テスト数推移

| バージョン | テスト数 | 増加 |
|---|---|---|
| v88.0.0（ベース） | 3,997 | — |
| v88.1.0 | 3,999 | +2 |
| v88.2.0 | 4,001 | +2 |
| v88.3.0 | 4,003 | +2 |
| v88.4.0 | 4,005 | +2 |
| v88.5.0 | 4,007 | +2 |
| v88.6.0 | 4,009 | +2 |
| v88.7.0 | 4,011 | +2 |
| v88.8.0 | 4,013 | +2 |
| v88.9.0 | 4,015 | +2 |
| v89.0.0（宣言） | 4,019 | +4 |

**本スプリント合計**: +22 tests
