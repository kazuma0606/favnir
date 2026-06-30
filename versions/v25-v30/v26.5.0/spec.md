# v26.5.0 仕様書 — ストリーミング E2E デモ（kafka → elasticsearch）

## 概要

| 項目 | 内容 |
|---|---|
| バージョン | v26.5.0 |
| フェーズ | Streaming Native（v26.1〜v27.0） |
| テーマ | Kafka → Elasticsearch リアルタイムログ集計パイプライン E2E デモ |
| 依存関係 | v25.7.0（kafka Rune）・v25.8.0（elasticsearch Rune）・v26.4.0（Stream.* 操作）完了後 |
| 目標テスト数 | 2078 件（+8 件）|

---

## 背景と目的

v26.4.0 で `Stream.*` 操作 6 関数が実質化された。
v26.5.0 では Streaming Native フェーズの第 1 E2E デモとして、
Kafka から消費したログを Elasticsearch にインデックスするリアルタイムパイプラインを実装する。

ロードマップの「Kafka → 変換 → Elasticsearch のリアルタイムパイプラインが 50 行で書ける」
= Streaming Native の完成を象徴するデモに向けた最初のステップ。

### 既存 Rune の利用

本デモは実装済みの Rune を組み合わせる:

| Rune | 使用関数 | バージョン |
|---|---|---|
| kafka | `Kafka.connect("")`・`Kafka.consume_batch(conn, topic, group, count)` | v25.7.0 |
| elasticsearch | `Elasticsearch.connect("")`・`Elasticsearch.bulk(conn, idx, docs_json)` | v25.8.0 |

### ロードマップとの API 設計差異

ロードマップ v26.5 節のデモコードは `Kafka.consume[RawLog]("app-logs", "favnir-log-consumer")` 等の
型パラメータ付き理想 API を示しているが、現在の kafka Rune は `consume_batch(conn, topic, group_id, max_count)` を提供する。

v26.5.0 では実際の Rune API に合わせた `seq` パイプラインとして実装する:

```
FetchLogs(Unit -> String) |> FilterErrors(String -> String) |> IndexToES(String -> String)
```

追加の差異:
- ロードマップの `ES.index("logs-index")` は `ES.*` 名前空間を仮定しているが、実際の Rune は `import rune "elasticsearch"` で `Elasticsearch.*` 名前空間を提供する（既存の `elasticsearch_logs_etl.fav` で確認済み）
- ロードマップの `ParseLog`・`EnrichWithGeo` ステージは v26.5.0 スコープから除外する（型付きデシリアライズが未実装のため v27.x 以降）。`FilterErrors` のみ簡易 JSON 文字列マッチで実装する

---

## 機能仕様

### 1. `examples/streaming/kafka_to_elasticsearch.fav`

```favnir
import rune "kafka"
import rune "elasticsearch"

// ── Kafka → Elasticsearch リアルタイムログ集計デモ (v26.5.0) ─────────────────
// 前提: docker compose -f examples/streaming/docker-compose.yml up -d
// 実行: fav run examples/streaming/kafka_to_elasticsearch.fav
//
// 環境変数:
//   KAFKA_BOOTSTRAP_BROKERS  — Kafka ブローカー（省略: "localhost:9092"）
//   ELASTICSEARCH_URL        — ES エンドポイント（省略: "http://localhost:9200"）

// 1. Kafka から生ログをバッチ消費
stage FetchLogs: Unit -> Result<String, String> !Stream = |_| {
    bind conn <- Kafka.connect("")
    Kafka.consume_batch(conn, "app-logs", "favnir-log-consumer", 50)
}

// 2. WARN / ERROR を含むバッチのみ通過（簡易フィルタリング）
stage FilterErrors: String -> Result<String, String> !Pure = |batch_json| {
    if String.contains(batch_json, "ERROR") || String.contains(batch_json, "WARN")
    then Result.ok(batch_json)
    else Result.err("no error/warn logs in batch — skipping")
}

// 3. Elasticsearch の logs-index にバルクインデックス
stage IndexToES: String -> Result<String, String> !Elasticsearch = |batch_json| {
    bind conn <- Elasticsearch.connect("")
    bind _    <- Elasticsearch.bulk(conn, "logs-index", batch_json)
    Result.ok("indexed batch to logs-index")
}

seq LogPipeline = FetchLogs |> FilterErrors |> IndexToES
```

### 2. `examples/streaming/docker-compose.yml`

全依存サービスを定義。`docker compose up -d` 一発で起動できること。

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

### 3. `site/content/docs/streaming/kafka-to-elasticsearch.mdx`

- パイプライン構成図（FetchLogs → FilterErrors → IndexToES）
- Docker Compose 起動手順
- 環境変数設定
- ステージ別実装解説

---

## E2E 実行手順

```bash
# 1. サービス起動
docker compose -f examples/streaming/docker-compose.yml up -d

# 2. Kafka トピック作成（redpanda では起動後すぐに使用可能）
# （自動作成 or: rpk topic create app-logs）

# 3. デモ実行
fav run examples/streaming/kafka_to_elasticsearch.fav
```

> `FetchLogs` はスタブ Rune のため実際の Kafka 接続は確立せず、モック JSON を返す。
> 実環境テストは `#[ignore]` の Docker テストで対応（v27.0 以降）。

---

## Rust テスト（v265000_tests、8 件）

| テスト名 | 内容 | 期待値 |
|---|---|---|
| `kafka_to_es_demo_file_exists` | `examples/streaming/kafka_to_elasticsearch.fav` が存在する（compile check） | assert |
| `kafka_to_es_demo_has_consume` | デモに `consume` が含まれる | assert |
| `kafka_to_es_demo_has_bulk` | デモに `bulk` が含まれる（`Elasticsearch.bulk` 呼び出し確認） | assert |
| `kafka_to_es_demo_has_logs_index` | デモに `logs-index` が含まれる | assert |
| `kafka_to_es_demo_has_log_pipeline` | デモに `LogPipeline` が含まれる | assert |
| `streaming_docker_compose_exists` | `examples/streaming/docker-compose.yml` が存在する | assert |
| `streaming_docker_compose_has_kafka` | docker-compose.yml に `kafka` が含まれる | assert |
| `streaming_docker_compose_has_elasticsearch` | docker-compose.yml に `elasticsearch` が含まれる | assert |

> `changelog_has_v26_5_0` はテストから除外（CHANGELOG は実装完了後に追加するため、先行して count を固定する）。
> 代わりに `kafka_to_es_demo_has_log_pipeline` でパイプライン存在を確認する。

---

## 完了条件

- [ ] `fav/Cargo.toml` が `version = "26.5.0"` であること
- [ ] `examples/streaming/kafka_to_elasticsearch.fav` が存在すること
- [ ] デモに `seq LogPipeline` が定義されていること
- [ ] デモに `FetchLogs`・`FilterErrors`・`IndexToES` の 3 ステージが含まれること
- [ ] デモに `Kafka.consume_batch` 呼び出しが含まれること
- [ ] デモに `Elasticsearch.bulk` 呼び出しが含まれること（`bulk` キーワードで確認）
- [ ] デモに `"logs-index"` が含まれること
- [ ] `examples/streaming/docker-compose.yml` が存在すること
- [ ] docker-compose.yml に `kafka`（redpanda）サービスが含まれること
- [ ] docker-compose.yml に `elasticsearch` サービスが含まれること
- [ ] `site/content/docs/streaming/kafka-to-elasticsearch.mdx` が存在すること
- [ ] `CHANGELOG.md` に `[v26.5.0]` エントリが存在すること
- [ ] `benchmarks/v26.5.0.json` が存在すること（test_count: 2078）
- [ ] `v265000_tests` 8 件すべて PASS
- [ ] 総テスト数 ≥ 2078 件

---

## テスト件数

- v26.4.0 完了時: 2070 件
- v26.5.0 追加: 8 件（v265000_tests）
- **目標**: 2070 + 8 = **2078 件**

> `benchmarks/v26.4.0.json` で `test_count: 2070` を確認済み（実装前に Step 0 で再確認すること）。
