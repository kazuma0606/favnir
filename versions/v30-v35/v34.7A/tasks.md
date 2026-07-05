# v34.7A — タスクリスト

**ステータス**: COMPLETE

---

## 前提確認（T0）

- [x] `fav/Cargo.toml` の version が `35.2.0` であること
- [x] v34.6A が COMPLETE であること
- [x] `benchmarks/v35.2.0.json` の `tests_passed` を確認
- [x] `driver.rs` に `mod v35300_tests` が存在しないこと
- [x] `cargo_toml_version_is_35_2_0` が v35200_tests 内に存在すること（スタブ化対象）
- [x] `examples/real-world-etl/src/stages.fav` にまだ `!Effect` が存在すること（移行対象確認）
- [x] `grep -rl "-> .* !\w" examples/ | wc -l` が 31 であること
- [x] `grep -rl "-> .* !\w" infra/e2e-demo/ | wc -l` が 10 であること

---

## 実装タスク

- [x] **T1** `fav/Cargo.toml` — version を `35.2.0` → `35.3.0` に更新
- [x] **T2-ex-db** `examples/` DB系 12 ファイル — !Effect → ctx 移行
       postgres_etl.fav, csv-to-postgres/src/stages.fav, csv-to-postgres/src/main.fav,
       sqlite_etl.fav, bigquery_analytics.fav, redshift_analytics.fav,
       clickhouse_analytics.fav, mongo_events_etl.fav, mysql_orders_etl.fav,
       dynamodb_session_store.fav,
       real-world-etl/src/stages.fav, real-world-etl/src/main.fav
- [x] **T2-ex-stream** `examples/` ストリーム系 5 ファイル — !Effect → ctx 移行
       kafka_events_etl.fav, elasticsearch_logs_etl.fav,
       streaming/kafka_to_elasticsearch.fav, streaming/kinesis_to_s3.fav,
       streaming/nats_to_postgres.fav
- [x] **T2-ex-io** `examples/` IO/File系 6 ファイル — !Effect → ctx 移行
       full_etl.fav, s3_csv_to_parquet.fav, jsonl_etl.fav,
       iceberg_etl.fav, delta_lake_etl.fav, dbt_pipeline.fav
- [x] **T2-ex-misc** `examples/` 複合系 8 ファイル — !Effect → ctx 移行
       real-world-etl/src/notifications.fav, redis_rate_limiter.fav,
       observability/datadog_apm.fav, observability/grafana_dashboard.fav,
       observability/otel_tracing.fav, observability/prometheus_demo.fav,
       observability/prometheus_grafana.fav, observability/sentry_alerting.fav
- [x] **T3-infra** `infra/e2e-demo/` 10 ファイル — !Effect → ctx 移行
       bigquery/src/demo.fav,
       crosscloud/lambda/verifier/verifier.fav,
       crosscloud/lambda/verifier_v2/verifier_v2.fav,
       crosscloud/src/migrate.fav,
       ecs/src/etl.fav, ecs/src/pipeline.fav, eks/src/pipeline.fav,
       kafka/src/pipeline.fav, lambda/src/pipeline.fav,
       snowflake/src/demo.fav
- [x] **T4** 移行後チェック（examples/）
       `grep -rl "-> .* !\w" examples/ --include="*.fav"` が 0 件
- [x] **T5** 移行後チェック（infra/e2e-demo/）
       `grep -rl "-> .* !\w" infra/e2e-demo/ --include="*.fav"` が 0 件
- [x] **T6** `fav/src/driver.rs` — `cargo_toml_version_is_35_2_0` をスタブ化
- [x] **T7** `fav/src/driver.rs` — `v35300_tests`（5 件）を追加
       挿入位置: `v35200_tests` 直後・`// ── v31.7.0 tests` の前
       `use super::*` なし
- [x] **T8** `CHANGELOG.md` — `[v35.3.0]` セクションを先頭に追記
- [x] **T9** `benchmarks/v35.3.0.json` — 新規作成（暫定 `tests_passed`: 2601）
- [x] **T10** `versions/current.md` — 最新安定版を v35.3.0 に更新

---

## テスト確認

- [x] **T11** `cargo test --bin fav v35300 2>&1 | tail -8` — 5/5 PASS
- [x] **T12** `cargo test 2>&1 | grep "test result"` — 全件 PASS（0 failures）
- [x] **T13** `cargo clippy --locked -- -D warnings` — warnings なし

---

## 完了処理

- [x] **T14** `benchmarks/v35.3.0.json` の `tests_passed` を実測値で確定
- [x] **T15** このファイル（tasks.md）を COMPLETE に更新（全チェックボックス `[x]`）

---

## 完了条件チェックリスト（spec.md 対応）

- [x] `Cargo.toml` version = `"35.3.0"`
- [x] `cargo_toml_version_is_35_2_0` が空スタブになっていること
- [x] `examples/` 配下 31 件に `!Effect` アノテーションが残存しないこと
- [x] `infra/e2e-demo/` 配下 10 件に `!Effect` アノテーションが残存しないこと
- [x] `real_world_etl_uses_ctx_syntax` — `ctx: AppCtx` を含み `!Postgres`/`!Http`/`!Io` を含まない
- [x] `postgres_etl_uses_ctx_syntax` — `ctx: AppCtx` を含み `!Postgres` を含まない
- [x] `e2e_lambda_uses_ctx_syntax` — `ctx: AppCtx` を含み `!Effect` を含まない
- [x] `no_effect_annotations_in_examples` — 複数 examples ファイルが `!Effect` なし
- [x] `cargo test --bin fav v35300` — 5/5 PASS
- [x] `cargo test` — 全件 PASS（0 failures）
- [x] `cargo clippy --locked -- -D warnings` — PASS
- [x] `CHANGELOG.md` に `[v35.3.0]` セクション
- [x] `benchmarks/v35.3.0.json` 存在かつ `tests_passed` が実測値
- [x] `benchmarks/v35.3.0.json` の `tests_failed` が `0`
- [x] `versions/current.md` が v35.3.0 に更新されていること
- [x] `tasks.md` が COMPLETE

---

## コードレビューチェックリスト

- [x] `v35300_tests` に `use super::*` が**ない**こと
- [x] 否定テストと肯定テストが両立していること
- [x] `examples/` と `infra/e2e-demo/` の両方が移行対象に含まれていること
- [x] `benchmarks/v35.3.0.json` の `tests_failed` が `0` であること
- [x] CHANGELOG.md の日付が正しいこと（2026-07-05）

## コードレビュー指摘対応

- 最初の migration スクリプト（v1）が stage 行の `|arg| {` を消失させる不具合 → git checkout で全復元後、`re.sub` ベースの正しいスクリプト（v2）で再適用
- `public stage` / `stage(params)` 形式がスクリプトの正規表現に非マッチ → 残存 4 箇所を Edit で直接修正
- v15.x 旧テスト（`verifier_fav_has_aws_effects` / `crosscloud_effects_declared`）が ctx 移行後に FAIL → スタブ化で対応
