# v75.9.0 実装計画 — 安定化・コードフリーズ

Date: 2026-08-15

---

## Step 1: CHANGELOG.md 更新（テスト追加より先）

`CHANGELOG.md` の先頭に v75.9.0 エントリを追加する。

---

## Step 2: テストモジュール v759000_tests 追加

`fav/src/driver.rs` の末尾に以下を追加する：

```rust
#[cfg(test)]
mod v759000_tests {
    use super::*;

    #[test]
    fn temporal_full_sprint_all_stable() {
        // v75.1.0 — FreshnessPolicy / check_freshness
        let fp = FreshnessPolicy { max_age_secs: 300, strategy: FreshnessStrategy::Fail };
        assert!(check_freshness(1000, 1200, &fp), "age=200 < 300: fresh");
        assert!(!check_freshness(1000, 1500, &fp), "age=500 > 300: stale");

        // v75.2.0 — TemporalRange / AsOfQuery
        let range = TemporalRange { from_ts: 0, to_ts: 1000 };
        assert!(is_in_range(500, &range));
        let asof_result = format_as_of_query(&AsOfQuery {
            table: "orders".to_string(), as_of_ts: 0,
        });
        assert!(asof_result.is_ok());
        assert!(asof_result.unwrap().contains("orders"));

        // v75.3.0 — ScdRow / apply_scd2_update
        let scd_result = apply_scd2_update(&[], "{}", 1);
        assert!(scd_result.is_ok());
        assert_eq!(scd_result.unwrap().len(), 1);

        // v75.4.0 — TemporalJoinConfig
        let join_cfg = TemporalJoinConfig {
            left_key:    "id".to_string(),
            right_key:   "id".to_string(),
            as_of_field: "ts".to_string(),
        };
        assert!(validate_temporal_join_config(&join_cfg).is_ok());
        let join_sql = format_temporal_join_sql("left_tbl", "right_tbl", &join_cfg);
        assert!(join_sql.contains("valid_from"));

        // v75.5.0 — RetentionPolicy / apply_retention_check
        let rp = RetentionPolicy { max_age_days: 30, action: RetentionAction::Delete };
        assert_eq!(apply_retention_check(1000, 1060, &rp), RetentionResult::Keep);

        // v75.6.0 — StreamFreshnessMonitor / check_stream_lag
        let monitor = StreamFreshnessMonitor {
            source: "kafka".to_string(), max_lag_secs: 120,
        };
        let lag = check_stream_lag(1000, 1060, &monitor);
        assert!(!lag.exceeded);

        // v75.7.0 — TemporalContract / validate_temporal_contract
        let contract = TemporalContract {
            name: "test".to_string(), freshness: None, retention: None,
        };
        assert!(validate_temporal_contract(&contract, 0, 0).is_ok());

        // v75.8.0 — cmd_time_travel / parse_time_travel_timestamp
        let ttq = TimeTravelQuery {
            table: "t".to_string(), as_of_ts: 0, format: TimeTravelFormat::Delta,
        };
        assert!(cmd_time_travel(&ttq).contains("VERSION AS OF 0"));
        assert_eq!(parse_time_travel_timestamp("2025-01-01T00:00:00Z"), Ok(1735689600));
    }

    #[test]
    fn temporal_e2e_pipeline_valid() {
        let data_ts: i64 = 1735689600; // 2025-01-01T00:00:00Z
        let now = data_ts + 60;        // 60 秒後

        // 1. 鮮度チェック
        let fp = FreshnessPolicy { max_age_secs: 300, strategy: FreshnessStrategy::Fail };
        assert!(check_freshness(data_ts, now, &fp), "data is fresh (60s < 300s)");

        // 2. タイムトラベルクエリ生成
        let ttq = TimeTravelQuery {
            table: "orders".to_string(), as_of_ts: data_ts, format: TimeTravelFormat::Snowflake,
        };
        let sql = cmd_time_travel(&ttq);
        assert!(sql.contains("AS OF TIMESTAMP"), "snowflake format");
        assert!(sql.contains("orders"), "table name present");

        // 3. 保持チェック
        let rp = RetentionPolicy { max_age_days: 365, action: RetentionAction::Delete };
        assert_eq!(apply_retention_check(data_ts, now, &rp), RetentionResult::Keep,
            "data is within retention window");

        // 4. ストリーム遅延確認
        let monitor = StreamFreshnessMonitor {
            source: "kafka".to_string(), max_lag_secs: 120,
        };
        let lag = check_stream_lag(data_ts, now, &monitor);
        assert!(!lag.exceeded, "lag=60 < 120: within limit");
        assert_eq!(lag.lag_secs, 60);

        // 5. コントラクト検証
        let contract = TemporalContract {
            name: "OrdersPipeline".to_string(),
            freshness: Some(FreshnessPolicy { max_age_secs: 300, strategy: FreshnessStrategy::Fail }),
            retention: Some(RetentionPolicy { max_age_days: 365, action: RetentionAction::Delete }),
        };
        assert!(validate_temporal_contract(&contract, data_ts, now).is_ok(),
            "full contract passes for fresh data");
    }
}
```

---

## Step 3: Cargo.toml バージョン更新

`fav/Cargo.toml`: `75.8.0` → `75.9.0`
`driver.rs` 内のバージョン文字列アサーションを一括更新（replace_all）。

---

## Step 4: versions/current.md 更新

- 進行中バージョン: v75.9.0
- 次に切る版: v76.0.0

---

## Step 5: 最終確認

`cargo test` が 3710 tests all pass であることを確認。
