# Spec: v98.8.0 — サイトドキュメント（Analytics / KPI パターンガイド）

## Background

v98.1.0〜v98.7.0 で SAP Analytics の全機能（KpiDefinition / KpiAlert / SacDataset / fav report --sap / E2E デモ）を実装した。
v98.8.0 では、これらをまとめた開発者向けガイドドキュメント `site/content/docs/guides/sap-analytics.mdx` を新規作成する。

## Goals

1. `site/content/docs/guides/sap-analytics.mdx` — Analytics / KPI パターンガイド新規作成
2. `fav/src/driver.rs` — `mod v98800_tests`（2 テスト）追加

## Syntax / API Examples

### sap-analytics.mdx 構成

```
---
title: "SAP Analytics Guide"
order: 12
category: "Guide"
description: "KPI 定義・BW/4HANA クエリ・SAC データプッシュの完全ガイド"
---
```

**セクション構成**:

1. **概要** — KPI 監視 pipeline の全体像（Fetch → Evaluate → Push → Alert）
2. **KPI 定義パターン** — `KpiDefinition` / `KpiThreshold` / `KpiSnapshot` / `KpiStatus` の使い方
3. **BW/4HANA クエリ** — `ctx.sap.bw_query()` の使い方
4. **SAC データプッシュ** — `SacDataset` / `sac_push_mock` の設定
5. **`fav report --sap` コマンド** — CLI リファレンス（フラグ一覧）

### KPI 定義パターン（コード例）

```favnir
import rune "sap-odata"

pipeline kpi_check !SapOData !SapAnalytics {
    stage Define {
        bind kpi <- Result.ok(KpiDefinition {
            name:      "DailyRevenue",
            unit:      "JPY",
            threshold: KpiThreshold { warning: 500000.0, critical: 1000000.0 },
            extract:   |_| 0.0
        })
    }
    |> stage Evaluate {
        bind snap <- Result.ok(make_kpi_snapshot(kpi, 750000.0, "2026-09-03"))
    }
    |> stage Alert {
        bind alert <- Result.ok(KpiAlert {
            kpi_name: snap.kpi.name,
            status:   snap.status,
            message:  Float.to_string(snap.value)
        })
        bind msg <- Result.ok(format_kpi_alert(alert))
        bind _   <- Result.ok(msg)
    }
}
```

### `fav report --sap` コマンドリファレンス

```bash
fav report --sap [--entity <name>] [--from <date>] [--to <date>] [--output <file>]
```

| フラグ | デフォルト | 説明 |
|---|---|---|
| `--entity` | `SalesOrder` | 対象エンティティ名 |
| `--from` | （空） | 期間開始日 |
| `--to` | （空） | 期間終了日 |
| `--output` | `report.html` | 出力ファイルパス |

## Success Criteria

- `site/content/docs/guides/sap-analytics.mdx` が存在する
- `sap-analytics.mdx` に `KpiDefinition` が含まれる
- `cargo test -- --test-threads=1` が 4,251 tests, 0 failures で通過する

## Error Codes

新規エラーコードなし。

## Files to Modify

| ファイル | 変更種別 |
|---|---|
| `site/content/docs/guides/sap-analytics.mdx` | 新規作成 |
| `fav/src/driver.rs` | 追記（`mod v98800_tests`） |
| `CHANGELOG.md` | 追記 |
| `versions/current.md` | 更新 |
