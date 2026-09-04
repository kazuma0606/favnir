# Plan: v98.4.0 — レポート自動生成 pipeline

## 実装順序

### Step 1: effect_catalog.rs に SAP_ANALYTICS 定数を追加

`fav/src/effect_catalog.rs` に追記：

```rust
/// SAP Analytics Cloud へのデータプッシュを伴う pipeline に付与するエフェクトマーカー
pub const SAP_ANALYTICS: &str = "SapAnalytics";
```

依存なし。先行して実施する。

> **Note**: ロードマップの `## v98.4.0` 末尾に「Effect::SapAnalytics を Effect enum に追加」という注記があるが、
> Rust の Effect enum は v35.4.0 で削除済み（`effect_catalog.rs` NOTE 参照）。
> `effect_catalog.rs` への文字列定数追加がその代替実装であり、本バージョンのスコープはこれで完了。

---

### Step 2: checker.fav の ns_to_effect に "Sac" ブランチを追加

`fav/self/checker.fav` の `ns_to_effect` 関数内、`if ns == "Grafana" { "IO" }` ブロックの直前に追加：

```favnir
if ns == "Sac" {
    "SapAnalytics"
} else {
```

Step 1 と独立して実施可能。

---

### Step 3: sac.fav に report_to_sac_rows を追加

`runes/sap-odata/sac.fav` に追記：

```favnir
use sap_odata.sales_order

-- SalesReport を SAC CSV 行リストに変換するヘルパー
-- ヘッダー行: "Date,Currency,Amount"
-- データ行: "YYYY-MM-DD,USD,12345.0" 形式
-- SalesReport フィールド: report_date: String / by_currency: List<CurrencyTotal>（sales_order.fav L88-93）
public fn report_to_sac_rows(report: sales_order.SalesReport) -> Result<List<String>, String> {
    bind header <- Result.ok("Date,Currency,Amount")
    bind rows   <- Result.ok(List.map(report.by_currency, |t|
        String.concat([report.report_date, ",", t.currency, ",", Float.to_string(t.amount)])
    ))
    Result.ok(List.push([header], rows))
}
```

また `runes/sap-odata/sap_odata.fav` に re-export を追加：

```favnir
public fn report_to_sac_rows(report: sales_order.SalesReport) -> Result<List<String>, String> {
    sac.report_to_sac_rows(report)
}
```

---

### Step 4: pipeline_analytics.fav を新規作成

`infra/e2e-demo/sap-odata/pipeline_analytics.fav` を新規作成。

内容: `daily_sales_report` pipeline（Fetch → Aggregate → Push の 3 ステージ）。

---

### Step 5: driver.rs に mod v98400_tests を追加

`mod v98300_tests` の直後に追加：

```rust
#[cfg(test)]
mod v98400_tests {
    // use super::* は不要（std::fs のみ使用）
    #[test]
    fn pipeline_analytics_fav_exists() {
        std::fs::read_to_string("../infra/e2e-demo/sap-odata/pipeline_analytics.fav")
            .expect("pipeline_analytics.fav should exist");
    }

    #[test]
    fn pipeline_analytics_has_daily_sales_report() {
        let content = std::fs::read_to_string("../infra/e2e-demo/sap-odata/pipeline_analytics.fav")
            .expect("pipeline_analytics.fav should exist");
        assert!(
            content.contains("daily_sales_report"),
            "pipeline_analytics.fav should contain daily_sales_report pipeline"
        );
    }
}
```

---

### Step 6: テスト実行

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -- --test-threads=1 2>&1 | grep "test result"
```

期待値: 4,243 tests, 0 failures

---

### Step 7: CHANGELOG.md に v98.4.0 エントリを追加

先頭に `[v98.4.0]` エントリを追記。

---

### Step 8: versions/current.md 更新

最新安定版を `v98.4.0` に更新（テスト数 4,243）。

---

### Step 9: CI 事前確認

- `cargo clippy --locked -- -D warnings`
- `./target/debug/fav fmt --check self/compiler.fav`
- `./target/debug/fav fmt --check self/checker.fav`
