# v26.5.0 実装計画 — ストリーミング E2E デモ（kafka → elasticsearch）

## 実装方針

- 新規 Cargo 依存・Rust コードは追加しない（既存の kafka / elasticsearch Rune を使用）
- `examples/streaming/` ディレクトリを新規作成し、デモ .fav と docker-compose.yml を配置する
- Favnir コード（.fav）はコンパイルエラーなしで `fav run` できることを目標とする
- docker-compose.yml は実際に `docker compose up` できる内容にする（ポート・ヘルスチェック含む）
- ドキュメントページ（mdx）は `site/content/docs/streaming/` に新規作成

---

## 実装ステップ

### Step 0: 事前確認

```bash
grep 'version = ' fav/Cargo.toml                                 # "26.4.0" であること
cat benchmarks/v26.4.0.json                                      # "test_count":2070 であること
cargo test --bin fav 2>&1 | tail -3                              # 2070 件 PASS であること
ls examples/streaming/ 2>/dev/null || echo "not found"           # 未存在であること
```

### Step 1: `fav/Cargo.toml` bump（26.4.0 → 26.5.0）

```toml
version = "26.5.0"
```

### Step 2: `examples/streaming/kafka_to_elasticsearch.fav` 新規作成

spec.md §1 の内容を実装。3 ステージ + `seq LogPipeline`:

```favnir
import rune "kafka"
import rune "elasticsearch"

stage FetchLogs: Unit -> Result<String, String> !Stream = |_| {
    bind conn <- Kafka.connect("")
    Kafka.consume_batch(conn, "app-logs", "favnir-log-consumer", 50)
}

stage FilterErrors: String -> Result<String, String> !Pure = |batch_json| {
    if String.contains(batch_json, "ERROR") || String.contains(batch_json, "WARN")
    then Result.ok(batch_json)
    else Result.err("no error/warn logs in batch — skipping")
}

stage IndexToES: String -> Result<String, String> !Elasticsearch = |batch_json| {
    bind conn <- Elasticsearch.connect("")
    bind _    <- Elasticsearch.bulk(conn, "logs-index", batch_json)
    Result.ok("indexed batch to logs-index")
}

seq LogPipeline = FetchLogs |> FilterErrors |> IndexToES
```

### Step 3: `examples/streaming/docker-compose.yml` 新規作成

spec.md §2 の内容を実装。Kafka（Redpanda）・Elasticsearch の 2 サービス定義。

```yaml
version: "3.8"

services:
  kafka:
    image: redpandadata/redpanda:latest
    command:
      - redpanda
      - start
      - --overprovisioned
      - --node-id 0
      - --kafka-addr 0.0.0.0:9092
      - --advertise-kafka-addr localhost:9092
    ports:
      - "9092:9092"
      - "9644:9644"

  elasticsearch:
    image: elasticsearch:8.12.0
    environment:
      - discovery.type=single-node
      - xpack.security.enabled=false
      - ES_JAVA_OPTS=-Xms512m -Xmx512m
    ports:
      - "9200:9200"
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:9200/_cluster/health"]
      interval: 10s
      timeout: 5s
      retries: 10
```

### Step 4: `site/content/docs/streaming/kafka-to-elasticsearch.mdx` 新規作成

- パイプライン概要（FetchLogs → FilterErrors → IndexToES）
- Docker Compose 起動手順
- 環境変数（KAFKA_BOOTSTRAP_BROKERS / ELASTICSEARCH_URL）
- 各ステージの解説
- スコープ外（型付きデシリアライズ / ACK / リトライ）

> `site/content/docs/streaming/` ディレクトリが未存在の場合、Write ツールが自動作成する。

### Step 5: `CHANGELOG.md` 更新

```markdown
## [v26.5.0] — 2026-06-27 — ストリーミング E2E デモ（kafka → elasticsearch）

### Added
- `examples/streaming/kafka_to_elasticsearch.fav` — Kafka → Elasticsearch リアルタイムログ集計デモ（FetchLogs / FilterErrors / IndexToES）
- `examples/streaming/docker-compose.yml` — Kafka（Redpanda）/ Elasticsearch サービス定義
- `site/content/docs/streaming/kafka-to-elasticsearch.mdx` — E2E デモドキュメント
```

### Step 6: `benchmarks/v26.5.0.json` 新規作成

```json
{"version":"26.5.0","test_count":2078,"timestamp":"2026-06-27"}
```

### Step 7: `fav/src/driver.rs` に `v265000_tests` 追加

`v264000_tests` の直後に追加（8 件）:

```rust
// ── v265000_tests (v26.5.0) — kafka → elasticsearch E2E デモ ─────────────────
#[cfg(test)]
mod v265000_tests {
    #[test]
    fn kafka_to_es_demo_file_exists() {
        let src = include_str!("../../examples/streaming/kafka_to_elasticsearch.fav");
        assert!(!src.is_empty(), "kafka_to_elasticsearch.fav must not be empty");
    }
    #[test]
    fn kafka_to_es_demo_has_consume() {
        let src = include_str!("../../examples/streaming/kafka_to_elasticsearch.fav");
        assert!(src.contains("consume"), "demo must call consume");
    }
    #[test]
    fn kafka_to_es_demo_has_bulk() {
        let src = include_str!("../../examples/streaming/kafka_to_elasticsearch.fav");
        assert!(src.contains("bulk"), "demo must call Elasticsearch.bulk");
    }
    #[test]
    fn kafka_to_es_demo_has_logs_index() {
        let src = include_str!("../../examples/streaming/kafka_to_elasticsearch.fav");
        assert!(src.contains("logs-index"), "demo must reference logs-index");
    }
    #[test]
    fn kafka_to_es_demo_has_log_pipeline() {
        let src = include_str!("../../examples/streaming/kafka_to_elasticsearch.fav");
        assert!(src.contains("LogPipeline"), "demo must define LogPipeline");
    }
    #[test]
    fn streaming_docker_compose_exists() {
        let src = include_str!("../../examples/streaming/docker-compose.yml");
        assert!(!src.is_empty(), "docker-compose.yml must not be empty");
    }
    #[test]
    fn streaming_docker_compose_has_kafka() {
        let src = include_str!("../../examples/streaming/docker-compose.yml");
        assert!(src.contains("kafka"), "docker-compose.yml must define kafka service");
    }
    #[test]
    fn streaming_docker_compose_has_elasticsearch() {
        let src = include_str!("../../examples/streaming/docker-compose.yml");
        assert!(src.contains("elasticsearch"), "docker-compose.yml must define elasticsearch service");
    }
}
```

### Step 8: テスト確認

```bash
cd fav && cargo test v265000 --bin fav          # 8/8 PASS
cd fav && cargo test --bin fav -j 8 -- --test-threads=8 2>&1 | tail -4  # 2078 件 PASS
```

---

## ファイル変更一覧

| ファイル | 操作 |
|---|---|
| `fav/Cargo.toml` | version bump 26.4.0 → 26.5.0 |
| `examples/streaming/kafka_to_elasticsearch.fav` | **新規作成**（3 ステージ + seq） |
| `examples/streaming/docker-compose.yml` | **新規作成** |
| `site/content/docs/streaming/kafka-to-elasticsearch.mdx` | **新規作成** |
| `CHANGELOG.md` | `[v26.5.0]` エントリ先頭に追加 |
| `benchmarks/v26.5.0.json` | **新規作成** |
| `fav/src/driver.rs` | `v265000_tests`（8 件）追加 |

---

## 注意事項

- `examples/streaming/` ディレクトリは未存在 → Write ツールが自動作成する。
- `site/content/docs/streaming/` ディレクトリも未存在 → 同様に自動作成。
- `.fav` ファイルの `import rune "kafka"` と `import rune "elasticsearch"` で名前空間が `Kafka.*` / `Elasticsearch.*` になることを確認（既存の `kafka_events_etl.fav` のパターンを参照）。
- `include_str!` のパスは `fav/src/driver.rs` から見た相対パス:
  - `"../../examples/streaming/kafka_to_elasticsearch.fav"`
  - `"../../examples/streaming/docker-compose.yml"`
- docker-compose.yml の `version: "3.8"` は最新 Docker Compose では非推奨だが後方互換のため残す。
- Kafka Rune のスタブ実装では実際の Kafka 接続は確立しない（モック値を返す）。

## リスクと対応

| リスク | 対応 |
|---|---|
| `import rune "kafka"` と `import rune "elasticsearch"` の名前空間が衝突 | 両 Rune は異なる名前空間（`Kafka.*` / `Elasticsearch.*`）を使用するため衝突しない |
| `.fav` ファイルの `!Stream !Elasticsearch` 複合エフェクトがチェッカーで弾かれる | `seq` pipeline の各 stage に異なるエフェクトを持たせる設計（stage 間のエフェクト境界が明確） |
| docker-compose.yml の elasticsearch イメージが古い可能性 | `elasticsearch:8.12.0` は LTS に近いバージョン。ローカル検証不要（構造テストのみ） |
