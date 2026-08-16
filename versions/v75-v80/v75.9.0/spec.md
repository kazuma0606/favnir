# v75.9.0 仕様書 — 安定化・コードフリーズ

Date: 2026-08-15
Status: 計画中

---

## Background

v75.1.0〜v75.8.0 で実装した Temporal Data Native 基盤（FreshnessPolicy、TemporalRange/AsOfQuery、TemporalJoinConfig、RetentionPolicy、StreamFreshnessMonitor、TemporalContract、TimeTravelQuery/parse_time_travel_timestamp）を通しで確認する安定化バージョン。新しい型や関数は追加せず、既存 API が連携して動作することを 2 件の統合テストで保証する。

---

## Goals

1. `temporal_full_sprint_all_stable` — v75.1〜v75.8 の全 Temporal 型を 1 テストで網羅する（各型のインスタンスを作成し基本 API を通す）
2. `temporal_e2e_pipeline_valid` — データ到着 → 鮮度チェック → タイムトラベルクエリ → 保持チェック → ストリーム遅延確認 の E2E シナリオを 1 テストで検証する
3. Rust テスト 2 件追加で 3710 tests に到達する

---

## テスト仕様

### `temporal_full_sprint_all_stable`

v75.1〜v75.8 の各型について基本 API を呼び出し、すべて期待通りの結果を返すことを確認する。

| バージョン | 型 / 関数 | アサーション |
|---|---|---|
| v75.1.0 | `check_freshness(data_ts=1000, now=1200, policy{max_age_secs=300})` | `== true`（新鮮） |
| v75.1.0 | `check_freshness(data_ts=1000, now=1500, policy{max_age_secs=300, strategy=Fail})` | `== false`（陳腐） |
| v75.2.0 | `is_in_range(500, &TemporalRange { from_ts: 0, to_ts: 1000 })` | `== true` |
| v75.2.0 | `format_as_of_query(&AsOfQuery { table: "orders", as_of_ts: 0 })` | `is_ok() == true` かつ `"orders"` を含む |
| v75.3.0 | `apply_scd2_update(&[], "data", 1)` | `is_ok() == true` かつ `len() == 1` |
| v75.4.0 | `validate_temporal_join_config(&TemporalJoinConfig { left_key: "id", right_key: "id", as_of_field: "ts" })` | `Ok(())` |
| v75.4.0 | `format_temporal_join_sql("left", "right", &config)` | `"valid_from"` を含む |
| v75.5.0 | `apply_retention_check(row_ts=1000, now=1060, policy{max_age_days=30})` | `RetentionResult::Keep` |
| v75.6.0 | `check_stream_lag(last_event_ts=1000, now=1060, monitor{max_lag_secs=120}).exceeded` | `== false` |
| v75.7.0 | `validate_temporal_contract(&TemporalContract{..., freshness: None, retention: None}, 0, 0)` | `Ok(())` |
| v75.8.0 | `cmd_time_travel(&TimeTravelQuery { table: "t", as_of_ts: 0, format: Delta })` | `"VERSION AS OF 0"` を含む |
| v75.8.0 | `parse_time_travel_timestamp("2025-01-01T00:00:00Z")` | `Ok(1735689600)` |

---

### `temporal_e2e_pipeline_valid`

E2E シナリオ: データ (data_ts = 1735689600、now = data_ts + 60) を処理するパイプライン全体の確認。

1. **鮮度チェック**: `FreshnessPolicy { max_age_secs: 300, strategy: Fail }` で `check_freshness(data_ts, now)` → `true`
2. **タイムトラベルクエリ生成**: `TimeTravelQuery { table: "orders", as_of_ts: data_ts, format: Snowflake }` → `cmd_time_travel` が `"AS OF TIMESTAMP"` と `"orders"` を含む
3. **保持チェック**: `RetentionPolicy { max_age_days: 365, action: Delete }` で `apply_retention_check(data_ts, now)` → `RetentionResult::Keep`
4. **ストリーム遅延確認**: `StreamFreshnessMonitor { source: "kafka", max_lag_secs: 120 }` で `check_stream_lag(data_ts, now)` → `exceeded == false`、`lag_secs == 60`
5. **コントラクト検証**: `TemporalContract { freshness: Some(FreshnessPolicy { max_age_secs: 300, strategy: Fail }), retention: Some(RetentionPolicy { max_age_days: 365, action: Delete }) }` で `validate_temporal_contract(data_ts, now)` → `Ok(())`

---

## Success Criteria

- `temporal_full_sprint_all_stable` が pass
- `temporal_e2e_pipeline_valid` が pass
- `cargo test` が 3710 tests all pass
- `CHANGELOG.md` の先頭に v75.9.0 エントリが存在する

---

## 変更ファイル

- `fav/src/driver.rs` — `v759000_tests` モジュールを追加
- `CHANGELOG.md` — v75.9.0 エントリを追加
- `versions/current.md` — 進行中バージョンを更新
- `fav/Cargo.toml` — バージョンを `75.8.0` → `75.9.0` に更新
- `fav/Cargo.lock` — バージョン更新に伴い自動更新

---

## 依存（既実装）

- v75.1.0: `FreshnessPolicy`, `check_freshness`, `format_freshness_warning`
- v75.2.0: `TemporalRange`, `AsOfQuery`, `format_as_of_query`, `unix_secs_to_utc`, `is_leap`
- v75.3.0: `Scd2Row`, `validate_scd2_row`
- v75.4.0: `TemporalJoinConfig`, `validate_temporal_join_config`, `format_temporal_join_sql`
- v75.5.0: `RetentionPolicy`, `RetentionAction`, `RetentionResult`, `apply_retention_check`
- v75.6.0: `StreamFreshnessMonitor`, `StreamLagResult`, `check_stream_lag`, `format_stream_lag_report`
- v75.7.0: `TemporalContract`, `validate_temporal_contract`, `format_temporal_contract_report`
- v75.8.0: `TimeTravelFormat`, `TimeTravelQuery`, `cmd_time_travel`, `parse_time_travel_timestamp`

---

## 対象外

- 新しい型・関数の追加（安定化のみ）
- site/ MDX 追加（v76.0.0 宣言バージョンで行う）
