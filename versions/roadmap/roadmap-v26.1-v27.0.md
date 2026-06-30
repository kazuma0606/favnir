# Roadmap v26.1.0 〜 v27.0.0 — Streaming Native

Date: 2026-06-24
**完了日**: 2026-06-27（v27.0.0 — Streaming Native 宣言）

## 目標

v26.0「Rune Foundation」でコア 8 Rune が動くようになった。
`fav run examples/full_etl.fav`（postgres → 集計 → s3 → kafka 通知）が実際の Docker 環境で動く。

しかし現状の Kafka 連携は「ポーリング + バッチ処理」の組み合わせに過ぎない。
本当のストリーミング——Kafka から届いたイベントを即座に変換して Elasticsearch に流す——は
まだ型安全に書けない。

このフェーズでは、**ストリーミングパイプラインを型安全に記述できる言語基盤を完成させる**。
AWS Kinesis / NATS / RabbitMQ / SQS / Pulsar を実質化し、
`#[streaming]` stage に本物のバックプレッシャー制御を加え、
「リアルタイムパイプラインが 50 行で書ける」を達成する。

> **Streaming Native の定義（本プロジェクト固有）**
> 「ストリーミング Rune 5 本が実質化され、`Stream.*` 操作 6 関数が使え、
>  E2E デモ 3 本が Docker Compose で動く」状態を指す。

**完了条件（最終テスト）:**

```bash
# 1. 全 Rust テストが通る
cargo test

# 2. ストリーミング Rune が各自動テストを通過する
cargo test kinesis nats rabbitmq sqs pulsar

# 3. Stream.* 操作のテストが通る
cargo test streaming

# 4. E2E デモが Docker Compose で動く
docker compose up kafka elasticsearch kinesis nats postgres -d
fav run examples/streaming/kafka_to_elasticsearch.fav
fav run examples/streaming/kinesis_to_s3.fav
fav run examples/streaming/nats_to_postgres.fav
```

---

## 設計決定事項

| 項目 | 決定 |
|---|---|
| バックプレッシャーの実装方式 | `bounded channel`（Rust の `tokio::sync::mpsc` ベース）。容量超過でプロデューサをブロック |
| `Stream<T>` 型の表現 | `!streaming` エフェクトを持つ `stage` の出力型として内部で扱う。Favnir 構文は変えない |
| ウィンドウの種類（v26.4 対象） | タンブリングウィンドウのみ（スライディング・セッションは Streaming Native 後のフェーズ） |
| kinesis のローカル環境 | LocalStack（v25.2 で整備済み）。KCL ライブラリは不使用 |
| nats のローカル環境 | `nats-server` Docker イメージ。JetStream 有効化 |
| rabbitmq のローカル環境 | `rabbitmq:3-management` Docker イメージ |
| sqs のローカル環境 | LocalStack（v25.2 / v25.6 で整備済み） |
| pulsar のローカル環境 | `apachepulsar/pulsar` Docker イメージ。standalone モード |
| E2E デモの前提条件 | `docker compose up` 一発で動くこと。手動設定不要 |
| 破壊的変更 | なし（STABILITY.md v1.x ポリシーに従う） |

---

## バージョン計画

### v26.1 — kinesis Rune 実質化

**テーマ**: AWS ストリーミングの中心。Kafka Rune（v25.7）と API 設計の一貫性を保ちながら実装する。

**依存関係**: v25.7（kafka Rune）完了後推奨（`produce / consume` API 設計の参照）

```favnir
import runes/kinesis

// Kinesis からリアルタイムで注文イベントを消費する
seq OrderStream =
  Kinesis.consume[OrderEvent]("orders", "favnir-consumer")
  |> ValidateOrder
  |> EnrichWithProduct
  |> Postgres.insert("processed_orders")
```

実装する関数:

| 関数 | 内容 |
|---|---|
| `Kinesis.put_record(stream, key, data)` | レコード送信（パーティションキー指定） |
| `Kinesis.put_records(stream, records)` | バッチ送信（最大 500 件、10 MB 以内） |
| `Kinesis.get_records[T](shard, iterator)` | 型付き取得（BSON → T デシリアライズ） |
| `Kinesis.get_shard_iterator(stream, shard, type)` | イテレータ取得（LATEST / TRIM_HORIZON / AT_SEQUENCE）|
| `Kinesis.consume[T](stream, group, fn)` | 継続消費ループ（Enhanced Fan-Out 対応） |

LocalStack で全関数が動作。`cargo test kinesis` で 5 件以上 PASS。

---

### v26.2 — nats Rune 実質化

**テーマ**: 軽量・高速メッセージング。IoT / マイクロサービス・エッジコンピューティングの定番。
JetStream の永続メッセージ機能まで含めて実装する。

**依存関係**: v26.1 と並行可能

実装する関数:

| 関数 | 内容 |
|---|---|
| `NATS.publish(subject, payload)` | メッセージ発行（コア NATS） |
| `NATS.subscribe[T](subject, fn)` | 型付き購読（コア NATS） |
| `NATS.request[T](subject, payload, timeout)` | リクエスト/レスポンス（タイムアウト付き） |
| `NATS.jetstream_publish(stream, payload)` | JetStream 永続メッセージ送信 |
| `NATS.jetstream_consume[T](stream, consumer, fn)` | JetStream 永続消費（ACK 対応） |

`nats-server`（Docker）で全関数が動作。`cargo test nats` で 5 件以上 PASS。

---

### v26.3 — rabbitmq Rune 実質化

**テーマ**: エンタープライズ MQ の定番。AMQP プロトコル。
Exchange / Queue / Binding の概念を Favnir の型システムで表現する。

**依存関係**: v26.2 と並行可能

実装する関数:

| 関数 | 内容 |
|---|---|
| `RabbitMQ.connect(config)` | 接続確立（TLS 対応） |
| `RabbitMQ.declare_exchange(name, type, opts)` | Exchange 宣言（direct / fanout / topic） |
| `RabbitMQ.declare_queue(name, opts)` | Queue 宣言（durable / exclusive / auto-delete） |
| `RabbitMQ.bind(queue, exchange, routing_key)` | Binding 設定 |
| `RabbitMQ.publish(exchange, routing_key, msg)` | メッセージ発行 |
| `RabbitMQ.consume[T](queue, fn)` | 型付き消費ループ |
| `RabbitMQ.ack(delivery)` | 配信確認（ACK） |
| `RabbitMQ.nack(delivery, requeue)` | 否定確認（NACK、再キューオプション付き） |

`rabbitmq:3-management`（Docker）で全関数が動作。`cargo test rabbitmq` で 4 件以上 PASS。

---

### v26.4 — `#[streaming]` バックプレッシャー対応 + `Stream.*` 操作

**テーマ**: 現在の `#[streaming]` は chunk_size ベースの単純ポーリング。
本物のストリーミングセマンティクス（バックプレッシャー・ウィンドウ・マージ）を実装する。

**依存関係**: v26.1〜v26.3 完了後（実際の Rune とのインテグレーション検証のため）

```favnir
// バックプレッシャー対応ストリーミング stage
#[streaming(chunk_size: 1000, backpressure: true)]
stage TransformLogs: Stream<RawLog> -> Stream<ParsedLog> !Io = |stream| {
  Stream.map(stream, |log| {
    ParsedLog {
      timestamp: DateTime.parse(log.ts),
      level:     LogLevel.from_str(log.level),
      message:   log.msg,
      source:    log.host
    }
  })
}

// タンブリングウィンドウで 1 分ごとに集計
#[streaming(window: 60)]
stage AggregatePerMinute: Stream<ParsedLog> -> Stream<LogSummary> !Io = |stream| {
  Stream.window(stream, 60, |batch| {
    LogSummary {
      count:       List.length(batch),
      error_count: batch |> List.filter(|l| l.level == "ERROR") |> List.length,
      period_start: batch |> List.head |> Option.map(|l| l.timestamp)
    }
  })
}
```

実装する操作:

| 操作 | 内容 |
|---|---|
| `Stream.map[A, B](stream, fn)` | 各要素を変換（型変換保証） |
| `Stream.filter[T](stream, predicate)` | 条件を満たす要素のみ通過 |
| `Stream.flat_map[A, B](stream, fn)` | 各要素を展開してから変換（1→N 変換） |
| `Stream.window(stream, size_secs, fn)` | タンブリングウィンドウ（N 秒ごとにバッチ化） |
| `Stream.merge(streams)` | 複数ストリームを 1 つに合流 |
| `Stream.split(stream, predicate)` | 条件で 2 ストリームに分岐 |

`cargo test streaming` で 6 件以上 PASS。
`#[streaming(backpressure: true)]` が VM で実際にバックプレッシャーを機能させることを確認。

---

### v26.5 — ストリーミング E2E デモ（kafka → elasticsearch）

**テーマ**: Kafka → 変換 → Elasticsearch のリアルタイムログ集計パイプライン。

**依存関係**: v25.7（kafka）・v25.8（elasticsearch）・v26.4（Stream.*）完了後

```favnir
// examples/streaming/kafka_to_elasticsearch.fav
seq LogPipeline =
  Kafka.consume[RawLog]("app-logs", "favnir-log-consumer")
  |> ParseLog
  |> EnrichWithGeo
  |> FilterErrors
  |> ES.index("logs-index")

stage ParseLog: RawLog -> ParsedLog !Pure = |log| {
  ParsedLog {
    timestamp: DateTime.parse(log.ts),
    level:     LogLevel.from_str(log.level),
    message:   log.msg,
    host:      log.host
  }
}

stage FilterErrors: ParsedLog -> ParsedLog !Pure = |log| {
  // WARN 以上のみ ES に流す
  if log.level == "ERROR" || log.level == "WARN"
  then Result.ok(log)
  else Result.err("skip")
}
```

`examples/streaming/kafka_to_elasticsearch.fav` を作成し、
`docker compose up kafka elasticsearch -d && fav run examples/streaming/kafka_to_elasticsearch.fav`
が動くことを確認する。`examples/streaming/docker-compose.yml` に全依存サービスを定義する。

---

### v26.6 — ストリーミング E2E デモ（kinesis → s3）

**テーマ**: Kinesis → s3 のイベントアーカイブパイプライン。
AWS ユーザーの典型的なログアーカイブパターン。

**依存関係**: v25.2（s3）・v26.1（kinesis）・v26.4（Stream.*）完了後

```favnir
// examples/streaming/kinesis_to_s3.fav
seq ArchivePipeline =
  Kinesis.consume[ClickEvent]("clickstream", "archive-consumer")
  |> BatchEvents
  |> SerializeToParquet
  |> UploadToS3

// 1000 件または 30 秒ごとにバッチ化
#[streaming(window: 30)]
stage BatchEvents: Stream<ClickEvent> -> Stream<List<ClickEvent>> !Pure = |stream| {
  Stream.window(stream, 30, |batch| batch)
}
```

`examples/streaming/kinesis_to_s3.fav` を作成し、LocalStack で動くことを確認する。

---

### v26.7 — ストリーミング E2E デモ（nats → postgres）

**テーマ**: NATS から届く IoT センサーデータを Postgres に蓄積するパイプライン。

**依存関係**: v25.1（postgres）・v26.2（nats）・v26.4（Stream.*）完了後

```favnir
// examples/streaming/nats_to_postgres.fav
seq SensorPipeline =
  NATS.subscribe[SensorReading]("sensors.>")
  |> ValidateSensor
  |> EnrichWithMetadata
  |> Postgres.insert("sensor_readings")

stage ValidateSensor: SensorReading -> SensorReading !Pure = |reading| {
  if reading.value > -100.0 && reading.value < 1000.0
  then Result.ok(reading)
  else Result.err("out of range: " ++ Float.to_string(reading.value))
}
```

`examples/streaming/nats_to_postgres.fav` を作成し、Docker で動くことを確認する。
`examples/streaming/README.md` に 3 本のデモの実行手順をまとめる。

---

### v26.8 — sqs Rune 実質化

**テーマ**: AWS 非同期処理の基礎。Lambda トリガーとの連携。
SQS は kinesis より単純な「キュー」モデルで、確実な 1 回以上配信を保証する。

**依存関係**: v25.2（LocalStack AWS 認証基盤）完了後

実装する関数:

| 関数 | 内容 |
|---|---|
| `SQS.send_message(queue_url, body)` | メッセージ送信（遅延配信オプション付き） |
| `SQS.send_message_batch(queue_url, messages)` | バッチ送信（最大 10 件） |
| `SQS.receive_messages[T](queue_url, max)` | 型付きメッセージ受信（最大 10 件） |
| `SQS.delete_message(queue_url, receipt)` | 受信確認（削除） |
| `SQS.purge(queue_url)` | キュー全削除（テスト用） |
| `SQS.consume[T](queue_url, fn)` | 継続消費ループ（自動削除オプション）|

LocalStack で全関数が動作。`cargo test sqs` で 4 件以上 PASS。

---

### v26.9 — pulsar Rune 追加

**テーマ**: Kafka の代替。マルチテナント・マルチクラスタ対応。
geo-replication 機能を持ち、グローバルな分散ストリーミングに適する。

**依存関係**: v26.3（RabbitMQ）完了後推奨（テナント / namespace の概念設計参照）

実装する関数:

| 関数 | 内容 |
|---|---|
| `Pulsar.produce(topic, key, value)` | メッセージ送信（スキーマ検証付き） |
| `Pulsar.consume[T](topic, subscription, fn)` | 型付き消費（Exclusive / Shared / Key_Shared）|
| `Pulsar.ack(msg)` | ACK（確認） |
| `Pulsar.nack(msg)` | NACK（再配信要求） |

`apachepulsar/pulsar`（Docker standalone）で動作。`cargo test pulsar` で 3 件以上 PASS。

---

## v27.0 — Streaming Native マイルストーン宣言

**完了条件:**

| コンポーネント | 完了基準 |
|---|---|
| kinesis Rune | 5 条件クリア + 5 件テスト + LocalStack 動作 |
| nats Rune | 5 条件クリア + 5 件テスト |
| rabbitmq Rune | 5 条件クリア + 4 件テスト |
| sqs Rune | 5 条件クリア + 4 件テスト + LocalStack 動作 |
| pulsar Rune | 5 条件クリア + 3 件テスト |
| `#[streaming]` バックプレッシャー対応 | `cargo test streaming` 6 件 PASS |
| `Stream.*` 操作 6 関数 | map / filter / flat_map / window / merge / split |
| E2E デモ（kafka→ES） | Docker Compose で動作 |
| E2E デモ（kinesis→s3） | LocalStack で動作 |
| E2E デモ（nats→postgres） | Docker で動作 |

**最終テスト（全件 PASS が完了条件）:**

```bash
# 1. 全 Rust テストが通る
cargo test

# 2. ストリーミング Rune テスト
cargo test kinesis nats rabbitmq sqs pulsar streaming

# 3. E2E デモ全 3 本
docker compose -f examples/streaming/docker-compose.yml up -d
fav run examples/streaming/kafka_to_elasticsearch.fav
fav run examples/streaming/kinesis_to_s3.fav
fav run examples/streaming/nats_to_postgres.fav
```

> 「Kafka → 変換 → Elasticsearch のリアルタイムパイプラインが 50 行で書ける」
> = Streaming Native の完成を象徴するデモ

---

## 参考リンク

- マスタースケジュール: `versions/roadmap/roadmap-v25.1-v30.0.md`
- 前フェーズ: `versions/roadmap/roadmap-v25.1-v26.0.md`
- 次フェーズ: `versions/roadmap/roadmap-v27.1-v28.0.md`
