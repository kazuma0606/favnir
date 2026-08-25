# Roadmap v87.1.0 〜 v88.0.0 — SAP Sales 1.0

Date: 2026-08-22
Status: 未着手（v87.0.0 完了後に開始）

マスターロードマップ: [roadmap-v85.1-v90.0.md](roadmap-v85.1-v90.0.md)

---

## 前提

- 直前完了: v87.0.0「SAP Master Data 1.0 宣言」（tests = 3,975）
- 本スプリントは SAP Integration Era の第 3 スプリント
- 目標: v88.0.0「SAP Sales 1.0 宣言」（tests = 3,997）

### 着手前チェックリスト

- `versions/current.md` の最新安定版が v87.0.0 になっていることを確認する
- `versions/v85-v90/v87.1.0/` ディレクトリを作成し、spec.md / plan.md / tasks.md を準備すること
- `runes/sap-odata/` に BusinessPartner 型・関数が実装済みであることを確認する

### スプリントの性格

受注伝票（`SalesOrder` / `SalesOrderItem`）を型安全に扱えるようにする。
CRUD と日次売上レポートパイプライン（業務シナリオ 2）を実装し、
大量データのページネーション基盤も整備する。
A（新機能）70% + B（統合）30% の構成。

---

## バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v87.1.0 | `SalesOrder` / `SalesOrderItem` 型定義 | 3975 + 2 = 3977 | 未着手 |
| v87.2.0 | `SalesOrderFilter` + `sales_orders()` クエリ | 3977 + 2 = 3979 | 未着手 |
| v87.3.0 | `sales_order_by_id()` + `$expand=to_Item` | 3979 + 2 = 3981 | 未着手 |
| v87.4.0 | `create_sales_order()` + `NewSalesOrder` | 3981 + 2 = 3983 | 未着手 |
| v87.5.0 | シナリオ 2: 日次売上レポート（SalesOrder 集計 → S3） | 3983 + 2 = 3985 | 未着手 |
| v87.6.0 | ページネーション基盤（`$top` / `$skip` / `@odata.nextLink`） | 3985 + 2 = 3987 | 未着手 |
| v87.7.0 | `SalesReport` 集計型 + `group_by_currency()` | 3987 + 2 = 3989 | 未着手 |
| v87.8.0 | モックサーバーテスト — 受注シナリオ全操作検証 | 3989 + 2 = 3991 | 未着手 |
| v87.9.0 | 安定化・コードフリーズ | 3991 + 2 = 3993 | 未着手 |
| v88.0.0 | SAP Sales 1.0 宣言 ★クリーンアップ | 3993 + 4 = 3997 | 未着手 |

---

## v87.1.0 — `SalesOrder` / `SalesOrderItem` 型定義

受注伝票と受注明細の Favnir 型を定義する。

```favnir
type SalesOrderStatus = Open | InProcess | Completed | Cancelled

type SalesOrderItem = {
    item_number: Int,
    material_id: String,
    description: String,
    quantity:    Float,
    unit:        String,
    net_amount:  Float,
    currency:    String
}

type SalesOrder = {
    order_id:     String,
    customer_id:  String,
    status:       SalesOrderStatus,
    total_amount: Float,
    currency:     String,
    sales_org:    String,
    created_at:   String,
    items:        Option<List<SalesOrderItem>>
}
```

**実装ファイル:** `runes/sap-odata/types.fav`（BusinessPartner 型の後に追加）

**完了条件**: Rust テスト 2 件（3975 + 2 = 3977）
- `sales_order_type_defined_in_rune`
- `sales_order_item_type_defined_in_rune`

---

## v87.2.0 — `SalesOrderFilter` + `sales_orders()` クエリ

受注の一覧取得関数を実装する。

```favnir
type SalesOrderFilter = {
    customer_id:    Option<String>,
    status:         Option<SalesOrderStatus>,
    created_after:  Option<String>,
    created_before: Option<String>,
    sales_org:      Option<String>,
    top:            Option<Int>
}

public fn sales_orders(
    cfg:    SapConfig,
    filter: SalesOrderFilter
) -> Result<List<SalesOrder>, String>
```

**実装ファイル:** `runes/sap-odata/sales_order.fav`（新規作成）

**完了条件**: Rust テスト 2 件（3977 + 2 = 3979）
- `sales_orders_function_exists`
- `sales_order_filter_type_exists`

---

## v87.3.0 — `sales_order_by_id()` + `$expand=to_Item`

単一受注の取得と明細 expand を実装する。

```favnir
public fn sales_order_by_id(
    cfg:          SapConfig,
    order_id:     String,
    expand_items: Bool
) -> Result<SalesOrder, String>
```

`expand_items = true` で `$expand=to_Item` を付与し、明細を含む完全な受注を取得する。

**完了条件**: Rust テスト 2 件（3979 + 2 = 3981）
- `sales_order_by_id_function_exists`
- `sales_order_expand_items_in_rune`

---

## v87.4.0 — `create_sales_order()` + `NewSalesOrder`

受注の新規作成を実装する。

```favnir
type NewSalesOrderItem = {
    material_id: String,
    quantity:    Float,
    unit:        String
}

type NewSalesOrder = {
    customer_id: String,
    sales_org:   String,
    currency:    String,
    items:       List<NewSalesOrderItem>
}

public fn create_sales_order(
    cfg:   SapConfig,
    order: NewSalesOrder
) -> Result<SalesOrder, String>
```

**完了条件**: Rust テスト 2 件（3981 + 2 = 3983）
- `create_sales_order_function_exists`
- `new_sales_order_type_exists`

---

## v87.5.0 — シナリオ 2: 日次売上レポート（SalesOrder 集計 → S3）

業務シナリオ 2 の E2E 実装。完了受注を集計して日次売上レポートを生成する。

```favnir
type SalesReport = {
    report_date:  String,
    total_orders: Int,
    total_amount: Float,
    by_currency:  List<CurrencyTotal>
}

type CurrencyTotal = {
    currency: String,
    amount:   Float,
    count:    Int
}

fn daily_sales_report(ctx: AppCtx) -> Result<SalesReport, String> {
    bind cfg    <- sap_odata.sap_config_from_env()
    bind orders <- sap_odata.sales_orders(cfg, SalesOrderFilter {
        status:         Option.some(SalesOrderStatus.Completed),
        created_after:  Option.some("2026-08-22"),
        customer_id:    Option.none(),
        created_before: Option.none(),
        sales_org:      Option.none(),
        top:            Option.none()
    })
    bind report <- build_sales_report("2026-08-22", orders)
    bind json   <- Json.encode(report)
    bind _      <- ctx.s3.put_object("favnir-sap-demo", "reports/daily/2026-08-22.json", json)
    Result.ok(report)
}
```

**完了条件**: Rust テスト 2 件（3983 + 2 = 3985）
- `sales_report_type_exists`
- `sap_e2e_pipeline_contains_daily_sales_report`

---

## v87.6.0 — ページネーション基盤

大量データ（SAP は 100 万件超の受注を持つケースがある）を安全に処理するページネーション。

```favnir
type PagedResult = {
    items:      List<String>,   -- JSON 行のリスト
    next_token: Option<String>  -- @odata.nextLink から抽出した URL
}

fn odata_list_paged(
    cfg:        SapConfig,
    entity_set: String,
    params:     ODataParams,
    page_size:  Int
) -> Result<PagedResult, String>

fn odata_collect_all(
    cfg:        SapConfig,
    entity_set: String,
    params:     ODataParams,
    max_pages:  Int
) -> Result<List<String>, String>
```

**実装ファイル:** `runes/sap-odata/client.fav`（v85.5.0 の `odata_list` の後に追加）

**完了条件**: Rust テスト 2 件（3985 + 2 = 3987）
- `paged_result_type_exists`
- `odata_list_paged_function_exists`

---

## v87.7.0 — `SalesReport` 集計型 + `group_by_currency()`

売上集計ヘルパー関数を実装する。

```favnir
fn group_by_currency(orders: List<SalesOrder>) -> List<CurrencyTotal>
fn build_sales_report(date: String, orders: List<SalesOrder>) -> Result<SalesReport, String>
fn format_sales_report(report: SalesReport) -> String
```

**実装ファイル:** `runes/sap-odata/sales_report.fav`（新規作成）

**完了条件**: Rust テスト 2 件（3987 + 2 = 3989）
- `group_by_currency_function_exists`
- `format_sales_report_function_exists`

---

## v87.8.0 — モックサーバーテスト — 受注シナリオ全操作検証

`sap_odata.test.fav` に SalesOrder CRUD + ページネーションのテストを追加。

**実装内容:**
- SalesOrder 作成・取得・フィルタのテスト
- ページネーション（100 件超）のテスト
- 日次売上レポート生成のテスト

**完了条件**: Rust テスト 2 件（3989 + 2 = 3991）
- `sap_odata_test_contains_sales_order_tests`
- `sap_odata_test_contains_pagination_test`

---

## v87.9.0 — 安定化・コードフリーズ

v87.1〜v87.8 の全機能を通しで確認する安定化スプリント。

**実装内容:**
- `cargo test` 全 pass 確認
- SalesOrder CRUD + ページネーションの動作確認
- 日次売上レポートパイプラインの E2E 確認
- バグ修正のみ受け入れ（新機能追加なし）

**完了条件**: Rust テスト 2 件（3991 + 2 = 3993）
- `sap_sales_order_crud_covered`
- `sap_sales_scenario2_report_pipeline_exists`

---

## v88.0.0 — SAP Sales 1.0 宣言 ★クリーンアップ

**宣言文**:
> 「受注が型になった。
>  `sales_orders()` で絞り込み、`sales_order_by_id()` で明細まで取得できる。
>  日次売上レポートが、Favnir の 10 行で書ける。」

**クリーンアップ作業:**
- `cargo clean` 実施
- `Cargo.toml` バージョンを `88.0.0` に更新
- `CHANGELOG.md` / `MILESTONE.md` / `README.md` 更新
- `versions/current.md` を v88.0.0 に更新
- driver.rs 内の旧 `cargo_toml_version` テスト（33 件）を `88.0.0` に一括更新

**完了条件**: `v88000_tests` 4 件（3993 + 4 = 3997）
- `cargo_toml_version_is_88_0_0`
- `changelog_has_v88_0_0`
- `milestone_has_sap_sales`
- `sap_odata_rune_has_sales_order_type`

---

## テスト数推移

| バージョン | テスト数 | 増加 |
|---|---|---|
| v87.0.0（ベース） | 3,975 | — |
| v87.1.0 | 3,977 | +2 |
| v87.2.0 | 3,979 | +2 |
| v87.3.0 | 3,981 | +2 |
| v87.4.0 | 3,983 | +2 |
| v87.5.0 | 3,985 | +2 |
| v87.6.0 | 3,987 | +2 |
| v87.7.0 | 3,989 | +2 |
| v87.8.0 | 3,991 | +2 |
| v87.9.0 | 3,993 | +2 |
| v88.0.0（宣言） | 3,997 | +4 |

**本スプリント合計**: +22 tests
