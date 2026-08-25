# Plan: v87.7.0 — `SalesReport` 集計型 + `group_by_currency()`

## 実装ステップ

### Step 1: `runes/sap-odata/sales_report.fav` を新規作成

`sales_order.fav` の型定義（`SalesOrder` / `SalesReport` / `CurrencyTotal`）を参照するため
`use sap_odata.types` と `use sap_odata.sales_order` を先頭に追加する。

3 関数を定義:
1. `fn group_by_currency(orders: List<SalesOrder>) -> List<CurrencyTotal>` — 内部ヘルパー（スタブ: `List.empty()` を返す）
2. `public fn build_sales_report(date: String, orders: List<SalesOrder>) -> Result<SalesReport, String>` — SalesReport 生成（基本実装）
3. `public fn format_sales_report(report: SalesReport) -> String` — テキスト形式フォーマット（スタブ）

### Step 2: `runes/sap-odata/sap_odata.fav` を更新（re-export）

- `use sap_odata.sales_order` の直後に `use sap_odata.sales_report` を追加する
- `create_sales_order` ラッパーの直後に以下を追加する:
  ```favnir
  public fn build_sales_report(date: String, orders: List<SalesOrder>) -> Result<SalesReport, String> {
      sales_report.build_sales_report(date, orders)
  }
  public fn format_sales_report(report: SalesReport) -> String {
      sales_report.format_sales_report(report)
  }
  ```

### Step 3: `infra/e2e-demo/sap-odata/pipeline.fav` を更新

- ローカルスタブ（`-- ヘルパー関数スタブ（v87.7.0 で本実装）` コメント + `fn build_sales_report { ... }`）を削除する
- `daily_sales_report()` 内の `build_sales_report(...)` 呼び出しを `sap_odata.build_sales_report(...)` に変更する

### Step 4: `fav/src/driver.rs` に `mod v87700_tests` を追加

`mod v87600_tests { ... }` の直後に追加:

```rust
#[cfg(test)]
mod v87700_tests {
    #[test]
    fn group_by_currency_function_exists() {
        let content = std::fs::read_to_string("../runes/sap-odata/sales_report.fav")
            .expect("sales_report.fav should exist");
        assert!(content.contains("fn group_by_currency("), "group_by_currency should be defined in sales_report.fav");
    }
    #[test]
    fn format_sales_report_function_exists() {
        let content = std::fs::read_to_string("../runes/sap-odata/sales_report.fav")
            .expect("sales_report.fav should exist");
        assert!(content.contains("fn format_sales_report("), "format_sales_report should be defined in sales_report.fav");
    }
}
```

### Step 5: `cargo test` で全 pass 確認

3,987 + 2 = 3,989 tests, 0 failures を確認する。

---

**Note**: CHANGELOG / MILESTONE / site MDX 更新は v88.0.0 宣言バージョンでまとめて実施するため、本バージョンでは省略する。
