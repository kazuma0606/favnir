# v34.7A — Spec

## 概要

**テーマ**: Examples / E2E デモの `!Effect` → Capability Context 移行

**バージョン番号**: Cargo.toml `35.3.0`（プロジェクト追跡名 v34.7A）

**背景**: v34.7.0 ではインクリメンタルコンパイルキャッシュ統計のみ実装し、
ロードマップが要求する「examples/ と infra/e2e-demo/ の `!Effect` → ctx 移行」が未実装だった。
本バージョンはその補完実装（差分）である。

---

## 実装されていなかった要件（ロードマップ原文）

`versions/roadmap/roadmap-v33.1-v34.0.md` の「移行対象（破壊的変更）」より:

> 4. **E2E デモ・examples**
>    - `infra/e2e-demo/` 配下の `.fav` ファイル
>    - `examples/` 配下の `.fav` ファイル
>    - `fav/tests/fixtures/` 配下のテストフィクスチャ

実測:
- `examples/` 配下: **31 ファイル**
- `infra/e2e-demo/` 配下: **10 ファイル**

合計 **41 ファイル**。

> **スコープ外**: `fav/tests/fixtures/` 配下のテストフィクスチャは、
> 既存の Rust テストが直接 `include_str!` で読み込んでおり、
> `!Effect` 構文のままでも cargo test が通ることを利用した設計になっている。
> 移行すると既存テストの `assert!(src.contains("!Io"))` 等が壊れるため、
> 本バージョンではスコープ外とし将来版（v35.x 以降）の課題とする。

---

## 移行パターン（v34.6A と同一）

```favnir
// Before
fn load_orders(ctx: LoadCtx) -> Result<List<Order>, String> !Postgres !Http {
    ...
}

// After
fn load_orders(ctx: AppCtx) -> Result<List<Order>, String> {
    bind { db, http } <- ctx
    ...
}
```

`!Effect` → ctx フィールド対応表は v34.6A spec.md を参照。

---

## 移行対象ファイル一覧

### examples/ （31 ファイル）

```
examples/bigquery_analytics.fav
examples/clickhouse_analytics.fav
examples/csv-to-postgres/src/main.fav
examples/csv-to-postgres/src/stages.fav
examples/delta_lake_etl.fav
examples/dbt_pipeline.fav
examples/dynamodb_session_store.fav
examples/elasticsearch_logs_etl.fav
examples/full_etl.fav
examples/iceberg_etl.fav
examples/jsonl_etl.fav
examples/kafka_events_etl.fav
examples/mongo_events_etl.fav
examples/mysql_orders_etl.fav
examples/observability/datadog_apm.fav
examples/observability/grafana_dashboard.fav
examples/observability/otel_tracing.fav
examples/observability/prometheus_demo.fav
examples/observability/prometheus_grafana.fav
examples/observability/sentry_alerting.fav
examples/postgres_etl.fav
examples/real-world-etl/src/main.fav
examples/real-world-etl/src/notifications.fav
examples/real-world-etl/src/stages.fav
examples/redis_rate_limiter.fav
examples/redshift_analytics.fav
examples/s3_csv_to_parquet.fav
examples/sqlite_etl.fav
examples/streaming/kafka_to_elasticsearch.fav
examples/streaming/kinesis_to_s3.fav
examples/streaming/nats_to_postgres.fav
```

### infra/e2e-demo/ （10 ファイル）

```
infra/e2e-demo/bigquery/src/demo.fav
infra/e2e-demo/crosscloud/lambda/verifier/verifier.fav
infra/e2e-demo/crosscloud/lambda/verifier_v2/verifier_v2.fav
infra/e2e-demo/crosscloud/src/migrate.fav
infra/e2e-demo/ecs/src/etl.fav
infra/e2e-demo/ecs/src/pipeline.fav
infra/e2e-demo/eks/src/pipeline.fav
infra/e2e-demo/kafka/src/pipeline.fav
infra/e2e-demo/lambda/src/pipeline.fav
infra/e2e-demo/snowflake/src/demo.fav
```

---

## 実装スコープ

### 変更ファイル

1. `fav/Cargo.toml` — version `35.2.0` → `35.3.0`
2. `examples/**/*.fav` — 31 ファイルの `!Effect` → ctx 移行
3. `infra/e2e-demo/**/*.fav` — 10 ファイルの `!Effect` → ctx 移行
4. `fav/src/driver.rs` — `cargo_toml_version_is_35_2_0` をスタブ化、`v35300_tests` 5 件追加
5. `benchmarks/v35.3.0.json` — 新規作成
6. `CHANGELOG.md` — `[v35.3.0]` セクション先頭追記
7. `versions/current.md` — 最新安定版を v35.3.0 に更新

---

## テスト仕様（v35300_tests）

```rust
// ── v35.3.0 tests (v34.7A supplement: examples/ + infra/ !Effect → ctx migration) ──
#[cfg(test)]
mod v35300_tests {
    #[test]
    fn cargo_toml_version_is_35_3_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("35.3.0"), "Cargo.toml must contain '35.3.0'");
    }

    #[test]
    fn real_world_etl_uses_ctx_syntax() {
        let src = include_str!("../../examples/real-world-etl/src/stages.fav");
        assert!(
            src.contains("ctx: AppCtx"),
            "examples/real-world-etl/src/stages.fav must use ctx: AppCtx parameter"
        );
        assert!(
            !src.contains("!Postgres") && !src.contains("!Http") && !src.contains("!Io"),
            "examples/real-world-etl/src/stages.fav must not contain !Effect annotations"
        );
    }

    #[test]
    fn postgres_etl_uses_ctx_syntax() {
        let src = include_str!("../../examples/postgres_etl.fav");
        assert!(
            src.contains("ctx: AppCtx"),
            "examples/postgres_etl.fav must use ctx: AppCtx parameter"
        );
        assert!(
            !src.contains("!Postgres"),
            "examples/postgres_etl.fav must not contain !Postgres annotation"
        );
    }

    #[test]
    fn e2e_lambda_uses_ctx_syntax() {
        let src = include_str!("../../infra/e2e-demo/lambda/src/pipeline.fav");
        assert!(
            src.contains("ctx: AppCtx"),
            "infra/e2e-demo/lambda/src/pipeline.fav must use ctx: AppCtx parameter"
        );
        assert!(
            !src.contains("!Http") && !src.contains("!Io") && !src.contains("!Postgres"),
            "infra/e2e-demo/lambda/src/pipeline.fav must not contain !Effect annotations"
        );
    }

    #[test]
    fn no_effect_annotations_in_examples() {
        // 代表的な examples を確認（各グループから 1 件）
        let files = [
            include_str!("../../examples/full_etl.fav"),
            include_str!("../../examples/kafka_events_etl.fav"),
            include_str!("../../examples/redis_rate_limiter.fav"),
        ];
        for src in &files {
            assert!(
                !src.contains("!Postgres") && !src.contains("!Http")
                    && !src.contains("!Redis") && !src.contains("!Stream")
                    && !src.contains("!Io") && !src.contains("!Llm")
                    && !src.contains("!Trace") && !src.contains("!Snowflake"),
                "example files must not contain !Effect annotations after v34.7A migration"
            );
        }
    }
}
```

### 設計注記

- `v35300_tests` は `v35200_tests` 直後・`// ── v31.7.0 tests` の前に挿入
- `use super::*` なし
- 否定チェック（`!src.contains("!Effect")`）と肯定チェック（`src.contains("ctx: AppCtx")`）の両方を実施

---

## 完了条件

- [ ] `Cargo.toml` version = `"35.3.0"`
- [ ] `cargo_toml_version_is_35_2_0` が空スタブになっていること
- [ ] `examples/` 配下 31 ファイルに `!Effect` アノテーションが残存しないこと
- [ ] `infra/e2e-demo/` 配下 10 ファイルに `!Effect` アノテーションが残存しないこと
- [ ] `cargo test --bin fav v35300` — 5/5 PASS
- [ ] `cargo test` — 全件 PASS（2601 件想定 = 2596 + 5、0 failures）
- [ ] `cargo clippy --locked -- -D warnings` — PASS
- [ ] `CHANGELOG.md` に `[v35.3.0]` セクション
- [ ] `benchmarks/v35.3.0.json` 存在かつ `tests_passed` が実測値
- [ ] `benchmarks/v35.3.0.json` の `tests_failed` が `0`
- [ ] `versions/current.md` が v35.3.0 に更新されていること
- [ ] `tasks.md` が COMPLETE

---

## 補足：v34.7A 完了で !Effect 移行が完結

v34.5A + v34.6A + v34.7A の完了により、ロードマップが要求した `!Effect` 廃止作業のうち
以下がすべて実装済みとなる:

| 項目 | 実装バージョン |
|---|---|
| W022 lint ルール（fav lint 時に警告） | v34.5.0 |
| ast.rs `is_deprecated()` | v34.5A (v35.1.0) |
| checker.rs 型チェック時の deprecation 警告 | v34.5A (v35.1.0) |
| runes/ 100 ファイル ctx 移行 | v34.6A (v35.2.0) |
| examples/ 31 ファイル ctx 移行 | v34.7A (v35.3.0) |
| infra/e2e-demo/ 10 ファイル ctx 移行 | v34.7A (v35.3.0) |

残課題（将来版）: `!Effect` パース継続廃止（現在はパースしたうえで deprecation 警告）
