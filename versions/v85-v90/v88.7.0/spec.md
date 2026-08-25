# Spec: v88.7.0 — `StockAlert` 型 + `detect_stock_shortage()` 完全化

## Background

v88.6.0 で `stock.fav` を新規作成し、`StockSeverity` / `StockAlert` 型と
`detect_stock_shortage()` スタブを定義した。
本バージョンでは `format_stock_alerts()` ヘルパー関数を追加し、
Rust テスト 2 件で `detect_stock_shortage` / `format_stock_alerts` の存在を担保する。
これにより SAP Procurement × Sales クロスチェックの公開 API が揃う。

## Goals

1. `runes/sap-odata/stock.fav` に `format_stock_alerts()` 関数を追加する（スタブ）
2. Rust テスト 2 件で `detect_stock_shortage` / `format_stock_alerts` の存在を担保する

## API / Syntax Examples

```favnir
-- runes/sap-odata/stock.fav に追加（v88.7.0）

-- 在庫不足アラートを人間可読な文字列にフォーマットする
-- Note: public なし（モジュール内ヘルパー。外部公開は pipeline.fav 経由のみ想定）
fn format_stock_alerts(alerts: List<StockAlert>) -> String {
    String.concat(["Stock Alerts: ", Int.to_string(List.length(alerts)), " items"])
}
```

Note: `detect_stock_shortage()` は v88.6.0 でスタブ実装済み（`Result.err("not implemented")` 返し）。
本バージョンでは本実装は行わず、Rust テストで存在を確認するのみ。

## Success Criteria（Rust テストで担保）

- `runes/sap-odata/stock.fav` に以下を含む:
  - `"public fn detect_stock_shortage("` — v88.6.0 追加済み
  - `"format_stock_alerts"` — 本バージョンで追加
- `cargo test` で 4,011 tests, 0 failures（T0 で確認したベースライン 4,009 + 2）
- Rust テスト 2 件:
  - `detect_stock_shortage_function_exists`（`stock.fav` に `"public fn detect_stock_shortage("` を確認）
  - `format_stock_alerts_function_exists`（`stock.fav` に `"format_stock_alerts"` を確認）

## Files to Modify

| ファイル | 変更種別 |
|---|---|
| `runes/sap-odata/stock.fav` | 追記（`format_stock_alerts()` 関数追加） |
| `fav/src/driver.rs` | `mod v88700_tests` 追加 |

**Note**: CHANGELOG / MILESTONE / site MDX 更新は v89.0.0 宣言バージョンでまとめて実施する（本バージョンは対象外）
