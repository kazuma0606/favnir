# Plan: v88.7.0 — `StockAlert` 型 + `detect_stock_shortage()` 完全化

## 実装ステップ

### Step 1: `runes/sap-odata/stock.fav` に `format_stock_alerts` を追加

`detect_stock_shortage()` の直後に追加:

```favnir
-- 在庫不足アラートを人間可読な文字列にフォーマットする（v88.7.0）
fn format_stock_alerts(alerts: List<StockAlert>) -> String {
    String.concat(["Stock Alerts: ", Int.to_string(List.length(alerts)), " items"])
}
```

### Step 2: `fav/src/driver.rs` に `mod v88700_tests` を追加

`mod v88600_tests { ... }` の直後に追加:

```rust
#[cfg(test)]
mod v88700_tests {
    #[test]
    fn detect_stock_shortage_function_exists() {
        let content = std::fs::read_to_string("../runes/sap-odata/stock.fav")
            .expect("runes/sap-odata/stock.fav should exist");
        assert!(content.contains("public fn detect_stock_shortage("), "detect_stock_shortage should be defined in stock.fav");
    }
    #[test]
    fn format_stock_alerts_function_exists() {
        let content = std::fs::read_to_string("../runes/sap-odata/stock.fav")
            .expect("runes/sap-odata/stock.fav should exist");
        assert!(content.contains("format_stock_alerts"), "format_stock_alerts should be defined in stock.fav");
    }
}
```

### Step 3: `cargo test` で全 pass 確認

4,009 + 2 = 4,011 tests, 0 failures を確認する。

---

**Note**: CHANGELOG / MILESTONE / site MDX 更新は v89.0.0 宣言バージョンでまとめて実施するため、本バージョンでは省略する。
