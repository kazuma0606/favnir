# v75.2.0 実装計画 — `TemporalRange` / `AsOfQuery` 型

Date: 2026-08-14

---

## 事前確認（T0）

- [ ] `fav/Cargo.toml` のバージョンが `75.1.0` であることを確認
- [ ] `cargo test` が全 pass（3694 tests）であることを確認
- [ ] `fav/tmp/hello.fav` が存在することを確認（bootstrap テスト要件）

---

## ステップ 1: driver.rs に型定義・関数を追加

**ファイル**: `fav/src/driver.rs`

v751000 テストモジュールの後に追加:

```rust
// --- v75.2.0: TemporalRange / AsOfQuery 型 ---

#[derive(Debug, Clone, PartialEq)]
pub struct TemporalRange {
    pub from_ts: i64,
    pub to_ts: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AsOfQuery {
    pub table: String,
    pub as_of_ts: i64,
}

/// タイムスタンプが期間内（両端を含む閉区間）かを判定する。
pub fn is_in_range(ts: i64, range: &TemporalRange) -> bool {
    ts >= range.from_ts && ts <= range.to_ts
}

/// AsOfQuery を Snowflake の AS OF TIMESTAMP SQL 文字列に変換する。
/// フォーマット: SELECT * FROM {table} AS OF TIMESTAMP '{YYYY-MM-DD HH:MM:SS}'
pub fn format_as_of_query(q: &AsOfQuery) -> String {
    // UNIX秒を UTC の YYYY-MM-DD HH:MM:SS に変換
    let secs = q.as_of_ts;
    let (y, mo, d, h, mi, s) = unix_secs_to_utc(secs);
    format!(
        "SELECT * FROM {} AS OF TIMESTAMP '{:04}-{:02}-{:02} {:02}:{:02}:{:02}'",
        q.table, y, mo, d, h, mi, s
    )
}

/// UNIX秒を (year, month, day, hour, min, sec) UTC に分解する簡易実装。
fn unix_secs_to_utc(secs: i64) -> (i32, u32, u32, u32, u32, u32) {
    // 1970-01-01 00:00:00 UTC からの経過秒
    let secs = secs.max(0) as u64;
    let s = (secs % 60) as u32;
    let total_min = secs / 60;
    let mi = (total_min % 60) as u32;
    let total_hours = total_min / 60;
    let h = (total_hours % 24) as u32;
    let total_days = total_hours / 24;
    // グレゴリオ暦計算（簡易版）
    let (y, mo, d) = days_to_ymd(total_days);
    (y, mo, d, h, mi, s)
}

fn days_to_ymd(mut days: u64) -> (i32, u32, u32) {
    let mut year = 1970_i32;
    loop {
        let days_in_year = if is_leap(year) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }
    let month_days: [u32; 12] = [31, if is_leap(year) { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut month = 1u32;
    for &md in &month_days {
        if days < md as u64 {
            break;
        }
        days -= md as u64;
        month += 1;
    }
    (year, month, days as u32 + 1)
}

fn is_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}
```

---

## ステップ 2: CHANGELOG.md にエントリ追加（テスト追加より先）

**ファイル**: `CHANGELOG.md`

```markdown
## [v75.2.0] — 2026-08-14 — `TemporalRange` / `AsOfQuery` 型

### Added
- `TemporalRange` 構造体（from_ts: i64, to_ts: i64）
- `AsOfQuery` 構造体（table: String, as_of_ts: i64）
- `is_in_range(ts: i64, range: &TemporalRange) -> bool` — 閉区間判定
- `format_as_of_query(q: &AsOfQuery) -> String` — Snowflake AS OF TIMESTAMP SQL 生成

### Tests
- `v752000_tests` 2 件追加（合計テスト数: 3696, +2）
  - `temporal_range_filters_correctly` — 閉区間フィルタの境界値判定を検証
  - `as_of_query_generates_sql` — Snowflake AS OF TIMESTAMP 形式の SQL 出力を検証
```

---

## ステップ 3: driver.rs にテストモジュールを追加

`use super::*` は `TemporalRange` / `AsOfQuery` / `is_in_range` / `format_as_of_query` を参照するために必須。

```rust
#[cfg(test)]
mod v752000_tests {
    use super::*;

    #[test]
    fn temporal_range_filters_correctly() {
        let range = TemporalRange { from_ts: 1000, to_ts: 2000 };
        // 範囲内
        assert!(is_in_range(1000, &range), "from_ts (lower bound) should be in range");
        assert!(is_in_range(1500, &range), "mid point should be in range");
        assert!(is_in_range(2000, &range), "to_ts (upper bound) should be in range");
        // 範囲外
        assert!(!is_in_range(999, &range), "below from_ts should be out of range");
        assert!(!is_in_range(2001, &range), "above to_ts should be out of range");
    }

    #[test]
    fn as_of_query_generates_sql() {
        // 2026-01-01 00:00:00 UTC = 1735689600
        let q = AsOfQuery {
            table: "orders".to_string(),
            as_of_ts: 1_735_689_600,
        };
        let sql = format_as_of_query(&q);
        assert!(sql.contains("SELECT * FROM orders"), "should have SELECT FROM orders");
        assert!(sql.contains("AS OF TIMESTAMP"), "should have AS OF TIMESTAMP");
        assert!(sql.contains("2026-01-01"), "should format date as 2026-01-01");
        assert!(sql.contains("00:00:00"), "should format time as 00:00:00");
    }
}
```

---

## ステップ 4: `fav/Cargo.toml` バージョン更新

`"75.1.0"` → `"75.2.0"`、および driver.rs 内のバージョン文字列アサーションを一括更新（replace_all）。

---

## ステップ 5: `versions/current.md` 更新

- 「進行中バージョン」を v75.2.0 に更新
- 「次に切る版」を v75.3.0 に更新

---

## ステップ 6: 動作確認

```bash
cd fav && cargo test v752000 -- --nocapture
# temporal_range_filters_correctly / as_of_query_generates_sql の 2 件 pass

cargo test -j 8 -- --test-threads=8 2>&1 | tail -5
# 3696 passed; 0 failed
```

---

## 実装順序まとめ

```
T0: 事前確認
1: driver.rs — 型定義・関数追加
2: CHANGELOG.md — エントリ追加（テストより先）
3: driver.rs — v752000_tests 追加
4: Cargo.toml — バージョン更新（replace_all で全アサーション更新）
5: versions/current.md — 更新
6: cargo test 全 pass 確認（3696 tests）
```
