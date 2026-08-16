# v75.6.0 実装計画 — Stream freshness monitoring

Date: 2026-08-15
Status: 計画中

---

## 実装ステップ（依存順）

### Step 1: driver.rs — `StreamFreshnessMonitor` 構造体追加

`fav/src/driver.rs` の末尾（v75.5.0 ブロックの後）に追加する。

```rust
// --- v75.6.0: Stream freshness monitoring ---

/// ストリームの遅延（lag）を監視する設定。
#[derive(Debug, Clone)]
pub struct StreamFreshnessMonitor {
    pub source:       String,  // ストリームソース識別子（例: "kafka://orders-topic"）
    pub max_lag_secs: u64,     // 許容最大遅延（秒）
}
```

### Step 2: driver.rs — `StreamLagResult` 構造体追加

```rust
/// `check_stream_lag` の判定結果。
#[derive(Debug, Clone)]
pub struct StreamLagResult {
    pub lag_secs: u64,    // 実際の遅延（秒）。未来タイムスタンプの場合は 0
    pub exceeded: bool,   // lag_secs > max_lag_secs のとき true
    pub source:   String, // StreamFreshnessMonitor.source のコピー
}
```

### Step 3: driver.rs — `check_stream_lag` 関数追加

```rust
/// 最後のイベントタイムスタンプと現在時刻からストリーム遅延を判定する。
///
/// # 判定ロジック
/// `lag_secs = max(now - last_event_ts, 0)` として計算する。
/// `now < last_event_ts`（未来タイムスタンプ）は lag = 0 として扱う。
/// `exceeded = lag_secs > max_lag_secs`（開区間。ちょうど max_lag_secs 秒は exceeded=false）。
pub fn check_stream_lag(
    last_event_ts: i64,
    now:           i64,
    monitor:       &StreamFreshnessMonitor,
) -> StreamLagResult {
    let lag_secs = now.saturating_sub(last_event_ts).max(0) as u64;
    let exceeded = lag_secs > monitor.max_lag_secs;
    StreamLagResult {
        lag_secs,
        exceeded,
        source: monitor.source.clone(),
    }
}
```

### Step 4: driver.rs — `format_stream_lag_report` 関数追加

```rust
/// ストリーム遅延レポートを人間が読める文字列で返す。
///
/// フォーマット:
/// - 正常時: `"[OK] source={source} lag={lag_secs}s"`
/// - 超過時: `"[EXCEEDED] source={source} lag={lag_secs}s"`
pub fn format_stream_lag_report(result: &StreamLagResult) -> String {
    let status = if result.exceeded { "[EXCEEDED]" } else { "[OK]" };
    format!("{status} source={} lag={}s", result.source, result.lag_secs)
}
```

### Step 4.5: cargo check 確認

`cargo check` でコンパイルエラーがないことを確認する。

### Step 5: CHANGELOG.md 更新（テスト追加より先）

`CHANGELOG.md` の先頭に v75.6.0 エントリを追加する。

### Step 6: driver.rs — テストモジュール追加

```rust
#[cfg(test)]
mod v756000_tests {
    use super::*;

    #[test]
    fn stream_lag_within_threshold() {
        let monitor = StreamFreshnessMonitor {
            source:       "kafka://orders-topic".to_string(),
            max_lag_secs: 30,
        };
        // lag=20 秒 → exceeded=false
        let result = check_stream_lag(100, 120, &monitor);
        assert_eq!(result.lag_secs, 20, "lag_secs must be 20");
        assert!(!result.exceeded, "20s lag must not exceed 30s threshold");
        // ちょうど 30 秒 → exceeded=false（開区間）
        let result2 = check_stream_lag(100, 130, &monitor);
        assert!(!result2.exceeded, "exactly at threshold must not exceed");
        // source フィールドの転写確認
        assert_eq!(result.source, "kafka://orders-topic", "source must be copied from monitor");
        // format report
        let report = format_stream_lag_report(&result);
        assert!(report.contains("[OK]"), "report must contain [OK]");
        assert!(report.contains("kafka://orders-topic"), "report must contain source");
    }

    #[test]
    fn stream_lag_exceeded_detected() {
        let monitor = StreamFreshnessMonitor {
            source:       "kafka://orders-topic".to_string(),
            max_lag_secs: 30,
        };
        // lag=31 秒 → exceeded=true
        let result = check_stream_lag(100, 131, &monitor);
        assert_eq!(result.lag_secs, 31, "lag_secs must be 31");
        assert!(result.exceeded, "31s lag must exceed 30s threshold");
        // format report
        let report = format_stream_lag_report(&result);
        assert!(report.contains("[EXCEEDED]"), "report must contain [EXCEEDED]");
        // 未来タイムスタンプ → lag=0, exceeded=false
        let result2 = check_stream_lag(100, 99, &monitor);
        assert_eq!(result2.lag_secs, 0, "future event must have lag=0");
        assert!(!result2.exceeded, "future event must not exceed");
    }
}
```

### Step 7: Cargo.toml・driver.rs バージョン更新

- `Cargo.toml`: `"75.5.0"` → `"75.6.0"`
- `driver.rs` 内の `version = \"75.5.0\"` を `replace_all` で `version = \"75.6.0\"` に更新

### Step 8: versions/current.md 更新

- 「進行中バージョン」を v75.6.0 に更新
- 「次に切る版」を v75.7.0 に更新

### Step 9: 最終確認

- `cargo test` 全件 pass（3704 tests）
- `cargo test v756000` 2 件 pass

---

## 依存関係

```
Step 1 (StreamFreshnessMonitor)
  └→ Step 3 (check_stream_lag — monitor 引数の型)
Step 2 (StreamLagResult)
  └→ Step 3 (check_stream_lag の戻り型)
  └→ Step 4 (format_stream_lag_report の引数型)
Step 3, 4 (関数)
  └→ Step 6 (テスト)
Step 5 (CHANGELOG) — Step 6 より先に実施
Step 7 (バージョン更新) — Step 6 完了後
Step 8 (current.md) — Step 7 完了後
Step 9 (最終確認) — Step 7, 8 完了後
```

---

## リスク

- `now.saturating_sub(last_event_ts).max(0) as u64` — `now < last_event_ts` の場合（例: `now=99, last_event_ts=100` → `-1`）に `.max(0)` が 0 を返すことを確認すること（`(-1i64).max(0) = 0` ✓）
- `source` フィールドの URL バリデーションは呼び出し側の責任（doc コメントに明記）
