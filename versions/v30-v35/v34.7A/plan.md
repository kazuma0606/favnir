# v34.7A — 実装プラン

## 方針

`examples/` 31 ファイルと `infra/e2e-demo/` 10 ファイルの `!Effect` → ctx 構文移行。
移行パターンは v34.6A（runes/ 移行）と同一。`cargo clean` は x.3.0 のため不要。

---

## 実装ステップ

### Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml` の version を `35.2.0` → `35.3.0` に変更。

---

### Step 2: examples/ 31 ファイルの !Effect → ctx 移行

#### 2.1 移行前の残存確認

```bash
grep -rl "-> .* !\w" /c/Users/yoshi/favnir/examples --include="*.fav" | wc -l
# → 31 件であること
```

#### 2.2 移行方法（v34.6A と同一パターン）

各ファイルで:
1. `fn f(args...) -> R !Eff` → `fn f(ctx: AppCtx, args...) -> R`
2. 関数本体の先頭に `bind { field } <- ctx` を追加
3. プリミティブ呼び出しを接続ベース API に変更

#### 2.3 グループ別移行（examples/）

**DB 系（ctx.db）**:
```
examples/postgres_etl.fav
examples/csv-to-postgres/src/stages.fav
examples/csv-to-postgres/src/main.fav
examples/sqlite_etl.fav
examples/bigquery_analytics.fav
examples/redshift_analytics.fav
examples/clickhouse_analytics.fav
examples/mongo_events_etl.fav
examples/mysql_orders_etl.fav
examples/dynamodb_session_store.fav
examples/real-world-etl/src/stages.fav
examples/real-world-etl/src/main.fav
```

**ストリーム系（ctx.stream）**:
```
examples/kafka_events_etl.fav
examples/elasticsearch_logs_etl.fav
examples/streaming/kafka_to_elasticsearch.fav
examples/streaming/kinesis_to_s3.fav
examples/streaming/nats_to_postgres.fav
```

**IO / ファイル系（ctx.io）**:
```
examples/full_etl.fav
examples/s3_csv_to_parquet.fav
examples/jsonl_etl.fav
examples/iceberg_etl.fav
examples/delta_lake_etl.fav
examples/dbt_pipeline.fav
```

**複合系（複数フィールド）**:
```
examples/real-world-etl/src/notifications.fav  (http, io)
examples/redis_rate_limiter.fav                (redis, io)
examples/observability/datadog_apm.fav         (tracer, http)
examples/observability/grafana_dashboard.fav   (tracer, http)
examples/observability/otel_tracing.fav        (tracer)
examples/observability/prometheus_demo.fav     (tracer)
examples/observability/prometheus_grafana.fav  (tracer, http)
examples/observability/sentry_alerting.fav     (tracer, http)
```

---

### Step 3: infra/e2e-demo/ 10 ファイルの !Effect → ctx 移行

#### 3.1 移行前の残存確認

```bash
grep -rl "-> .* !\w" /c/Users/yoshi/favnir/infra/e2e-demo --include="*.fav" | wc -l
# → 10 件であること
```

#### 3.2 各ファイルの移行

```
infra/e2e-demo/bigquery/src/demo.fav           (db)
infra/e2e-demo/crosscloud/lambda/verifier/verifier.fav    (http, io)
infra/e2e-demo/crosscloud/lambda/verifier_v2/verifier_v2.fav  (http, io)
infra/e2e-demo/crosscloud/src/migrate.fav      (db, io)
infra/e2e-demo/ecs/src/etl.fav                 (db, io)
infra/e2e-demo/ecs/src/pipeline.fav            (db, io)
infra/e2e-demo/eks/src/pipeline.fav            (db, stream)
infra/e2e-demo/kafka/src/pipeline.fav          (stream)
infra/e2e-demo/lambda/src/pipeline.fav         (db, http)
infra/e2e-demo/snowflake/src/demo.fav          (warehouse)
```

---

### Step 4: 移行後の確認

```bash
# examples/ に !Effect が残っていないこと
grep -rl "-> .* !\w" /c/Users/yoshi/favnir/examples --include="*.fav"
# → 0 件になること

# infra/e2e-demo/ に !Effect が残っていないこと
grep -rl "-> .* !\w" /c/Users/yoshi/favnir/infra/e2e-demo --include="*.fav"
# → 0 件になること
```

---

### Step 5: driver.rs 更新

#### 5.1 cargo_toml_version_is_35_2_0 をスタブ化

```bash
grep -n "cargo_toml_version_is_35_2_0" fav/src/driver.rs
```

#### 5.2 v35300_tests の挿入

`v35200_tests` の終端 `}` の直後に v35300_tests を挿入（spec.md の通り）。

---

### Step 6: ビルド + テスト実行

```bash
cd /c/Users/yoshi/favnir/fav
cargo build 2>&1 | tail -5
cargo test --bin fav v35300 2>&1 | tail -8
cargo test 2>&1 | grep "test result"
cargo clippy --locked -- -D warnings 2>&1 | tail -5
```

---

### Step 7: CHANGELOG.md 更新

```markdown
## [v35.3.0] — 2026-07-04

### Changed
- `examples/` 配下 31 ファイル — `!Effect` アノテーションを廃止し Capability Context に移行
- `infra/e2e-demo/` 配下 10 ファイル — 同上

### Notes
- v34.7A 補完実装: ロードマップが要求した Examples / E2E デモの ctx 移行を実施
- v34.5A + v34.6A + v34.7A の完了により !Effect 廃止作業が全完了
```

---

### Step 8: benchmarks/v35.3.0.json 作成

```json
{
  "version": "35.3.0",
  "milestone": "Production Ready",
  "date": "2026-07-04",
  "tests_passed": 2601,
  "tests_failed": 0,
  "notes": "v34.7A補完: examples/ 31ファイル + infra/e2e-demo/ 10ファイル !Effect → ctx 移行完了。!Effect廃止作業全完了。v35300_tests 5 件追加。"
}
```

---

### Step 9: versions/current.md 更新

- `最新安定版` → `**v35.3.0** — Examples / E2E デモ ctx 移行（v34.7A）`
- `次に切る版` → `未定`
- `現行マスターロードマップ`: 既存のまま（次ロードマップは未定）

---

## テスト実行

```bash
cd /c/Users/yoshi/favnir/fav
cargo test --bin fav v35300 2>&1 | tail -8
cargo test 2>&1 | grep "test result"
```

---

## 完了処理

- `benchmarks/v35.3.0.json` の `tests_passed` を実測値で確定
- `tasks.md` を COMPLETE に更新
