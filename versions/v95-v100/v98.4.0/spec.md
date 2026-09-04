# Spec: v98.4.0 — レポート自動生成 pipeline

## Background

v98.1.0〜v98.3.0 で SAP Analytics の型定義（`KpiDefinition<T>` / `BwQuery<T>` / `SacDataset`）を完了した。
v98.4.0 では、これらの型を組み合わせた E2E pipeline `daily_sales_report` を実装し、
SAP 売上データ → 集計レポート → SAC データプッシュの流れを Favnir pipeline で表現する。

あわせて、SAP Analytics 系エフェクト（`!SapAnalytics`）を effect_catalog.rs と checker.fav に登録し、
パイプラインシグネチャとの整合性を確保する。

## Goals

1. `infra/e2e-demo/sap-odata/pipeline_analytics.fav` — `daily_sales_report` pipeline 新規作成
2. `runes/sap-odata/sac.fav` — `report_to_sac_rows` ヘルパー追加
3. `fav/src/effect_catalog.rs` — `SAP_ANALYTICS` 定数追加
4. `fav/self/checker.fav` — `ns_to_effect` に `"Sac"` → `"SapAnalytics"` 追加
5. `fav/src/driver.rs` — `mod v98400_tests`（2 テスト）追加

## Syntax / API Examples

### pipeline_analytics.fav

> **Note**: `today()` は Favnir stdlib に存在しない。実装では `"2026-09-02"` のようなリテラル文字列を使うか、
> `build_sales_report` の第 1 引数（`date: String`）に固定値を渡す形にする。

```favnir
use sap_odata

-- 日次売上レポートを生成して SAC にプッシュする pipeline
pipeline daily_sales_report !SapOData !SapAnalytics {
    stage Fetch {
        bind orders <- ctx.sap.sales_orders(SalesOrderFilter {
            date_from: Option.some("2026-09-02"),
            date_to:   Option.none(),
            top:       Option.some(5000)
        })
    }
    |> stage Aggregate {
        bind report <- build_sales_report("2026-09-02", orders)
    }
    |> stage Push {
        bind rows <- report_to_sac_rows(report)
        bind _    <- ctx.sap.sac_push(SacDataset {
            model_id: "FAVNIR_DAILY_SALES",
            rows:     rows
        })
    }
}
```

### sac.fav — report_to_sac_rows ヘルパー

```favnir
-- SalesReport を SAC CSV 行リストに変換するヘルパー
-- 先頭行はヘッダー行（"Date,Currency,Amount"）、以降がデータ行
public fn report_to_sac_rows(report: sales_order.SalesReport) -> Result<List<String>, String> {
    bind header <- Result.ok("Date,Currency,Amount")
    bind rows   <- Result.ok(List.map(report.by_currency, |t|
        String.concat([report.report_date, ",", t.currency, ",", Float.to_string(t.amount)])
    ))
    Result.ok(List.push([header], rows))
}
```

> **Note**: `SalesReport` は `sales_order.fav` で定義済み（`report_date: String` / `by_currency: List<CurrencyTotal>`）。
> `CurrencyTotal` は `{ currency: String, amount: Float }` 。

### effect_catalog.rs — SAP_ANALYTICS 定数

```rust
/// SAP Analytics Cloud へのデータプッシュを伴う pipeline に付与するエフェクトマーカー
pub const SAP_ANALYTICS: &str = "SapAnalytics";
```

### checker.fav — ns_to_effect 追記

`ns_to_effect` の末尾 `if ns == "Grafana" { "IO" }` ブロックの直前に追加：

```favnir
if ns == "Sac" {
    "SapAnalytics"
} else {
    ...
```

## Success Criteria

- `infra/e2e-demo/sap-odata/pipeline_analytics.fav` が存在し `daily_sales_report` を含む
- `runes/sap-odata/sac.fav` に `report_to_sac_rows` が含まれる
- `fav/src/effect_catalog.rs` に `SAP_ANALYTICS` 定数が存在する
- `fav/self/checker.fav` の `ns_to_effect` に `"Sac"` ブランチが存在する
- `cargo test -- --test-threads=1` が 4,243 tests, 0 failures で通過する

## Error Codes

新規エラーコードなし。

## Files to Modify

| ファイル | 変更種別 |
|---|---|
| `infra/e2e-demo/sap-odata/pipeline_analytics.fav` | 新規作成 |
| `runes/sap-odata/sac.fav` | 追記（`report_to_sac_rows`） |
| `fav/src/effect_catalog.rs` | 追記（`SAP_ANALYTICS` 定数） |
| `fav/self/checker.fav` | 追記（`ns_to_effect` に `"Sac"` ブランチ） |
| `fav/src/driver.rs` | 追記（`mod v98400_tests`） |
| `CHANGELOG.md` | 追記 |
| `versions/current.md` | 更新 |
