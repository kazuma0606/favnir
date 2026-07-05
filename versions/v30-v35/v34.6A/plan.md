# v34.6A — 実装プラン

## 方針

`runes/` 配下 100 ファイルの `!Effect` → ctx 構文移行。
各ファイルの全 public fn に `ctx: AppCtx` 第一引数を追加し、
本体で `bind { field } <- ctx` を使って capability にアクセスする。

`cargo clean` は x.2.0 のため不要。

---

## 移行パターン早見表

```
fn f(arg1: T1) -> R !Effect         →  fn f(ctx: AppCtx, arg1: T1) -> R
fn f(arg1: T1, arg2: T2) -> R !E    →  fn f(ctx: AppCtx, arg1: T1, arg2: T2) -> R
public fn f(a: A) -> R !Postgres    →  public fn f(ctx: AppCtx, a: A) -> R
複数 !Effect: !Http !Io             →  bind { http, io } <- ctx
```

---

## 実装ステップ

### Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml` の version を `35.1.0` → `35.2.0` に変更。

---

### Step 2: runes/ 100 ファイルの !Effect → ctx 移行

#### 2.1 移行前の残存確認

```bash
grep -rl "-> .* !\w" /c/Users/yoshi/favnir/runes --include="*.fav" | grep -v "runes/ctx/" | wc -l
```

100 件が表示されることを確認。

#### 2.2 移行方法

各ファイルを以下の規則で修正する:

**シグネチャ変更**:
```
fn f(args...) -> R !Eff1 !Eff2   →   fn f(ctx: AppCtx, args...) -> R
```

**本体の先頭に bind 追加**（使用するフィールドのみ）:
```
!Postgres       →   bind { db } <- ctx
!Redis          →   bind { redis } <- ctx
!Http           →   bind { http } <- ctx
!Io / !File     →   bind { io } <- ctx
!Stream         →   bind { stream } <- ctx
!Llm            →   bind { llm } <- ctx
!Snowflake      →   bind { warehouse } <- ctx
!Trace          →   bind { tracer } <- ctx
```

**プリミティブ呼び出しの変更**:
```
Postgres.execute_raw(sql, params)           →   Postgres.execute_with_conn_raw(db, sql, params)
Postgres.query_raw(sql, params)             →   Postgres.query_with_conn_raw(db, sql, params)
Redis.get_raw(key)                          →   Redis.get_with_conn_raw(redis, key)
Redis.set_raw(key, val, ttl)               →   Redis.set_with_conn_raw(redis, key, val, ttl)
HTTP.get(url)                              →   http.get(url)
IO.println(msg)                            →   io.println(msg)
```

#### 2.3 グループ別移行

**グループ A: データベース系**（db フィールド）
```
runes/postgres/client.fav
runes/postgres/postgres.fav
runes/postgres/postgres_db.fav
runes/mysql/mysql.fav
runes/mongodb/mongodb.fav
runes/dynamodb/dynamodb.fav
runes/aws/dynamodb.fav
runes/elasticsearch/elasticsearch.fav
runes/sqlite/sqlite.fav
runes/clickhouse/clickhouse.fav
runes/bigquery/bigquery.fav
runes/redshift/redshift.fav
runes/azure-postgres/client.fav
runes/azure-postgres/azure_postgres.fav
runes/snowflake/client.fav
runes/snowflake/snowflake.fav
runes/snowflake/snowflake_db.fav
runes/db/connection.fav
runes/db/query.fav
runes/db/migration.fav
runes/db/transaction.fav
runes/db/db.test.fav
runes/duckdb/query.fav
runes/duckdb/io.fav
runes/duckdb/duckdb.test.fav
```

**グループ B: Redis**（redis フィールド）
```
runes/redis/redis.fav
runes/cache/cache.fav
```

**グループ C: HTTP / ネットワーク系**（http フィールド）
```
runes/http/client.fav
runes/http/request.fav
runes/http/retry.fav
runes/http/http_client_impl.fav
runes/graphql/client.fav
runes/graphql/graphql.fav
runes/grpc/client.fav
runes/grpc/server.fav
runes/aws/s3.fav
runes/aws/s3_storage.fav
runes/aws/sqs.fav
runes/aws/secrets.fav
runes/azure-blob/azure_blob.fav
runes/airtable/airtable.fav
runes/hubspot/hubspot.fav
runes/intercom/intercom.fav
runes/linear/linear.fav
runes/notion/notion.fav
runes/pagerduty/pagerduty.fav
runes/sendgrid/sendgrid.fav
runes/shopify/shopify.fav
runes/slack/slack.fav
runes/stripe/stripe.fav
runes/twilio/twilio.fav
runes/zendesk/zendesk.fav
runes/github/github.fav
runes/email/email.fav
```

**グループ D: ストリーム系**（stream フィールド）
```
runes/kafka/kafka.fav
runes/kinesis/kinesis.fav
runes/nats/nats.fav
runes/pulsar/pulsar.fav
runes/rabbitmq/rabbitmq.fav
runes/sqs/sqs.fav
runes/incremental/pipeline.fav
runes/incremental/checkpoint.fav
runes/state/state.fav
```

**グループ E: IO / ファイル系**（io フィールド）
```
runes/fs/fs.fav
runes/io/io_impl.fav
runes/csv/csv.fav
runes/jsonl/jsonl.fav
runes/toml/toml.fav
runes/dbt/dbt.fav
runes/iceberg/iceberg.fav
runes/delta-lake/delta-lake.fav
runes/rune_loader/loader.fav
runes/env/access.fav
runes/env/dotenv.fav
runes/env/env_impl.fav
runes/env/typed.fav
```

**グループ F: LLM / AI 系**（llm フィールド）
```
runes/llm/client.fav
runes/llm/llm.fav
runes/vertex-ai/vertex-ai.fav
runes/sagemaker/sagemaker.fav
runes/pinecone/pinecone.fav
runes/mlflow/mlflow.fav
```

**グループ G: 監視・ログ系**（tracer フィールド）
```
runes/datadog/datadog.fav
runes/grafana/grafana.fav
runes/otel/otel.fav
runes/prometheus/prometheus.fav
runes/sentry/sentry.fav
runes/log/emitter.fav
runes/log/metric.fav
```

**グループ H: 汎用 / その他**
```
runes/auth/apikey.fav       (http)
runes/auth/jwt.fav          (io)
runes/auth/oauth2.fav       (http)
runes/gen/hint.fav          (io)
runes/gen/output.fav        (io)
runes/gen/primitives.fav    (io)
runes/gen/structured.fav    (llm)
runes/queue/queue.fav       (stream)
runes/sql/query.fav         (db)
runes/stat/stat.fav         (io)
runes/stat/stat.test.fav    (io)
```

---

### Step 3: 移行後の確認

```bash
# !Effect が runes/ に残っていないこと（ctx/ を除く）
grep -rl "-> .* !\w" /c/Users/yoshi/favnir/runes --include="*.fav" | grep -v "runes/ctx/"
# → 0 件になること
```

---

### Step 4: driver.rs 更新

#### 4.1 cargo_toml_version_is_35_1_0 をスタブ化

```bash
grep -n "cargo_toml_version_is_35_1_0" fav/src/driver.rs
```

#### 4.2 v35200_tests の挿入

`v35100_tests` の終端 `}` の直後に v35200_tests を挿入（spec.md の通り）。

---

### Step 5: ビルド + テスト実行

```bash
cd /c/Users/yoshi/favnir/fav
cargo build 2>&1 | tail -5
cargo test --bin fav v35200 2>&1 | tail -8
cargo test 2>&1 | grep "test result"
cargo clippy --locked -- -D warnings 2>&1 | tail -5
```

---

### Step 6: CHANGELOG.md 更新

```markdown
## [v35.2.0] — 2026-07-04

### Changed
- `runes/` 配下 100 ファイル — `!Effect` アノテーションを廃止し Capability Context に移行
  - すべての public fn に `ctx: AppCtx` 第一引数を追加
  - `bind { field } <- ctx` で capability を束縛

### Notes
- v34.6A 補完実装: ロードマップが要求した Rune ファイル全件の ctx 移行を実施
```

---

### Step 7: benchmarks/v35.2.0.json 作成

```json
{
  "version": "35.2.0",
  "milestone": "Production Ready",
  "date": "2026-07-04",
  "tests_passed": 2596,
  "tests_failed": 0,
  "notes": "v34.6A補完: runes/ 100ファイル !Effect → ctx 移行完了。v35200_tests 5 件追加。"
}
```

---

### Step 8: versions/current.md 更新

- `最新安定版` → `**v35.2.0** — Rune ファイル ctx 移行（v34.6A）`
- `次に切る版` → `**v35.3.0** — Examples / E2E デモ ctx 移行（v34.7A）`

---

## テスト実行

```bash
cd /c/Users/yoshi/favnir/fav
cargo test --bin fav v35200 2>&1 | tail -8
cargo test 2>&1 | grep "test result"
```

---

## 完了処理

- `benchmarks/v35.2.0.json` の `tests_passed` を実測値で確定
- `tasks.md` を COMPLETE に更新
