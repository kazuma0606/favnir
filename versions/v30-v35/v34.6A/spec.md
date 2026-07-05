# v34.6A — Spec

## 概要

**テーマ**: Rune ファイル全件の `!Effect` → Capability Context 移行

**バージョン番号**: Cargo.toml `35.2.0`（プロジェクト追跡名 v34.6A）

**背景**: v34.6.0 では W023 lint ルール（チェックポイント）のみ実装し、
ロードマップが要求する「Rune ファイル（`runes/` 配下）の `!Effect` → ctx 移行」が未実装だった。
本バージョンはその補完実装（差分）である。

---

## 実装されていなかった要件（ロードマップ原文）

`versions/roadmap/roadmap-v33.1-v34.0.md` の「移行対象（破壊的変更）」より:

> 3. **Rune ファイル**（`runes/` 配下、全 50+ rune）
>    - `runes/postgres/client.fav` 等の `!Postgres` → `ctx.db: PgConn`
>    - `runes/redis/redis.fav` の `!Redis` → `ctx.redis: RedisClient`
>    - 他すべての rune の `!Effect` 宣言

実測: `runes/` 配下で `!Effect` アノテーションを持つファイルは **100 件**。

---

## 移行パターン

### 標準パターン（副作用あり fn）

```favnir
// Before（現状）
public fn execute(sql: String, params: String) -> Result<Unit, String> !Postgres {
    Postgres.execute_raw(sql, params)
}

// After（ctx 構文）
public fn execute(ctx: AppCtx, sql: String, params: String) -> Result<Unit, String> {
    bind { db } <- ctx
    Postgres.execute_with_conn_raw(db, sql, params)
}
```

### `!Effect` → ctx フィールド対応表

| 廃止する `!Effect` | ctx フィールド | フィールド型 |
|---|---|---|
| `!Io` / `!File` | `ctx.io` | `IoCtx` |
| `!Http` / `!Network` / `!Rpc` | `ctx.http` | `HttpClient` |
| `!DbRead` / `!DbWrite` / `!Db` | `ctx.db` | `DbConn` |
| `!Postgres` | `ctx.db` | `PgConn` |
| `!MySQL` | `ctx.db` | `MySqlConn` |
| `!MongoDB` | `ctx.db` | `MongoConn` |
| `!DynamoDB` | `ctx.db` | `DynamoConn` |
| `!Elasticsearch` | `ctx.db` | `EsConn` |
| `!Redis` | `ctx.redis` | `RedisClient` |
| `!Stream` / `!Checkpoint` | `ctx.stream` | `StreamClient` |
| `!Trace` | `ctx.tracer` | `Tracer` |
| `!Snowflake` | `ctx.warehouse` | `SnowflakeConn` |
| `!Llm` | `ctx.llm` | `LlmClient` |
| `!Gcp` | `ctx.http` | `HttpClient` |
| `!DbRead` / `!DbWrite` / `!DbAdmin` | `ctx.db` | `DbConn` |
| `!Network` / `!Rpc` | `ctx.http` | `HttpClient` |
| `!File` | `ctx.io` | `IoCtx` |
| `!Checkpoint` | `ctx.stream` | `StreamClient` |
| `!AzureDb` | `ctx.db` | `DbConn` |
| `!AzureStorage` | `ctx.io` | `IoCtx` |
| `!PipelineState` | `ctx.stream` | `StreamClient` |
| `!Emit<T>` | `ctx.emitter` | `Emitter<T>` |

### 複数 !Effect がある場合

```favnir
// Before
fn process(sql: String, url: String) -> Result<String, String> !Postgres !Http {
    ...
}

// After
fn process(ctx: AppCtx, sql: String, url: String) -> Result<String, String> {
    bind { db, http } <- ctx
    ...
}
```

---

## 移行対象ファイル一覧（100 件）

`runes/` 配下の以下すべてのファイル（`-> .* !\w+` パターンを含むもの）:

```
runes/airtable/airtable.fav        runes/shopify/shopify.fav
runes/auth/apikey.fav              runes/slack/slack.fav
runes/auth/jwt.fav                 runes/snowflake/client.fav
runes/auth/oauth2.fav              runes/snowflake/snowflake.fav
runes/aws/dynamodb.fav             runes/snowflake/snowflake_db.fav
runes/aws/s3.fav                   runes/sql/query.fav
runes/aws/s3_storage.fav           runes/sqlite/sqlite.fav
runes/aws/secrets.fav              runes/sqs/sqs.fav
runes/aws/sqs.fav                  runes/stat/stat.fav
runes/azure-blob/azure_blob.fav    runes/stat/stat.test.fav
runes/azure-postgres/azure_postgres.fav  runes/state/state.fav
runes/azure-postgres/client.fav    runes/stripe/stripe.fav
runes/bigquery/bigquery.fav        runes/toml/toml.fav
runes/cache/cache.fav              runes/twilio/twilio.fav
runes/clickhouse/clickhouse.fav    runes/vertex-ai/vertex-ai.fav
runes/csv/csv.fav                  runes/zendesk/zendesk.fav
runes/datadog/datadog.fav          runes/db/connection.fav
runes/db/db.test.fav               runes/db/migration.fav
runes/db/query.fav                 runes/db/transaction.fav
runes/dbt/dbt.fav                  runes/delta-lake/delta-lake.fav
runes/duckdb/duckdb.test.fav       runes/duckdb/io.fav
runes/duckdb/query.fav             runes/dynamodb/dynamodb.fav
runes/elasticsearch/elasticsearch.fav  runes/email/email.fav
runes/env/access.fav               runes/env/dotenv.fav
runes/env/env_impl.fav             runes/env/typed.fav
runes/fs/fs.fav                    runes/gen/hint.fav
runes/gen/output.fav               runes/gen/primitives.fav
runes/gen/structured.fav           runes/github/github.fav
runes/grafana/grafana.fav          runes/graphql/client.fav
runes/graphql/graphql.fav          runes/grpc/client.fav
runes/grpc/server.fav              runes/hubspot/hubspot.fav
runes/http/client.fav              runes/http/http_client_impl.fav
runes/http/request.fav             runes/http/retry.fav
runes/iceberg/iceberg.fav          runes/incremental/checkpoint.fav
runes/incremental/pipeline.fav     runes/intercom/intercom.fav
runes/io/io_impl.fav               runes/jsonl/jsonl.fav
runes/kafka/kafka.fav              runes/kinesis/kinesis.fav
runes/linear/linear.fav            runes/llm/client.fav
runes/llm/llm.fav                  runes/log/emitter.fav
runes/log/metric.fav               runes/mlflow/mlflow.fav
runes/mongodb/mongodb.fav          runes/mysql/mysql.fav
runes/nats/nats.fav                runes/notion/notion.fav
runes/otel/otel.fav                runes/pagerduty/pagerduty.fav
runes/pinecone/pinecone.fav        runes/postgres/client.fav
runes/postgres/postgres.fav        runes/postgres/postgres_db.fav
runes/prometheus/prometheus.fav    runes/pulsar/pulsar.fav
runes/queue/queue.fav              runes/rabbitmq/rabbitmq.fav
runes/redshift/redshift.fav        runes/redis/redis.fav
runes/rune_loader/loader.fav       runes/sagemaker/sagemaker.fav
runes/sendgrid/sendgrid.fav        runes/sentry/sentry.fav
```

（`runes/ctx/` の 4 ファイルはすでに ctx 構文 — 除外）

---

## 実装スコープ

### 変更ファイル

1. `fav/Cargo.toml` — version `35.1.0` → `35.2.0`
2. `runes/**/*.fav` — 100 ファイルの `!Effect` → ctx 移行
3. `fav/src/driver.rs` — `cargo_toml_version_is_35_1_0` をスタブ化、`v35200_tests` 5 件追加
4. `benchmarks/v35.2.0.json` — 新規作成
5. `CHANGELOG.md` — `[v35.2.0]` セクション先頭追記
6. `versions/current.md` — 最新安定版を v35.2.0 に更新

---

## テスト仕様（v35200_tests）

```rust
// ── v35.2.0 tests (v34.6A supplement: runes/ !Effect → ctx migration) ──
#[cfg(test)]
mod v35200_tests {
    #[test]
    fn cargo_toml_version_is_35_2_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("35.2.0"), "Cargo.toml must contain '35.2.0'");
    }

    #[test]
    fn postgres_client_uses_ctx_syntax() {
        let src = include_str!("../../runes/postgres/client.fav");
        assert!(
            src.contains("ctx: AppCtx"),
            "runes/postgres/client.fav must use ctx: AppCtx parameter"
        );
        assert!(
            !src.contains("!Postgres"),
            "runes/postgres/client.fav must not contain !Postgres annotation"
        );
    }

    #[test]
    fn redis_rune_uses_ctx_syntax() {
        let src = include_str!("../../runes/redis/redis.fav");
        assert!(
            src.contains("ctx: AppCtx"),
            "runes/redis/redis.fav must use ctx: AppCtx parameter"
        );
        assert!(
            !src.contains("!Redis"),
            "runes/redis/redis.fav must not contain !Redis annotation"
        );
    }

    #[test]
    fn kafka_rune_uses_ctx_syntax() {
        let src = include_str!("../../runes/kafka/kafka.fav");
        assert!(
            src.contains("ctx: AppCtx"),
            "runes/kafka/kafka.fav must use ctx: AppCtx parameter"
        );
        assert!(
            !src.contains("!Stream"),
            "runes/kafka/kafka.fav must not contain !Stream annotation"
        );
    }

    #[test]
    fn http_client_rune_uses_ctx_syntax() {
        let src = include_str!("../../runes/http/client.fav");
        assert!(
            src.contains("ctx: AppCtx"),
            "runes/http/client.fav must use ctx: AppCtx parameter"
        );
        assert!(
            !src.contains("!Http"),
            "runes/http/client.fav must not contain !Http annotation"
        );
    }
}
```

### 設計注記

- `v35200_tests` は `v35100_tests` 直後・`// ── v31.7.0 tests` の前に挿入
- `use super::*` なし
- 否定チェック（`!src.contains("!Postgres")`）と肯定チェック（`src.contains("ctx: AppCtx")`）の両方を実施

---

## 完了条件

- [ ] `Cargo.toml` version = `"35.2.0"`
- [ ] `cargo_toml_version_is_35_1_0` が空スタブになっていること
- [ ] `runes/` 配下 100 件すべてに `!Effect` アノテーションが残存しないこと
- [ ] `runes/` 配下の公開 fn がすべて `ctx: AppCtx` 第一引数を持つこと
- [ ] `cargo test --bin fav v35200` — 5/5 PASS
- [ ] `cargo test` — 全件 PASS（2596 件想定 = 2591 + 5、0 failures）
- [ ] `cargo clippy --locked -- -D warnings` — PASS
- [ ] `CHANGELOG.md` に `[v35.2.0]` セクション
- [ ] `benchmarks/v35.2.0.json` 存在かつ `tests_passed` が実測値
- [ ] `benchmarks/v35.2.0.json` の `tests_failed` が `0`
- [ ] `versions/current.md` が v35.2.0 に更新されていること
- [ ] `tasks.md` が COMPLETE
