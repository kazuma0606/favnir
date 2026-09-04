# Plan: v98.5.0 — KPI 閾値アラート + Slack / メール通知

## 実装順序

### Step 1: analytics.fav に KpiAlert 型と format_kpi_alert を追加

`runes/sap-odata/analytics.fav` の末尾（`bw_query_mock<T>` の後）に追記：

```favnir
-- KPI 閾値超えアラート型（v98.5.0〜）
public type KpiAlert = {
    kpi_name: String,
    status:   KpiStatus,
    message:  String
}

-- KpiAlert を人間が読める文字列にフォーマットする（v98.5.0〜）
-- 例: "[CRITICAL] Revenue: 15000.0"
public fn format_kpi_alert(alert: KpiAlert) -> String {
    bind level <- match alert.status {
        Ok          -> "OK"
        Warning(_)  -> "WARNING"
        Critical(_) -> "CRITICAL"
    }
    String.concat(["[", level, "] ", alert.kpi_name, ": ", alert.message])
}
```

コメントが `--` スタイルであることを確認する（`//` 不可）。

---

### Step 2: sap_odata.fav に KpiAlert / format_kpi_alert re-export を追加

> 挿入前に `sap_odata.fav` の Analytics re-export ブロック末尾（`bw_query_mock<T>` の後）の現在の内容を確認してから追記すること。

`runes/sap-odata/sap_odata.fav` の Analytics re-export ブロック末尾（`bw_query_mock` の後、`-- $batch` の前）に追記：

```favnir
public type KpiAlert = analytics.KpiAlert
public fn format_kpi_alert(alert: analytics.KpiAlert) -> String {
    analytics.format_kpi_alert(alert)
}
```

---

### Step 3: driver.rs に mod v98500_tests を追加

`mod v98400_tests` の直後に追加：

```rust
#[cfg(test)]
mod v98500_tests {
    // use super::* は不要（std::fs のみ使用）
    #[test]
    fn analytics_fav_has_kpi_alert() {
        let content = std::fs::read_to_string("../runes/sap-odata/analytics.fav")
            .expect("runes/sap-odata/analytics.fav should exist");
        assert!(
            content.contains("KpiAlert"),
            "analytics.fav should define KpiAlert (v98.5.0)"
        );
    }

    #[test]
    fn analytics_fav_has_format_kpi_alert() {
        let content = std::fs::read_to_string("../runes/sap-odata/analytics.fav")
            .expect("runes/sap-odata/analytics.fav should exist");
        assert!(
            content.contains("format_kpi_alert"),
            "analytics.fav should define format_kpi_alert (v98.5.0)"
        );
    }
}
```

---

### Step 4: テスト実行

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -- --test-threads=1 2>&1 | grep "test result"
```

期待値: 4,245 tests, 0 failures

---

### Step 5: CHANGELOG.md に v98.5.0 エントリを追加

---

### Step 6: versions/current.md 更新

最新安定版を `v98.5.0` に更新（テスト数 4,245）。

---

### Step 7: CI 事前確認

- `cargo clippy --locked -- -D warnings`
- `./target/debug/fav fmt --check self/compiler.fav`
- `./target/debug/fav fmt --check self/checker.fav`
