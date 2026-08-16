# v75.8.0 実装計画 — `fav time-travel` コマンド

Date: 2026-08-15

---

## Step 1: TimeTravelFormat enum 追加

`fav/src/driver.rs` の末尾に以下を追加：

```rust
// --- v75.8.0: fav time-travel コマンド ---
#[derive(Debug, Clone, PartialEq)]
pub enum TimeTravelFormat {
    Snowflake,
    Delta,
    Generic,
}
```

---

## Step 2: TimeTravelQuery 構造体追加

```rust
#[derive(Debug, Clone)]
pub struct TimeTravelQuery {
    pub table:    String,
    pub as_of_ts: i64,
    pub format:   TimeTravelFormat,
}
```

---

## Step 3: cmd_time_travel 関数追加

- `Snowflake`: `unix_secs_to_utc` を再利用して UTC 日時文字列を生成
- `Delta`: `VERSION AS OF {as_of_ts}`
- `Generic`: `WHERE _timestamp = {as_of_ts}`

```rust
pub fn cmd_time_travel(query: &TimeTravelQuery) -> String {
    match &query.format {
        TimeTravelFormat::Snowflake => {
            let (y, mo, d, h, mi, s) = unix_secs_to_utc(query.as_of_ts);
            format!(
                "SELECT * FROM {} AS OF TIMESTAMP '{:04}-{:02}-{:02} {:02}:{:02}:{:02}'",
                query.table, y, mo, d, h, mi, s
            )
        }
        TimeTravelFormat::Delta => {
            format!("SELECT * FROM {} VERSION AS OF {}", query.table, query.as_of_ts)
        }
        TimeTravelFormat::Generic => {
            format!("SELECT * FROM {} WHERE _timestamp = {}", query.table, query.as_of_ts)
        }
    }
}
```

---

## Step 4: parse_time_travel_timestamp 関数追加

- フォーマット: `"YYYY-MM-DDTHH:MM:SSZ"` (length=20)
- 区切り文字検証: `s[4]=='-'`, `s[7]=='-'`, `s[10]=='T'`, `s[13]==':'`, `s[16]==':'`, `s[19]=='Z'`
- year >= 1970 のみ許可
- `is_leap` を再利用して 1970 からの日数を計算

```rust
pub fn parse_time_travel_timestamp(s: &str) -> Result<i64, String> {
    if s.len() != 20 { return Err(format!("invalid timestamp: {s}")); }
    let b = s.as_bytes();
    if b[4] != b'-' || b[7] != b'-' || b[10] != b'T' || b[13] != b':' || b[16] != b':' || b[19] != b'Z' {
        return Err(format!("invalid timestamp format: {s}"));
    }
    let year:  i32 = s[0..4].parse().map_err(|_| format!("invalid year: {s}"))?;
    let month: u32 = s[5..7].parse().map_err(|_| format!("invalid month: {s}"))?;
    let day:   u32 = s[8..10].parse().map_err(|_| format!("invalid day: {s}"))?;
    let hour:  i64 = s[11..13].parse().map_err(|_| format!("invalid hour: {s}"))?;
    let min:   i64 = s[14..16].parse().map_err(|_| format!("invalid min: {s}"))?;
    let sec:   i64 = s[17..19].parse().map_err(|_| format!("invalid sec: {s}"))?;
    if year < 1970 { return Err(format!("year must be >= 1970: {year}")); }
    if month < 1 || month > 12 { return Err(format!("invalid month: {month}")); }
    // 1970-01-01 からの日数を計算
    let mut days: i64 = 0;
    for y in 1970..year {
        days += if is_leap(y) { 366 } else { 365 };
    }
    let month_days: [u32; 12] = [31, if is_leap(year) { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let max_day = month_days[(month - 1) as usize];
    if day < 1 || day > max_day { return Err(format!("invalid day: {day}")); }
    for m in 0..(month - 1) as usize {
        days += month_days[m] as i64;
    }
    days += (day - 1) as i64;
    Ok(days * 86400 + hour * 3600 + min * 60 + sec)
}
```

---

## Step 4.5: cargo check

`cargo check` でコンパイルエラーがないことを確認する。

---

## Step 5: CHANGELOG.md 更新

`CHANGELOG.md` の先頭に v75.8.0 エントリを追加（テスト追加より先に実施）。

---

## Step 6: テストモジュール v758000_tests 追加

```rust
#[cfg(test)]
mod v758000_tests {
    use super::*;

    #[test]
    fn time_travel_snowflake_format() {
        let query = TimeTravelQuery {
            table: "orders".to_string(),
            as_of_ts: 1735689600,
            format: TimeTravelFormat::Snowflake,
        };
        let sql = cmd_time_travel(&query);
        assert!(sql.contains("orders"));
        assert!(sql.contains("AS OF TIMESTAMP '2025-01-01 00:00:00'"));
        assert_eq!(parse_time_travel_timestamp("2025-01-01T00:00:00Z"), Ok(1735689600));
        assert!(parse_time_travel_timestamp("invalid").is_err());
    }

    #[test]
    fn time_travel_delta_format() {
        let query = TimeTravelQuery {
            table: "orders".to_string(),
            as_of_ts: 1735689600,
            format: TimeTravelFormat::Delta,
        };
        assert!(cmd_time_travel(&query).contains("VERSION AS OF 1735689600"));
        let query2 = TimeTravelQuery {
            table: "orders".to_string(),
            as_of_ts: 1735689600,
            format: TimeTravelFormat::Generic,
        };
        assert!(cmd_time_travel(&query2).contains("WHERE _timestamp = 1735689600"));
    }
}
```

---

## Step 7: Cargo.toml バージョン更新

`fav/Cargo.toml`: `75.7.0` → `75.8.0`
`driver.rs` 内のバージョン文字列アサーションも一括更新。

---

## Step 8: versions/current.md 更新

- 進行中バージョン: v75.8.0
- 次に切る版: v75.9.0

---

## Step 9: 最終確認

`cargo test` が 3708 tests all pass であることを確認。
