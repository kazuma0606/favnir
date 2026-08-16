# Roadmap v75.1.0 〜 v76.0.0 — Temporal Data Native

Date: 2026-08-14
Status: 未着手（v75.0.0 完了後に開始）

マスターロードマップ: [roadmap-v75.1-v80.0.md](roadmap-v75.1-v80.0.md)

---

## 前提

- 直前完了: v75.0.0「Favnir 2.0 宣言」（tests = 3692）
- 本スプリントは Phase 6「Favnir 3.0 宣言」の第 1 スプリント
- 目標: v76.0.0「Temporal Data Native 宣言」（tests = 3714）

### スプリントの性格

データエンジニアにとって「いつ時点のデータか」は命題である。
鮮度チェック・SCD・タイムトラベル・保持ポリシーを Favnir のファーストクラス型として実現する。
A（新言語機能）60% + B（CLI 拡張）40% の構成。

---

## バージョン一覧

| バージョン | 内容 | テスト数 | 状態 |
|---|---|---|---|
| v75.1.0 | `FreshnessPolicy` 型基盤 | 3692 + 2 = 3694 | 未着手 |
| v75.2.0 | `TemporalRange` / `AsOfQuery` 型 | 3694 + 2 = 3696 | 未着手 |
| v75.3.0 | SCD Type 1 / Type 2 ネイティブ型 | 3696 + 2 = 3698 | 未着手 |
| v75.4.0 | Temporal join（時点結合） | 3698 + 2 = 3700 | 未着手 |
| v75.5.0 | `RetentionPolicy` 型 | 3700 + 2 = 3702 | 未着手 |
| v75.6.0 | Stream freshness monitoring | 3702 + 2 = 3704 | 未着手 |
| v75.7.0 | Temporal contracts | 3704 + 2 = 3706 | 未着手 |
| v75.8.0 | `fav time-travel` コマンド | 3706 + 2 = 3708 | 未着手 |
| v75.9.0 | 安定化・コードフリーズ | 3708 + 2 = 3710 | 未着手 |
| v76.0.0 | Temporal Data Native 宣言 ★クリーンアップ | 3710 + 4 = 3714 | 未着手 |

---

## v75.1.0 — `FreshnessPolicy` 型基盤

データの「鮮度」を型で表現する基盤。パイプラインが古いデータを最新として扱うバグをコンパイル時に検出する足がかり。

```favnir
fn get_price(id: String, ctx: AppCtx) -> Result<Float, String> {
    bind raw  <- ctx.io.read_file_raw("prices.csv")
    bind _    <- FreshnessPolicy.check(raw, max_age: Duration.minutes(5))
    Result.ok(parse_price(raw))
}
```

**実装内容:**
- `FreshnessStrategy` enum（Warn, Fail）
- `FreshnessPolicy` 構造体（max_age_secs: u64, strategy: FreshnessStrategy）
- `check_freshness(data_ts: i64, now: i64, policy: &FreshnessPolicy) -> bool`
- `format_freshness_warning(policy: &FreshnessPolicy, age_secs: u64) -> String`

**完了条件**: Rust テスト 2 件（3692 + 2 = 3694）
- `freshness_policy_enforced`
- `freshness_stale_detected`

---

## v75.2.0 — `TemporalRange` / `AsOfQuery` 型

時点クエリを型安全に表現。Snowflake / Delta Lake の `AS OF` 構文に対応。

```favnir
// タイムトラベルクエリ（Snowflake AS OF）
bind snapshot <- AsOfQuery { table: "orders", as_of_ts: run_date }
// → SELECT * FROM orders AS OF TIMESTAMP '2026-01-01 00:00:00'

// 期間フィルター
bind range    <- TemporalRange { from_ts: start, to_ts: end }
bind filtered <- orders |> filter_in_range(range)
```

**実装内容:**
- `TemporalRange` 構造体（from_ts: i64, to_ts: i64）
- `AsOfQuery` 構造体（table: String, as_of_ts: i64）
- `format_as_of_query(q: &AsOfQuery) -> String` — SQL 文字列生成
- `is_in_range(ts: i64, range: &TemporalRange) -> bool`

**完了条件**: Rust テスト 2 件（3694 + 2 = 3696）
- `temporal_range_filters_correctly`
- `as_of_query_generates_sql`

---

## v75.3.0 — SCD Type 1 / Type 2 ネイティブ型

Slowly Changing Dimensions（緩やかに変化するディメンション）を型として表現。
データウェアハウスの定番パターンをファーストクラスにする。

```favnir
// SCD Type 2: 変更履歴を保持（data フィールドに JSON シリアライズ済みデータを格納）
fn upsert_customer(existing: List<ScdRow>, new_data: String, now: Int) -> List<ScdRow> {
    apply_scd2_update(existing, new_data, now)
    // → 旧レコードを valid_to で閉じ、新レコードを is_current=true で追加
}
```

**実装内容:**
- `ScdType` enum（Type1, Type2）
- `ScdRow` 構造体（valid_from: i64, valid_to: Option<i64>, is_current: bool, data: String）
- `apply_scd2_update(existing: &[ScdRow], new_data: &str, new_ts: i64) -> Vec<ScdRow>`

**完了条件**: Rust テスト 2 件（3696 + 2 = 3698）
- `scd2_creates_history_row`
- `scd2_marks_previous_expired`

---

## v75.4.0 — Temporal join（時点結合）

Point-in-time join を型安全に表現。「注文時点の商品価格」のような時系列結合を正確に表現する。

```favnir
// 注文日時点の商品価格で結合
bind result <- orders |> temporal_join(
    prices,
    key: "product_id",
    as_of_field: "order_date"
)
// → 各注文の order_date 時点で有効な価格レコードと結合
```

**実装内容:**
- `TemporalJoinConfig` 構造体（left_key: String, right_key: String, as_of_field: String）
- `format_temporal_join_sql(config: &TemporalJoinConfig) -> String`
- `validate_temporal_join_config(config: &TemporalJoinConfig) -> Result<(), String>` — フィールド名検証

**完了条件**: Rust テスト 2 件（3698 + 2 = 3700）
- `temporal_join_sql_generated`
- `temporal_join_invalid_config_rejected`

---

## v75.5.0 — `RetentionPolicy` 型

データ保持ポリシーを型で宣言。GDPR の「保存期間」要件をコードで表現する。

```favnir
contract UserDataPipeline {
    input:     { rows: List<Row> }
    output:    { processed: List<Row> }
    retention: RetentionPolicy { max_age_days: 365, action: Anonymize }
}
```

**実装内容:**
- `RetentionAction` enum（Delete, Archive, Anonymize）
- `RetentionPolicy` 構造体（max_age_days: u64, action: RetentionAction）
- `RetentionResult` enum（Keep, Delete, Archive, Anonymize）
- `apply_retention_check(row_ts: i64, now: i64, policy: &RetentionPolicy) -> RetentionResult`

**完了条件**: Rust テスト 2 件（3700 + 2 = 3702）
- `retention_delete_old_rows`
- `retention_anonymize_action`

---

## v75.6.0 — Stream freshness monitoring

ストリームの遅延（lag）を監視する型基盤。Kafka / Kinesis パイプラインの鮮度を型で管理する。

```favnir
fn monitor_stream(ctx: AppCtx) -> Result<Unit, String> {
    bind lag <- StreamFreshnessMonitor.check(
        source: "kafka://orders-topic",
        max_lag: Duration.seconds(30)
    )
    bind _ <- ctx.io.println(f"lag={lag.lag_secs}s exceeded={lag.exceeded}")
    Result.ok(Unit)
}
```

**実装内容:**
- `StreamFreshnessMonitor` 構造体（source: String, max_lag_secs: u64）
- `StreamLagResult` 構造体（lag_secs: u64, exceeded: bool, source: String）
- `check_stream_lag(last_event_ts: i64, now: i64, monitor: &StreamFreshnessMonitor) -> StreamLagResult`
- `format_stream_lag_report(result: &StreamLagResult) -> String`

**完了条件**: Rust テスト 2 件（3702 + 2 = 3704）
- `stream_lag_within_threshold`
- `stream_lag_exceeded_detected`

---

## v75.7.0 — Temporal contracts

コントラクトに鮮度・保持ポリシーを組み込む。時間要件をコントラクト違反として型チェックする。

```favnir
contract PricingPipeline {
    input:     { product_id: String }
    output:    { price: Float where self >= 0.0 }
    sla:       { max_latency_ms: 1000 }
    freshness: FreshnessPolicy { max_age_secs: 300, strategy: Fail }
    retention: RetentionPolicy { max_age_days: 90, action: Delete }
}
```

**実装内容:**
- `TemporalContract` 構造体（name: String, freshness: Option<FreshnessPolicy>, retention: Option<RetentionPolicy>）
- `validate_temporal_contract(contract: &TemporalContract, data_ts: i64, now: i64) -> Result<(), String>`
- `format_temporal_contract_report(contract: &TemporalContract, result: &Result<(), String>) -> String`

**完了条件**: Rust テスト 2 件（3704 + 2 = 3706）
- `temporal_contract_freshness_violation`
- `temporal_contract_retention_exceeded`

---

## v75.8.0 — `fav time-travel` コマンド

CLI から時点クエリを手軽に発行できるコマンド。デバッグ・監査用途。

```bash
$ fav time-travel --table orders --at "2026-01-01T00:00:00Z"
SELECT * FROM orders AS OF TIMESTAMP '2026-01-01 00:00:00'

$ fav time-travel --table orders --at "2026-01-01T00:00:00Z" --format delta
SELECT * FROM orders VERSION AS OF 1735689600
```

**実装内容:**
- `TimeTravelFormat` enum（Snowflake, Delta, Generic）
- `TimeTravelQuery` 構造体（table: String, as_of_ts: i64, format: TimeTravelFormat）
- `cmd_time_travel(query: &TimeTravelQuery) -> String` — SQL 文字列生成
- `parse_time_travel_timestamp(s: &str) -> Result<i64, String>` — RFC3339 パース（秒単位 UNIX タイム）

**完了条件**: Rust テスト 2 件（3706 + 2 = 3708）
- `time_travel_snowflake_format`
- `time_travel_delta_format`

---

## v75.9.0 — 安定化・コードフリーズ（Temporal Data Native 前最終調整）

v75.1〜v75.8 の全機能を通しで確認する最終安定化スプリント。

**実装内容:**
- v75.1〜v75.8 の全テスト通過確認（`cargo test` 全 pass）
- `fav time-travel` / `FreshnessPolicy` / `RetentionPolicy` の E2E 動作確認
- バグ修正のみ受け入れ（新機能追加なし）

**完了条件**: Rust テスト 2 件（3708 + 2 = 3710）
- `temporal_full_sprint_all_stable`
- `temporal_e2e_pipeline_valid`

---

## v76.0.0 — Temporal Data Native 宣言 ★クリーンアップ

**宣言文**:
> 「鮮度が型となり、SCD が構造となり、タイムトラベルが API となった。
>  Favnir のパイプラインは今、時間軸を型で保証する。」

**クリーンアップ作業:**
- `cargo clean` 実施
- `Cargo.toml` バージョンを `76.0.0` に更新
- `CHANGELOG.md` に v76.0.0 エントリを追加
- `MILESTONE.md` に「Temporal Data Native」を追記
- `README.md` に v76.0 達成を追記
- `versions/current.md` を更新

**完了条件**: `v76000_tests` 4 件（3710 + 4 = 3714）
- `cargo_toml_version_is_76_0_0`
- `changelog_has_v76_0_0`
- `milestone_has_temporal_data_native`
- `readme_mentions_temporal`

---

## テスト数推移（本スプリント）

| バージョン | テスト数 | 増加 |
|---|---|---|
| v75.0.0（ベース） | 3,692 | — |
| v75.1.0 | 3,694 | +2 |
| v75.2.0 | 3,696 | +2 |
| v75.3.0 | 3,698 | +2 |
| v75.4.0 | 3,700 | +2 |
| v75.5.0 | 3,702 | +2 |
| v75.6.0 | 3,704 | +2 |
| v75.7.0 | 3,706 | +2 |
| v75.8.0 | 3,708 | +2 |
| v75.9.0 | 3,710 | +2 |
| v76.0.0（宣言） | 3,714 | +4 |

**本スプリント合計**: +22 tests（3,692 → 3,714）

---

## 参考リンク

- マスターロードマップ: [roadmap-v75.1-v80.0.md](roadmap-v75.1-v80.0.md)
- 前スプリント（完了）: [roadmap-v74.1-v75.0.md](roadmap-v74.1-v75.0.md)
- 次スプリント: [roadmap-v76.1-v77.0.md](roadmap-v76.1-v77.0.md)
- 達成宣言: `MILESTONE.md`
- 進行状況: `versions/current.md`
