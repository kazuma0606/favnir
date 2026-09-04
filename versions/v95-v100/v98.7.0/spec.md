# Spec: v98.7.0 — E2E デモ（日次 KPI → SAC → Slack）

## Background

v98.1.0〜v98.6.0 で SAP Analytics の型（KpiDefinition/KpiAlert/SacDataset）・pipeline・CLIを実装した。
v98.7.0 では、これらを組み合わせた完全な E2E デモディレクトリ `analytics_demo/` を新規作成し、
「日次売上 KPI 計算 → SAC プッシュ → 閾値超え Slack アラート」の end-to-end フローを
`pipeline_kpi_monitor.fav` として実装する。

## Goals

1. `infra/e2e-demo/sap-odata/analytics_demo/pipeline_kpi_monitor.fav` — E2E デモ pipeline 新規作成
2. `infra/e2e-demo/sap-odata/analytics_demo/run.sh` — 実行スクリプト新規作成
3. `infra/e2e-demo/sap-odata/analytics_demo/README.md` — デモ概要ドキュメント新規作成
4. `fav/src/driver.rs` — `mod v98700_tests`（2 テスト）追加

## Language Design Notes

### Pipeline stage 環境継承（`|>` チェーン）

Favnir の `|>` pipeline ではすべての stage が **同一の環境（binding スコープ）を共有**する。
前の stage で `bind` した変数は後続の stage から参照できる（`pipeline_analytics.fav` で実証済み：
Fetch ステージの `orders` が Aggregate ステージで参照されている）。

本 pipeline では：
- `report` は Evaluate stage で束縛 → Push stage・Alert stage から参照可
- `snap` は Evaluate stage で束縛 → Alert stage から参照可

### 2 段フィールドアクセス（`snap.kpi.name`）

Favnir は `record.field1.field2` のネストアクセスをサポートする。
`analytics.fav` の `measure_kpi_status` 関数でも `kpi.threshold.critical` / `kpi.threshold.warning` として使用済み（実証済み）。
`snap` は `KpiSnapshot { kpi: KpiDefinition, ... }` 型で、`snap.kpi.name` は有効な 2 段アクセス。

### `extract` フィールドについて

`KpiDefinition.extract` フィールド（`|_| 0.0`）は API 互換のために指定するが、
本 pipeline では `report.total_amount` を直接 `make_kpi_snapshot` に渡すため使用しない。

## Syntax / API Examples

### pipeline_kpi_monitor.fav

```favnir
import rune "sap-odata"

-- 日次売上 KPI 監視 pipeline（v98.7.0）
-- 売上データ取得 → KPI 評価 → SAC プッシュ → Slack アラート送信
pipeline kpi_monitor !SapOData !SapAnalytics {
    stage Fetch {
        bind orders <- ctx.sap.sales_orders(SalesOrderFilter {
            date_from: Option.some("2026-09-02"),
            date_to:   Option.none(),
            top:       Option.some(5000)
        })
    }
    |> stage Evaluate {
        bind report  <- build_sales_report("2026-09-02", orders)
        bind kpi_def <- Result.ok(KpiDefinition {
            name:      "DailyRevenue",
            unit:      "JPY",
            threshold: KpiThreshold { warning: 500000.0, critical: 1000000.0 },
            extract:   |_| 0.0
        })
        bind snap    <- Result.ok(make_kpi_snapshot(kpi_def, report.total_amount, "2026-09-02"))
    }
    |> stage Push {
        bind rows <- report_to_sac_rows(report)
        bind _    <- Result.ok(sac_push_mock(SacDataset {
            model_id: "FAVNIR_DAILY_KPI",
            rows:     rows
        }))
    }
    |> stage Alert {
        bind alert <- Result.ok(KpiAlert {
            kpi_name: snap.kpi.name,
            status:   snap.status,
            message:  Float.to_string(snap.value)
        })
        bind msg   <- Result.ok(format_kpi_alert(alert))
        bind _     <- Result.ok(msg)
    }
}
```

> **Note**: `ctx.slack.post()` は Rune Registry に登録済み（`runes/slack/`）だが、
> 本デモではモック関数のみを使用する。`msg` を束縛することで将来の Slack 送信を示す。

### run.sh

```bash
#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# SAP KPI Monitor E2E デモ実行スクリプト（v98.7.0）
fav run "${SCRIPT_DIR}/pipeline_kpi_monitor.fav"
```

### README.md

- デモの概要（KPI 監視 pipeline）
- 前提条件（fav CLI / SAP Rune）
- 実行手順（`bash run.sh`）
- pipeline フロー図（Fetch → Evaluate → Push → Alert）

## Success Criteria

- `infra/e2e-demo/sap-odata/analytics_demo/pipeline_kpi_monitor.fav` が存在する
- `pipeline_kpi_monitor.fav` に `KpiAlert` が含まれる
- `cargo test -- --test-threads=1` が 4,249 tests, 0 failures で通過する

## Error Codes

新規エラーコードなし。

## Files to Modify

| ファイル | 変更種別 |
|---|---|
| `infra/e2e-demo/sap-odata/analytics_demo/pipeline_kpi_monitor.fav` | 新規作成 |
| `infra/e2e-demo/sap-odata/analytics_demo/run.sh` | 新規作成 |
| `infra/e2e-demo/sap-odata/analytics_demo/README.md` | 新規作成 |
| `fav/src/driver.rs` | 追記（`mod v98700_tests`） |
| `CHANGELOG.md` | 追記 |
| `versions/current.md` | 更新 |
