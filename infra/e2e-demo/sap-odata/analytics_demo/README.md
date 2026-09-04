# SAP KPI Monitor — E2E デモ（v98.7.0）

## 概要

日次売上 KPI を SAP OData から取得し、SAC（SAP Analytics Cloud）へプッシュした後、
閾値超えアラートを生成する end-to-end pipeline デモ。

- **pipeline**: `kpi_monitor`（`!SapOData !SapAnalytics`）
- **ファイル**: `pipeline_kpi_monitor.fav`

## Pipeline フロー

```
Fetch → Evaluate → Push → Alert
```

| Stage    | 処理内容                                      |
|----------|-----------------------------------------------|
| Fetch    | `ctx.sap.sales_orders()` で売上データ取得     |
| Evaluate | `build_sales_report` → `make_kpi_snapshot` で KPI 評価 |
| Push     | `report_to_sac_rows` → `sac_push_mock` で SAC プッシュ |
| Alert    | `KpiAlert` 生成 → `format_kpi_alert` でメッセージ整形 |

## 前提条件

- `fav` CLI がインストール済みであること
- `sap-odata` Rune が利用可能であること（`runes/sap-odata/`）

## 実行手順

```bash
bash run.sh
```

または直接：

```bash
fav run pipeline_kpi_monitor.fav
```

## KPI 閾値設定

| KPI           | Warning        | Critical        |
|---------------|----------------|-----------------|
| DailyRevenue  | 500,000 JPY    | 1,000,000 JPY   |
