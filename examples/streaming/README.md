# Favnir Streaming E2E デモ

Streaming Native フェーズ（v26.5〜v26.7）で実装した 3 本の E2E デモ。

## デモ一覧

| デモ | ファイル | テーマ |
|---|---|---|
| kafka → elasticsearch | `kafka_to_elasticsearch.fav` | リアルタイムログ集計 |
| kinesis → s3 | `kinesis_to_s3.fav` | クリックイベントアーカイブ |
| nats → postgres | `nats_to_postgres.fav` | IoT センサーデータ蓄積 |

---

## セットアップ

すべてのデモは同一の `docker-compose.yml` を使用する。

```bash
# 全サービス起動（healthcheck 完了まで待機）
docker compose -f examples/streaming/docker-compose.yml up -d --wait

# 状態確認
docker compose -f examples/streaming/docker-compose.yml ps
```

サービス一覧:

| サービス | ポート | 用途 |
|---|---|---|
| kafka（Redpanda） | 9092 | kafka_to_elasticsearch デモ |
| elasticsearch | 9200 | kafka_to_elasticsearch デモ |
| localstack（Kinesis/S3） | 4566 | kinesis_to_s3 デモ |
| nats | 4222 / 8222 | nats_to_postgres デモ |
| postgres | 5432 | nats_to_postgres デモ |
| pulsar | 6650 / 8080 | Pulsar Rune テスト用（v26.9.0〜、暫定 `!AWS` エフェクト） |

---

## デモ 1: kafka → elasticsearch（v26.5.0）

**テーマ**: Kafka から消費したアプリログを Elasticsearch にリアルタイムでインデックスする。

### 環境変数

| 変数名 | デフォルト値 | 説明 |
|---|---|---|
| `KAFKA_BOOTSTRAP_BROKERS` | `localhost:9092` | Kafka（Redpanda）ブローカー |
| `ELASTICSEARCH_URL` | `http://localhost:9200` | Elasticsearch エンドポイント |

### 実行

```bash
fav run examples/streaming/kafka_to_elasticsearch.fav
```

---

## デモ 2: kinesis → s3（v26.6.0）

**テーマ**: Kinesis から取得したクリックイベントを S3 にアーカイブする。

### 前提（LocalStack リソース作成）

```bash
# Kinesis ストリーム作成
aws --endpoint-url=http://localhost:4566 kinesis create-stream \
    --stream-name clickstream --shard-count 1

# S3 バケット作成
aws --endpoint-url=http://localhost:4566 s3 mb s3://clickstream-archive
```

### 環境変数

| 変数名 | デフォルト値 | 説明 |
|---|---|---|
| `KINESIS_ENDPOINT` | `http://localhost:4566` | Kinesis エンドポイント（LocalStack） |
| `AWS_ACCESS_KEY_ID` | `test` | LocalStack では任意値 |
| `AWS_SECRET_ACCESS_KEY` | `test` | LocalStack では任意値 |
| `AWS_DEFAULT_REGION` | `us-east-1` | AWS リージョン |

### 実行

```bash
fav run examples/streaming/kinesis_to_s3.fav
```

---

## デモ 3: nats → postgres（v26.7.0）

**テーマ**: NATS の `sensors.data` サブジェクトから IoT センサーデータを受信し、Postgres に蓄積する。

### 前提（Postgres テーブル作成）

```bash
docker exec -it $(docker compose -f examples/streaming/docker-compose.yml ps -q postgres) \
    psql -U favnir -d sensors -c "
    CREATE TABLE IF NOT EXISTS sensor_readings (
        id SERIAL PRIMARY KEY,
        data JSONB NOT NULL,
        received_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );"
```

### 環境変数

| 変数名 | デフォルト値 | 説明 |
|---|---|---|
| `NATS_URL` | `nats://localhost:4222` | NATS サーバー URL |
| `DATABASE_URL` | `host=localhost port=5432 user=favnir password=favnir dbname=sensors` | Postgres 接続文字列 |

### 実行

```bash
fav run examples/streaming/nats_to_postgres.fav
```

---

## 停止

```bash
docker compose -f examples/streaming/docker-compose.yml down
```

> **注意**: `down` コマンドを実行すると `postgres` サービスのデータが消失します（`volumes` マウントなし）。
> データを永続化したい場合は `down` の代わりに `stop` を使用してください。
