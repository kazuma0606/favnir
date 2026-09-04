# Spec: v98.5.0 — KPI 閾値アラート + Slack / メール通知

## Background

v98.1.0 で `KpiDefinition<T>` / `KpiSnapshot<T>` / `measure_kpi_status<T>` を実装済み。
v98.5.0 では、KPI 評価結果が閾値を超えた場合にアラートを発行するための型とヘルパーを追加する。
通知先（Slack / メール）への実際の送信は `ctx.slack.post()` 等の既存 Rune 経由で行う想定のため、
本バージョンのスコープは `KpiAlert` 型と `format_kpi_alert` ヘルパーの定義に限定する。

## Goals

1. `runes/sap-odata/analytics.fav` — `KpiAlert` 型と `format_kpi_alert` ヘルパーを追加
2. `runes/sap-odata/sap_odata.fav` — `KpiAlert` / `format_kpi_alert` の re-export を追加
3. `fav/src/driver.rs` — `mod v98500_tests`（2 テスト）追加

## Syntax / API Examples

### KpiAlert 型定義（analytics.fav に追加）

```favnir
-- KPI 閾値超えアラート
public type KpiAlert = {
    kpi_name: String,
    status:   KpiStatus,
    message:  String
}
```

### format_kpi_alert ヘルパー（analytics.fav に追加）

```favnir
-- KpiAlert を人間が読める文字列にフォーマットする
-- 例: "[CRITICAL] Revenue: 15000.0"
-- 例: "[WARNING] OrderCount: 450.0"
public fn format_kpi_alert(alert: KpiAlert) -> String {
    bind level <- match alert.status {
        Ok          -> "OK"
        Warning(_)  -> "WARNING"
        Critical(_) -> "CRITICAL"
    }
    String.concat(["[", level, "] ", alert.kpi_name, ": ", alert.message])
}
```

### 利用例（pipeline 等での使用イメージ）

```favnir
bind alerts <- List.filter(snaps, |s| s.status != Ok)
bind _      <- List.map(alerts, |snap|
    ctx.slack.post("#sap-alerts", format_kpi_alert(KpiAlert {
        kpi_name: snap.kpi.name,
        status:   snap.status,
        message:  Float.to_string(snap.value)
    }))
)
```

> **Note**: `s.status != Ok` は Favnir の等値比較で `KpiStatus` バリアントの比較。
> `Ok` バリアントのみを除外し、`Warning(_)` / `Critical(_)` をアラート対象とする。

### sap_odata.fav — re-export 追加

```favnir
-- Analytics re-export（v98.1.0〜）ブロック末尾に追加:
public type KpiAlert = analytics.KpiAlert
public fn format_kpi_alert(alert: analytics.KpiAlert) -> String {
    analytics.format_kpi_alert(alert)
}
```

## Success Criteria

- `runes/sap-odata/analytics.fav` に `KpiAlert` 型と `format_kpi_alert` 関数が含まれる
- `runes/sap-odata/sap_odata.fav` に `KpiAlert` / `format_kpi_alert` の re-export が含まれる
- `format_kpi_alert` の出力形式が `"[LEVEL] kpi_name: message"` であること
  - 例: `KpiAlert { kpi_name: "Revenue", status: Critical(15000.0), message: "15000.0" }` → `"[CRITICAL] Revenue: 15000.0"`
  - 例: `KpiAlert { kpi_name: "OrderCount", status: Warning(450.0), message: "450.0" }` → `"[WARNING] OrderCount: 450.0"`
- `cargo test -- --test-threads=1` が 4,245 tests, 0 failures で通過する

## Error Codes

新規エラーコードなし。

## Files to Modify

| ファイル | 変更種別 |
|---|---|
| `runes/sap-odata/analytics.fav` | 追記（`KpiAlert` 型 + `format_kpi_alert`） |
| `runes/sap-odata/sap_odata.fav` | 追記（`KpiAlert` / `format_kpi_alert` re-export） |
| `fav/src/driver.rs` | 追記（`mod v98500_tests`） |
| `CHANGELOG.md` | 追記 |
| `versions/current.md` | 更新 |
