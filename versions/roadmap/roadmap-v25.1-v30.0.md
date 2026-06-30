# Favnir Master Roadmap — v25.1 〜 v30.0

Date: 2026-06-24
Status: 計画中（v25.0.0 完了時点）

---

## 背景と方針

v25.0.0「Practical Self-Hosting」の宣言をもって、Favnir は以下を達成した:

```
v21.0 — Runtime Excellence    : VM が限界まで速い        ✓
v22.0 — Developer Tooling     : 開発体験が最高           ✓
v23.0 — Distributed Scale     : スケールアウトできる      ✓
v24.0 — VM in Favnir          : Favnir が Favnir を動かす ✓
v25.0 — Practical Self-Hosting : 自己記述として完成       ✓
```

v1.0 リリース候補として言語の「完成」を宣言した今、次のフェーズは
**「使われる言語」** への転換である。

言語機能の追加より、**エコシステムの実質化** を最優先とする。
50 を超える Rune カタログの大半はインターフェース定義（スタブ）に留まっており、
実際のデータエンジニアリング業務で使える完全実装が必要だ。

```
v26.0 — Rune Foundation       : 「コア Rune が本当に動く」
v27.0 — Streaming Native      : 「ストリームが型安全に流れる」
v28.0 — Data Lakehouse        : 「現代のデータ基盤に溶け込む」
v29.0 — Observability First   : 「パイプラインの内側が見える」
v30.0 — Ecosystem Maturity    : 「コミュニティが Rune を育てる」
```

### バージョン命名規則

| 種別 | 意味 |
|---|---|
| **x.0.0** — マイルストーン宣言版 | 直前の x-1.1〜x-1.9 の成果をまとめて宣言する。破壊的変更なし・新機能追加なし。 |
| **x.1〜x.9** — 実装版 | 各 Rune / 機能 / デモを 1 バージョン 1 テーマで順次実装する。 |

### 破壊的変更なし原則

v25.0.0 で STABILITY.md を確定した通り、v1.x（v25.x〜v30.x）では破壊的変更を行わない。
既存コードは常に動き続ける。新機能はすべて **追加のみ** で提供する。

### vm.fav Phase 6 の位置づけ

`CallFn` オペコード（ユーザー定義関数ディスパッチ）は v26.x 中に完成させる。
これはコア機能の完成であり、破壊的変更ではない。
完成した時点で「Favnir で書いた VM が Favnir を完全に実行する」が実現する。

### 「動く Rune」の定義

スタブ（インターフェース定義のみ）を「実質化」する基準:

```
1. connect  — 接続・認証が実際に確立できる
2. read     — データを Favnir の型として読み込める
3. write    — Favnir のデータを書き込める
4. error    — 失敗時に型付きエラーが返る
5. test     — cargo test でモックを使った自動テストが通る
```

この 5 条件を満たした Rune を「実質化済み」と定義する。

---

## v26.0 — Rune Foundation

**テーマ**: 「コア Rune が本当に動く」
**期間**: v25.1〜v25.9 → v26.0 マイルストーン宣言

### なぜ Practical Self-Hosting の次か

言語の自己記述能力が完成した今、最大の課題は「実際のデータパイプラインを書けない」ことだ。
postgres Rune で DB に繋ごうとするとスタブだった、s3 から CSV を読もうとしたら動かなかった
——こうした「最初の 30 分での詰まり」が採用の障壁になる。

コア 8 Rune（postgres / s3 / redis / mysql / mongodb / dynamodb / kafka / elasticsearch）を
完全実装することで「Favnir で書いたパイプラインが実際に動く」を実現する。

### 対象ユーザーへの価値

- 既存の Python / dbt パイプラインを Favnir に移行できる
- 「型安全 ETL」の恩恵を実業務で受けられる
- CI でパイプラインの正しさを自動検証できる

### やらないこと（スコープ外）

- 新しい言語機能・構文の追加
- Rune レジストリの公開基盤（→ v30.0）
- ストリーミング型のパイプライン（バッチ ETL のみ）— (→ v27.0)
- Rune の自動コード生成 / スキャフォールド

---

### v25.1 — postgres Rune 実質化

**依存関係**: なし（v25.0.0 で Rune スタブ + `runes/postgres/` ディレクトリが存在）

データエンジニアリングで最も使われる DB。

#### 実装内容

```favnir
// 接続・クエリ・トランザクション・プール
import runes/postgres

stage LoadUsers: Unit -> List<User> !Db = |_| {
  bind conn <- Postgres.connect(config.postgres)
  bind rows <- Postgres.query[User](conn, "SELECT * FROM users WHERE active = $1", [true])
  Result.ok(rows)
}

stage SaveResult: List<Summary> -> Unit !Db = |summaries| {
  bind conn <- Postgres.connect(config.postgres)
  bind _ <- Postgres.execute_many(conn, "INSERT INTO summaries VALUES ($1, $2)", summaries)
  Result.ok(unit)
}
```

#### 実装する関数

| 関数 | 内容 |
|---|---|
| `Postgres.connect(config)` | 接続確立（SSL 対応） |
| `Postgres.query[T](conn, sql, params)` | 型付きクエリ（行 → T にデシリアライズ） |
| `Postgres.execute(conn, sql, params)` | 更新・削除・DDL |
| `Postgres.execute_many(conn, sql, rows)` | バッチ挿入（COPY 利用） |
| `Postgres.transaction(conn, fn)` | トランザクション（ロールバック自動） |
| `Postgres.Pool.create(config)` | コネクションプール（v20.8 の機能を Rune に昇格） |

#### 完了条件

- `runes/postgres/postgres.fav` の全関数が Rust バックエンドで実装済み
- `cargo test` で MockPostgres を使ったテスト 5 件以上 PASS
- `fav run examples/postgres_etl.fav` が実際に動く（ローカル Docker + Postgres で検証）

#### 検証コマンド

```bash
cargo test postgres
fav run examples/postgres_etl.fav
```

---

### v25.2 — s3 Rune 実質化

**依存関係**: v25.1 完了後に実施（examples/ の構造確認のため）

データ基盤の起点。ほぼすべての ETL が S3 を経由する。

#### 実装内容

```favnir
import runes/s3

stage DownloadCsv: String -> List<Row> !Io = |key| {
  bind bytes <- S3.get_object(config.s3.bucket, key)
  bind rows  <- Csv.decode[Row](bytes)
  Result.ok(rows)
}

stage UploadParquet: List<Row> -> Unit !Io = |rows| {
  bind bytes <- Parquet.encode(rows)
  bind _     <- S3.put_object(config.s3.bucket, "output/result.parquet", bytes)
  Result.ok(unit)
}
```

#### 実装する関数

| 関数 | 内容 |
|---|---|
| `S3.get_object(bucket, key)` | オブジェクト取得（Bytes 返却） |
| `S3.put_object(bucket, key, bytes)` | オブジェクト書き込み |
| `S3.list_objects(bucket, prefix)` | プレフィックス一覧 |
| `S3.delete_object(bucket, key)` | 削除 |
| `S3.presign_url(bucket, key, ttl)` | 署名付き URL 生成 |
| `S3.stream_get(bucket, key)` | ストリーミング取得（大容量対応） |

#### 完了条件

- LocalStack（Docker）で全関数が動作
- `cargo test` でモックテスト 5 件以上 PASS
- `fav run examples/s3_csv_to_parquet.fav` が動く

#### 検証コマンド

```bash
cargo test s3
docker compose up localstack -d && fav run examples/s3_csv_to_parquet.fav
```

---

### v25.3 — redis Rune 実質化

**依存関係**: v25.1 と並行可能（外部サービス依存なし）

キャッシュ・セッション・レート制限・Pub/Sub のハブ。

#### 実装する関数

| 関数 | 内容 |
|---|---|
| `Redis.get[T](conn, key)` | 型付き GET（JSON デシリアライズ） |
| `Redis.set(conn, key, value, ttl)` | SET with TTL |
| `Redis.del(conn, key)` | DELETE |
| `Redis.incr(conn, key)` | INCR（カウンタ・レート制限） |
| `Redis.lpush / rpop(conn, key, value)` | キュー操作 |
| `Redis.publish(conn, channel, msg)` | Pub/Sub 送信 |
| `Redis.subscribe(conn, channel, fn)` | Pub/Sub 受信 |

#### 完了条件

- `runes/redis/` の全関数が Rust バックエンドで実装済み
- `cargo test` で MockRedis を使ったテスト 5 件以上 PASS

#### 検証コマンド

```bash
cargo test redis
```

---

### v25.4 — mysql Rune 実質化

**依存関係**: v25.1 完了（DbConn interface を共通化するため）

企業 DB の定番。postgres との API 統一感を保つ。

#### 実装方針

postgres Rune と同一の API インターフェース（`connect / query[T] / execute / transaction`）を
MySQL 向けに実装する。`interface DbConn` に対して `impl` することで、
DB 依存を注入パターンで切り替え可能にする。

#### 完了条件

- `runes/mysql/` の 4 関数（connect / query / execute / transaction）が実装済み
- `cargo test` でモックテスト 4 件以上 PASS
- postgres の既存テストとシグネチャの一貫性確認

#### 検証コマンド

```bash
cargo test mysql
```

---

### v25.5 — mongodb Rune 実質化

**依存関係**: なし（postgres / mysql と独立した API 体系）

ドキュメント系 NoSQL の代表。JSON との親和性が高い。

#### 実装する関数

| 関数 | 内容 |
|---|---|
| `Mongo.find[T](coll, filter)` | 型付き find（BSON → T） |
| `Mongo.insert_one(coll, doc)` | ドキュメント挿入 |
| `Mongo.insert_many(coll, docs)` | バッチ挿入 |
| `Mongo.update_one(coll, filter, update)` | 更新 |
| `Mongo.delete_one(coll, filter)` | 削除 |
| `Mongo.aggregate[T](coll, pipeline)` | 集計パイプライン |

#### 完了条件

- `runes/mongodb/` の 6 関数が実装済み
- `cargo test` でモックテスト 5 件以上 PASS

#### 検証コマンド

```bash
cargo test mongodb
```

---

### v25.6 — dynamodb Rune 実質化

**依存関係**: v25.2（s3 Rune）完了後推奨（LocalStack の AWS 認証基盤を共有）

AWS ユーザーの KV / NoSQL の中心。

#### 実装する関数

| 関数 | 内容 |
|---|---|
| `DynamoDB.get_item[T](table, key)` | 型付き GetItem |
| `DynamoDB.put_item(table, item)` | PutItem |
| `DynamoDB.delete_item(table, key)` | DeleteItem |
| `DynamoDB.query[T](table, condition)` | 型付き Query |
| `DynamoDB.scan[T](table, filter)` | Scan（全件） |
| `DynamoDB.batch_write(table, items)` | BatchWriteItem |

#### 完了条件

- `runes/dynamodb/` の 6 関数が実装済み
- LocalStack で全関数が動作
- `cargo test` でモックテスト 5 件以上 PASS

#### 検証コマンド

```bash
cargo test dynamodb
docker compose up localstack -d && fav run examples/dynamodb_etl.fav
```

---

### v25.7 — kafka Rune 実質化

**依存関係**: なし（ストリーミング Rune の先行）

ストリーミングパイプラインの中核。

#### 実装する関数

| 関数 | 内容 |
|---|---|
| `Kafka.produce(topic, key, value)` | メッセージ送信（非同期） |
| `Kafka.consume[T](topic, group_id, fn)` | 型付き consumer ループ |
| `Kafka.consume_batch[T](topic, group_id, size, fn)` | バッチ消費 |
| `Kafka.commit(consumer)` | オフセットコミット |
| `Kafka.seek(consumer, partition, offset)` | オフセット指定 |

#### 完了条件

- `runes/kafka/` の 5 関数が実装済み
- Docker Kafka（Redpanda）でモックテスト 5 件以上 PASS

#### 検証コマンド

```bash
cargo test kafka
```

---

### v25.8 — elasticsearch Rune 実質化

**依存関係**: なし（v29.1 の Rune Registry 検索基盤でも再利用）

全文検索・ログ分析・ベクトル検索（8.x+）。

#### 実装する関数

| 関数 | 内容 |
|---|---|
| `ES.index(index, doc)` | ドキュメントインデックス |
| `ES.search[T](index, query)` | 型付き検索（hits → T） |
| `ES.bulk(index, docs)` | バルクインデックス |
| `ES.delete(index, id)` | ドキュメント削除 |
| `ES.knn_search[T](index, vector, k)` | ベクトル近傍検索 |

#### 完了条件

- `runes/elasticsearch/` の 5 関数が実装済み
- `cargo test` でモックテスト 5 件以上 PASS

#### 検証コマンド

```bash
cargo test elasticsearch
```

---

### v25.9 — vm.fav Phase 6（CallFn 実装）

**依存関係**: なし（Rune 実質化と独立。並行実施可能）

ユーザー定義関数のディスパッチを vm.fav 内で完結させる。

#### 実装内容

```favnir
// vm.fav に追加する opcode ハンドラ
stage exec_call_fn: (VmState, Instruction) -> VmState = |vm, instr| {
  match instr {
    CallFn(fn_idx, argc) => {
      bind args <- collect_args(vm.stack, argc)
      bind fn   <- resolve_fn(vm.program, fn_idx)
      let frame = Frame { fn: fn, locals: args, pc: 0 }
      Result.ok(vm |> push_frame(frame))
    }
    _ => exec_other(vm, instr)
  }
}
```

#### 完了条件

- `fav run --vm=self/vm.fav self/compiler.fav -- hello.fav` が動く
- `fav run --vm=self/vm.fav self/checker.fav -- tests/bootstrap/hello.fav` が動く
- 4-stage bootstrap 全 6 fixture（bytecode_A == bytecode_B == bytecode_C）が PASS

#### 検証コマンド

```bash
cargo test -- --ignored bootstrap
fav run --vm=self/vm.fav self/compiler.fav -- tests/bootstrap/hello.fav
```

---

### v26.0 — Rune Foundation マイルストーン宣言

**完了条件:**

| コンポーネント | 実装 |
|---|---|
| postgres Rune（5 関数以上） | 完全実装 ✓ |
| s3 Rune（6 関数以上） | 完全実装 ✓ |
| redis Rune（7 関数以上） | 完全実装 ✓ |
| mysql Rune（4 関数以上） | 完全実装 ✓ |
| mongodb Rune（6 関数以上） | 完全実装 ✓ |
| dynamodb Rune（6 関数以上） | 完全実装 ✓ |
| kafka Rune（5 関数以上） | 完全実装 ✓ |
| elasticsearch Rune（5 関数以上） | 完全実装 ✓ |
| vm.fav Phase 6（CallFn） | 完全実装 ✓ |

> 「`fav run examples/full_etl.fav`（postgres → 集計 → s3 → kafka 通知）が動く」
> = Rune Foundation の完成を象徴するデモ

---

---

## v27.0 — Streaming Native

**テーマ**: 「ストリームが型安全に流れる」
**期間**: v26.1〜v26.9 → v27.0 マイルストーン宣言

### なぜ Rune Foundation の次か

コア Rune が動いた後、次の壁は「**大量データをリアルタイムに処理できない**」だ。
バッチ ETL は v26.0 で実現できるが、Kafka → 変換 → DB のストリーミングパイプラインは
まだ「ポーリング + バッチ」の組み合わせに過ぎない。
`stage` と `seq` というプリミティブをストリーミングに対応させ、
**型安全なリアルタイムパイプライン**を実現する。

### 対象ユーザーへの価値

- Kafka / Kinesis からのイベントを型安全に処理できる
- バックプレッシャー制御が言語レベルで保証される
- 「ストリームとバッチを同じ `seq` で書く」統一モデルが完成する

### やらないこと（スコープ外）

- CEP（複合イベント処理）/ Flink 相当の高度なウィンドウ結合
- ストリームの永続化・リプレイ基盤（→ Data Lakehouse）
- マルチリージョンレプリケーション
- ストリームのスキーマレジストリ（Confluent Schema Registry 相当）

---

### v26.1 — kinesis Rune 実質化

**依存関係**: v25.7（kafka Rune）完了後推奨（API 設計の一貫性のため）

AWS ストリーミングの中心。Kafka Rune との API 統一感を保つ。

#### 実装する関数

| 関数 | 内容 |
|---|---|
| `Kinesis.put_record(stream, key, data)` | レコード送信 |
| `Kinesis.put_records(stream, records)` | バッチ送信（最大 500 件） |
| `Kinesis.get_records[T](shard, iterator)` | 型付き取得 |
| `Kinesis.get_shard_iterator(stream, shard, type)` | イテレータ取得 |
| `Kinesis.consume[T](stream, group, fn)` | 継続消費ループ |

#### 完了条件

- `runes/kinesis/` の 5 関数が実装済み
- LocalStack で全関数が動作・テスト 5 件以上 PASS

#### 検証コマンド

```bash
cargo test kinesis
```

---

### v26.2 — nats Rune 実質化

**依存関係**: v26.1 と並行可能

軽量・高速メッセージング。IoT / マイクロサービスの定番。

#### 実装する関数

| 関数 | 内容 |
|---|---|
| `NATS.publish(subject, payload)` | メッセージ発行 |
| `NATS.subscribe[T](subject, fn)` | 型付き購読 |
| `NATS.request[T](subject, payload, timeout)` | リクエスト/レスポンス |
| `NATS.jetstream_publish(stream, payload)` | JetStream 永続メッセージ |
| `NATS.jetstream_consume[T](stream, fn)` | JetStream 消費 |

#### 完了条件

- `runes/nats/` の 5 関数が実装済み・テスト 5 件以上 PASS

#### 検証コマンド

```bash
cargo test nats
```

---

### v26.3 — rabbitmq Rune 実質化

**依存関係**: v26.2 と並行可能

エンタープライズ MQ の定番。AMQP プロトコル。

#### 実装する関数

| 関数 | 内容 |
|---|---|
| `RabbitMQ.publish(exchange, routing_key, msg)` | メッセージ発行 |
| `RabbitMQ.consume[T](queue, fn)` | 型付き消費 |
| `RabbitMQ.ack(delivery)` | 配信確認 |
| `RabbitMQ.nack(delivery, requeue)` | 否定確認（再キュー） |
| `RabbitMQ.declare_queue(name, options)` | キュー宣言 |

#### 完了条件

- `runes/rabbitmq/` の 5 関数が実装済み・テスト 4 件以上 PASS

#### 検証コマンド

```bash
cargo test rabbitmq
```

---

### v26.4 — `#[streaming]` stage の改善

**依存関係**: v26.1〜v26.3 完了後（実際の Rune とのインテグレーション検証のため）

現在の `#[streaming]` は chunk_size ベースの単純ポーリング。
真のストリーミング対応に拡張する。

#### 拡張内容

```favnir
// バックプレッシャー対応ストリーミング stage
#[streaming(chunk_size: 1000, backpressure: true)]
stage TransformLogs: Stream<RawLog> -> Stream<ParsedLog> !Io = |stream| {
  Stream.map(stream, |log| {
    ParsedLog {
      timestamp: DateTime.parse(log.ts),
      level:     LogLevel.from_str(log.level),
      message:   log.msg
    }
  })
}
```

#### 実装する操作

| 操作 | 内容 |
|---|---|
| `Stream.map[A, B](stream, fn)` | 変換 |
| `Stream.filter[T](stream, predicate)` | フィルタ |
| `Stream.flat_map[A, B](stream, fn)` | 展開 + 変換 |
| `Stream.window(stream, size, fn)` | タンブリングウィンドウ |
| `Stream.merge(streams)` | 複数ストリームの合流 |
| `Stream.split(stream, predicate)` | 条件分岐ルーティング |

#### 完了条件

- `Stream.*` 操作 6 関数が実装・テスト 6 件以上 PASS
- `#[streaming(backpressure: true)]` が VM で機能する

#### 検証コマンド

```bash
cargo test streaming
```

---

### v26.5〜v26.7 — ストリーミング E2E デモ整備

**依存関係**: v26.1〜v26.4 完了後

```favnir
// kafka → 変換 → elasticsearch のリアルタイムログ集計
seq LogPipeline =
  Kafka.consume[RawLog]("logs", "favnir-consumer")
  |> ParseLog
  |> EnrichWithGeo
  |> ES.index("logs-index")
```

3 つのデモパイプラインを `examples/streaming/` に追加:
1. `kafka_to_elasticsearch.fav` — ログ集計
2. `kinesis_to_s3.fav` — イベントアーカイブ
3. `nats_to_postgres.fav` — IoT センサーデータ収集

#### 完了条件（3 本まとめて）

- 各デモが Docker Compose 環境で `fav run` できる
- `examples/streaming/README.md` に実行手順あり

---

### v26.8 — sqs Rune 実質化

**依存関係**: v25.2（LocalStack AWS 認証基盤）完了後

AWS の非同期処理の基礎。Lambda トリガーとの連携。

#### 完了条件

- `SQS.send / receive / delete / purge` が実装済み・テスト 4 件以上 PASS

#### 検証コマンド

```bash
cargo test sqs
```

---

### v26.9 — pulsar Rune 追加

**依存関係**: v26.3（RabbitMQ）完了後推奨（AMQP 系の設計参照）

Kafka 代替。マルチテナント・マルチクラスタ対応。

#### 完了条件

- `Pulsar.produce / consume / ack` が実装済み・テスト 3 件以上 PASS

#### 検証コマンド

```bash
cargo test pulsar
```

---

### v27.0 — Streaming Native マイルストーン宣言

**完了条件:**

| コンポーネント | 実装 |
|---|---|
| kinesis / nats / rabbitmq / sqs Rune | 完全実装 ✓ |
| pulsar Rune | 完全実装 ✓ |
| `#[streaming]` バックプレッシャー対応 | 実装 ✓ |
| `Stream.*` 操作（6 関数以上） | 実装 ✓ |
| ストリーミング E2E デモ 3 本 | 動作確認 ✓ |

> 「Kafka → 変換 → Elasticsearch のリアルタイムパイプラインが 50 行で書ける」
> = Streaming Native の完成を象徴するデモ

---

---

## v28.0 — Data Lakehouse

**テーマ**: 「現代のデータ基盤に溶け込む」
**期間**: v27.1〜v27.9 → v28.0 マイルストーン宣言

### なぜ Streaming Native の次か

ストリーミングが型安全に動くようになった後、次の課題は
「**モダンなデータレイクハウスアーキテクチャ（Delta Lake / Iceberg）に対応できない**」だ。
Databricks / dbt / Spark との連携を通じて、
Favnir を「データ基盤の一部」として採用できるようにする。

### 対象ユーザーへの価値

- Delta Lake / Iceberg テーブルを Favnir から読み書きできる
- dbt モデルの出力を Favnir パイプラインで後処理できる
- Databricks / Redshift / BigQuery などの DWH と接続できる
- データレイクのスキーマ変更を型システムで検知できる

### やらないこと（スコープ外）

- Spark / Databricks ジョブの生成・実行
- Delta Lake の Vacuum / Optimize の自動スケジューリング
- BI ツール（Tableau / Looker）直接接続
- データカタログ（Glue / Dataplex）の管理 UI

---

### v27.1 — delta-lake Rune 追加

**依存関係**: v25.2（s3 Rune）完了後（Delta テーブルは S3 上に存在）

現代のデータレイクハウスの事実上の標準。

#### 実装する関数

| 関数 | 内容 |
|---|---|
| `DeltaLake.read[T](path)` | Delta テーブル読み込み（型付き） |
| `DeltaLake.write(path, data, mode)` | 書き込み（append / overwrite / merge） |
| `DeltaLake.merge[T](path, data, condition)` | MERGE（upsert） |
| `DeltaLake.history(path)` | トランザクションログ取得 |
| `DeltaLake.vacuum(path, retention)` | 古いファイル削除 |
| `DeltaLake.optimize(path)` | Zorder / コンパクション |

#### 完了条件

- `runes/delta-lake/` の 6 関数が実装済み・テスト 5 件以上 PASS
- ローカルパスで `DeltaLake.read / write` が動く

#### 検証コマンド

```bash
cargo test delta_lake
fav run examples/delta_lake_etl.fav
```

---

### v27.2 — iceberg Rune 追加

**依存関係**: v27.1（Delta Lake）と並行可能

Apache Iceberg。Snowflake / AWS Glue / Spark との親和性。

#### 実装する関数

| 関数 | 内容 |
|---|---|
| `Iceberg.read[T](catalog, table)` | テーブル読み込み |
| `Iceberg.append(catalog, table, data)` | データ追加 |
| `Iceberg.overwrite(catalog, table, data, filter)` | 条件上書き |
| `Iceberg.time_travel[T](catalog, table, snapshot_id)` | スナップショット読み込み |
| `Iceberg.schema_evolution(catalog, table, new_schema)` | スキーマ進化 |

#### 完了条件

- `runes/iceberg/` の 5 関数が実装済み・テスト 4 件以上 PASS

#### 検証コマンド

```bash
cargo test iceberg
```

---

### v27.3 — clickhouse Rune 追加

**依存関係**: v25.1（postgres）完了後（クエリ API の統一感）

分析特化の列指向 DB。ClickHouse Cloud / セルフホストに対応。

#### 実装する関数

| 関数 | 内容 |
|---|---|
| `ClickHouse.query[T](conn, sql)` | 型付きクエリ |
| `ClickHouse.insert(conn, table, rows)` | 高速バルク挿入 |
| `ClickHouse.async_insert(conn, table, rows)` | 非同期バルク挿入 |

#### 完了条件

- `runes/clickhouse/` の 3 関数が実装済み・テスト 3 件以上 PASS

#### 検証コマンド

```bash
cargo test clickhouse
```

---

### v27.4 — bigquery Rune 実質化

**依存関係**: v25.2（s3）完了後（GCS 接続基盤として参照）

Google BigQuery。v15.2 の部分実装を完全実装に昇格。

#### 完了条件

- `BigQuery.query / insert / load_from_gcs` が実装済み・テスト 4 件以上 PASS

#### 検証コマンド

```bash
cargo test bigquery
```

---

### v27.5 — redshift Rune 追加

**依存関係**: v25.1（postgres）完了後（Redshift は postgres 互換 API）

AWS の分析 DWH。COPY コマンドによる高速ロードに対応。

#### 完了条件

- `Redshift.query / copy_from_s3 / unload_to_s3` が実装済み・テスト 3 件以上 PASS

#### 検証コマンド

```bash
cargo test redshift
```

---

### v27.6 — jsonl Rune 追加

**依存関係**: なし

JSON Lines。LLM 学習データ・ログ処理の現代的な標準フォーマット。

#### 実装する関数

| 関数 | 内容 |
|---|---|
| `JSONL.read[T](path)` | 型付き行読み込み（ストリーミング） |
| `JSONL.write(path, rows)` | 書き込み |
| `JSONL.stream[T](path, fn)` | 大容量ストリーミング処理 |

#### 完了条件

- `runes/jsonl/` の 3 関数が実装済み・テスト 3 件以上 PASS

#### 検証コマンド

```bash
cargo test jsonl
```

---

### v27.7 — `fav infer --from delta` / `--from iceberg`

**依存関係**: v27.1（delta-lake）/ v27.2（iceberg）完了後

型推論を Delta Lake / Iceberg のスキーマから自動生成する。

```bash
fav infer --from delta --path s3://my-bucket/my-delta-table
# → type Row = { id: Int, name: String, amount: Float, created_at: DateTime }
```

#### 完了条件

- `fav infer --from delta/iceberg` が型定義 Favnir コードを標準出力に出力

#### 検証コマンド

```bash
fav infer --from delta --path /tmp/test_table
```

---

### v27.8 — dbt 連携

**依存関係**: v27.4（bigquery）または v25.1（postgres）いずれか完了後

dbt モデルの出力を Favnir パイプラインで参照できる。

```favnir
import runes/dbt

stage LoadDbtModel: Unit -> List<CustomerSummary> !Db = |_| {
  bind result <- Dbt.ref[CustomerSummary](config.dbt, "customer_summary")
  Result.ok(result)
}
```

#### 完了条件

- `Dbt.ref[T]` が dbt manifest.json を解析して SQL 実行し型付き結果を返す
- テスト 3 件以上 PASS

#### 検証コマンド

```bash
cargo test dbt
```

---

### v27.9 — sqlite Rune 追加

**依存関係**: なし（依存ゼロ・ローカル動作）

ローカル開発・軽量 ETL・テスト用。依存なしで動く組み込み DB。

#### 完了条件

- `SQLite.open / query / execute / close` が実装済み・テスト 4 件以上 PASS

#### 検証コマンド

```bash
cargo test sqlite
```

---

### v28.0 — Data Lakehouse マイルストーン宣言

**完了条件:**

| コンポーネント | 実装 |
|---|---|
| delta-lake / iceberg Rune | 完全実装 ✓ |
| clickhouse / bigquery / redshift Rune | 完全実装 ✓ |
| jsonl / sqlite Rune | 完全実装 ✓ |
| `fav infer --from delta/iceberg` | 実装 ✓ |
| dbt 連携 Rune | 実装 ✓ |

> 「Delta Lake テーブルを Favnir から型安全に読み書きし、
>  dbt モデルの結果を次のステージに渡す」
> = Data Lakehouse の完成を象徴するデモ

---

---

## v29.0 — Observability First

**テーマ**: 「パイプラインの内側が見える」
**期間**: v28.1〜v28.9 → v29.0 マイルストーン宣言

### なぜ Data Lakehouse の次か

データが正しく流れるようになった後、次の課題は
「**パイプラインが失敗した理由がわからない**」「**どこが遅いかわからない**」だ。
Favnir のエフェクトシステムは可観測性と相性が良い：
エフェクト境界でメトリクス・トレース・ログを自動収集できる。
「型安全なデータパイプライン」に「型安全なオブザーバビリティ」を加える。

### 対象ユーザーへの価値

- パイプラインのどの stage が遅いかを自動計測できる
- エラーが発生した stage と入力データを Sentry / Datadog で確認できる
- `#[track]` アノテーションで SLO 監視を宣言的に書ける

### やらないこと（スコープ外）

- OpsGenie / VictorOps 等アラート管理ツールの統合（→ コミュニティ Rune）
- ログの自動解析・異常検知（ML ベース）
- テスト自動生成（カバレッジ保証）
- コスト分析・クラウド費用の最適化提案

---

### v28.1 — prometheus Rune 追加

**依存関係**: なし

メトリクス収集の標準。`#[track]` アノテーションとの統合。

#### 実装する関数 + アノテーション

```favnir
import runes/prometheus

// stage の実行時間・成功率を自動収集
#[track(latency: true, error_rate: true)]
stage TransformRows: List<Row> -> List<Result> !Io = |rows| {
  // ...
}

// カスタムメトリクス
stage SendMetric: StageSummary -> Unit !Io = |summary| {
  bind _ <- Prometheus.counter("rows_processed_total", summary.count)
  bind _ <- Prometheus.histogram("stage_latency_ms", summary.duration_ms)
  Result.ok(unit)
}
```

#### 完了条件

- `Prometheus.counter / gauge / histogram` が実装済み
- `#[track]` アノテーションが stage の前後でメトリクスを自動送信
- テスト 4 件以上 PASS

#### 検証コマンド

```bash
cargo test prometheus
```

---

### v28.2 — datadog Rune 追加

**依存関係**: v28.1 と並行可能（API 設計の参照のみ）

APM・ログ・メトリクスの統合プラットフォーム。

#### 実装する関数

| 関数 | 内容 |
|---|---|
| `Datadog.metric(name, value, tags)` | カスタムメトリクス送信 |
| `Datadog.log(level, message, attrs)` | 構造化ログ送信 |
| `Datadog.trace(name, fn)` | APM トレース |
| `Datadog.event(title, text, tags)` | イベント通知 |

#### 完了条件

- `runes/datadog/` の 4 関数が実装済み・テスト 4 件以上 PASS

#### 検証コマンド

```bash
cargo test datadog
```

---

### v28.3 — OpenTelemetry 統合強化

**依存関係**: 既存 `otel.rs` が存在（Rune としての公開のみ）

既存の `otel.rs` を Rune として公開し、
Jaeger / Tempo / Honeycomb 等どのバックエンドでも使えるようにする。

```favnir
import runes/otel

#[trace(name: "load_from_db")]
stage LoadFromDb: Config -> List<Row> !Db = |config| {
  // スパンが自動的に開始・終了する
  bind conn <- Postgres.connect(config.db)
  bind rows <- Postgres.query[Row](conn, "SELECT * FROM orders")
  Result.ok(rows)
}
```

#### 完了条件

- `otel.rs` の機能が `runes/otel/` として Favnir から呼び出せる
- `#[trace]` アノテーションがスパンの自動開始・終了に対応

#### 検証コマンド

```bash
cargo test otel
```

---

### v28.4 — `fav profile` 強化

**依存関係**: v28.1（prometheus）完了後推奨（メトリクス基盤の共有）

既存の `fav profile` を stage 別のフレームグラフ出力に強化。

```bash
fav profile --format flamegraph src/pipeline.fav
# → profile.svg（インタラクティブなフレームグラフ）

fav profile --compare v26.0.0 src/pipeline.fav
# → パフォーマンス劣化した stage をハイライト
```

#### 完了条件

- `fav profile --format flamegraph` が SVG を出力
- `fav profile --compare <version>` が劣化 stage をハイライト表示

#### 検証コマンド

```bash
fav profile --format flamegraph tests/fixtures/etl.fav
```

---

### v28.5 — sentry Rune 追加

**依存関係**: なし

エラートラッキング。stage の失敗を自動レポート。

```favnir
import runes/sentry

// エラー発生時に自動的に Sentry へ送信
#[on_error(report_to: "sentry")]
stage ProcessPayment: PaymentRequest -> PaymentResult !Http = |req| {
  // ...
}
```

#### 完了条件

- `Sentry.capture_error / set_user / set_tag` が実装済み
- `#[on_error]` アノテーションが stage 失敗時に自動送信

#### 検証コマンド

```bash
cargo test sentry
```

---

### v28.6 — grafana Rune 追加

**依存関係**: v28.1（prometheus）完了後推奨

ダッシュボード更新 API。パイプライン結果をリアルタイムで可視化。

#### 完了条件

- `Grafana.create_annotation / push_dashboard` が実装済み・テスト 2 件以上 PASS

#### 検証コマンド

```bash
cargo test grafana
```

---

### v28.7〜v28.9 — オブザーバビリティ E2E デモ

**依存関係**: v28.1〜v28.6 完了後

3 つのデモ:
1. `prometheus_grafana.fav` — ETL パイプラインの Grafana ダッシュボード
2. `datadog_apm.fav` — マイクロサービス連携のトレース
3. `sentry_alerting.fav` — 失敗時の自動アラート

#### 完了条件

- 3 本とも `fav run examples/observability/*.fav` で動く

---

### v29.0 — Observability First マイルストーン宣言

**完了条件:**

| コンポーネント | 実装 |
|---|---|
| prometheus / datadog / sentry / grafana Rune | 完全実装 ✓ |
| OpenTelemetry Rune（otel.rs 昇格） | 完全実装 ✓ |
| `#[track]` / `#[trace]` / `#[on_error]` アノテーション | 実装 ✓ |
| `fav profile --format flamegraph` | 実装 ✓ |
| オブザーバビリティ E2E デモ 3 本 | 動作確認 ✓ |

> 「`#[track(latency, error_rate)]` を stage に付けるだけで
>  Grafana ダッシュボードにメトリクスが現れる」
> = Observability First の完成を象徴するデモ

---

---

## v30.0 — Ecosystem Maturity

**テーマ**: 「コミュニティが Rune を育てる」
**期間**: v29.1〜v29.9 → v30.0 マイルストーン宣言

### なぜ Observability First の次か

公式 Rune が充実し、パイプラインの可観測性が整った後、
次の課題は「**サードパーティが Rune を作れない**」「**Rune を配布できない**」だ。
Rust の `crates.io`、Python の `PyPI` に相当する
**Favnir Rune Registry** を公開し、コミュニティドリブンなエコシステムを構築する。

### 対象ユーザーへの価値

- 自作 Rune を `fav publish` で公開できる
- `fav add stripe` で Stripe 連携 Rune を一発で追加できる
- コミュニティの Rune を `fav search payment` で検索できる

### やらないこと（スコープ外）

- Rust クレートの再実装（既存の Cargo エコシステムに委ねる）
- IDE 全機能（DAP デバッガ統合・ブレークポイント等）— v30 以降
- Rune の型チェック自動生成 / AI コード補完
- マネタイズ基盤・有償プラン管理
- コミュニティフォーラム / Discord の運営（Registry 公開のみ）

---

### v29.1 — `fav publish` 実装（Rune Registry）

**依存関係**: v25.8（elasticsearch）完了後（検索基盤）

現在の `fav publish` はスタブ。実際のレジストリ基盤を構築する。

#### 仕組み

```bash
# rune.toml に名前・バージョン・依存を記述
fav publish             # Rune Registry に公開
fav add stripe          # rune.toml に依存追加 + ダウンロード
fav search payment      # キーワード検索
fav info stripe         # Rune 詳細・バージョン履歴
fav update              # 依存する Rune を最新版に更新
```

#### インフラ

```
Rune Registry:
  - API: Lambda + API Gateway（rune-registry）
  - ストレージ: S3（.fav ファイル + rune.toml）
  - 検索: Elasticsearch（v25.8 で実質化済み）
  - 認証: JWT（`fav login` で GitHub OAuth）
```

#### 完了条件

- `fav publish / add / search / info` が実際のレジストリ API を呼ぶ
- Lambda + S3 + ES のインフラが Terraform で管理されている
- E2E テスト（publish → search → add）が通る

#### 検証コマンド

```bash
fav publish --dry-run
fav search test
```

---

### v29.2 — mlflow Rune 追加

**依存関係**: なし（ML 系 Rune 最初の 1 本）

ML 実験管理・モデルレジストリ。データパイプラインと ML パイプラインの橋渡し。

#### 実装する関数

| 関数 | 内容 |
|---|---|
| `MLflow.log_metric(run_id, key, value, step)` | メトリクス記録 |
| `MLflow.log_param(run_id, key, value)` | パラメータ記録 |
| `MLflow.log_artifact(run_id, path)` | 成果物アップロード |
| `MLflow.start_run(experiment)` | 実験実行開始 |
| `MLflow.register_model(run_id, name)` | モデル登録 |
| `MLflow.load_model[T](name, version)` | モデル読み込み |

#### 完了条件

- `runes/mlflow/` の 6 関数が実装済み・テスト 5 件以上 PASS

#### 検証コマンド

```bash
cargo test mlflow
```

---

### v29.3 — pinecone Rune 追加

**依存関係**: v25.8（elasticsearch）完了後推奨（ベクトル検索 API の設計参照）

ベクトル DB。RAG パイプラインの構築に不可欠。

#### 実装する関数

| 関数 | 内容 |
|---|---|
| `Pinecone.upsert(index, vectors)` | ベクトル追加・更新 |
| `Pinecone.query[T](index, vector, k, filter)` | 近傍検索 |
| `Pinecone.delete(index, ids)` | ベクトル削除 |
| `Pinecone.fetch[T](index, ids)` | ID 指定取得 |

#### 完了条件

- `runes/pinecone/` の 4 関数が実装済み・テスト 4 件以上 PASS

#### 検証コマンド

```bash
cargo test pinecone
```

---

### v29.4 — vertex-ai / sagemaker Rune 追加

**依存関係**: v29.2（mlflow）完了後推奨（ML pipeline の統一 API 設計）

ML プラットフォームとのネイティブ連携。

```favnir
import runes/vertex-ai

stage ScoreWithModel: List<Feature> -> List<Prediction> !Http = |features| {
  bind preds <- VertexAI.predict[Prediction](
    config.vertex.endpoint,
    features |> List.map(feature_to_json)
  )
  Result.ok(preds)
}
```

#### 完了条件

- `VertexAI.predict / deploy_model` と `SageMaker.invoke / create_endpoint` が実装済み
- テスト各 3 件以上 PASS

#### 検証コマンド

```bash
cargo test vertex_ai sagemaker
```

---

### v29.5 — github Rune 追加

**依存関係**: なし（CI/CD 統合）

CI パイプラインから PR コメント・Issue 操作ができる。

```favnir
// データ品質チェックの結果を PR にコメント
stage PostQualityReport: QualityReport -> Unit !Http = |report| {
  bind _ <- GitHub.create_comment(
    config.github,
    env("GITHUB_PR_NUMBER"),
    report |> format_quality_report
  )
  Result.ok(unit)
}
```

#### 完了条件

- `GitHub.create_comment / create_issue / list_prs` が実装済み・テスト 3 件以上 PASS

#### 検証コマンド

```bash
cargo test github
```

---

### v29.6 — pagerduty Rune 追加

**依存関係**: v28.5（sentry）完了後推奨（インシデント通知の設計参照）

インシデント通知。パイプライン失敗の自動エスカレーション。

```favnir
#[on_error(escalate_to: "pagerduty", severity: "critical")]
stage CriticalLoad: Unit -> List<Order> !Db = |_| {
  // 失敗時に自動で PagerDuty アラートが作成される
}
```

#### 完了条件

- `PagerDuty.create_incident / resolve / add_note` が実装済み・テスト 2 件以上 PASS

#### 検証コマンド

```bash
cargo test pagerduty
```

---

### v29.7 — VS Code 拡張 公式リリース

**依存関係**: LSP は v9.11.0 から実装済み（LSP サーバーとしてすでに動作）

VS Code Marketplace に正式公開する。

#### 機能

```
- シンタックスハイライト（.fav ファイル）
- 型推論結果のインライン表示
- エラー / 警告のリアルタイム表示
- 補完（stage 名 / Rune 関数 / 型）
- 定義ジャンプ（F12）
- Rune ドキュメントのホバー表示
- `fav run` / `fav test` の統合ターミナル実行
```

#### 完了条件

- `extensions/vscode-favnir/` に Marketplace 公開パッケージが存在
- VS Code Marketplace で検索・インストールできる

---

### v29.8 — ドキュメントサイト v3

**依存関係**: v24.7 ドキュメントサイト v2 の土台あり

v24.7 で作ったドキュメントサイトを採用フォーカスに再構築。

```
/                    ← ランディングページ（「30 分で動く」体験）
/learn/              ← インタラクティブチュートリアル
/cookbook/           ← 実用レシピ（30 本以上）
/runes/              ← Rune ドキュメント（全実装済み Rune）
/playground/         ← ブラウザ内実行
/community/          ← Discord / GitHub Discussions リンク
```

#### 完了条件

- 上記 6 ページがすべて存在し公開済み
- cookbook が 30 本以上

---

### v29.9 — コミュニティ Rune コンテスト / ドネーション

**依存関係**: v29.1（Rune Registry）完了後

Rune Registry の普及を促進するプログラム:
- 「Stripe Rune」「Twilio Rune」「Linear Rune」等のコミュニティ実装を募集
- 採用された Rune は公式カタログに掲載

#### 完了条件

- コンテスト告知ページが公開済み
- コミュニティ投稿 Rune が Registry に 1 本以上存在

---

### v30.0 — Ecosystem Maturity マイルストーン宣言

**完了条件:**

| コンポーネント | 実装 |
|---|---|
| `fav publish / add / search / info / update` が実際に動く | ✓ |
| Rune Registry（Lambda + S3 + ES） | 稼働 ✓ |
| mlflow / pinecone / vertex-ai / sagemaker Rune | 完全実装 ✓ |
| github / pagerduty Rune | 完全実装 ✓ |
| VS Code 拡張 Marketplace 公開 | ✓ |
| ドキュメントサイト v3 公開 | ✓ |
| コミュニティ Rune 10 本以上 | ✓ |

> 「`fav add stripe` で Stripe 連携が 5 分で動く」
> = Ecosystem Maturity の完成を象徴するデモ

---

---

## 全体ロードマップ一覧

| バージョン | テーマ | 内容 | 状態 |
|---|---|---|---|
| v25.0.0 | Practical Self-Hosting 宣言 | Favnir による自己記述完成 | done |
| v25.1 | postgres Rune 実質化 | connect / query / execute / transaction | planned |
| v25.2 | s3 Rune 実質化 | get / put / list / presign / stream | planned |
| v25.3 | redis Rune 実質化 | get / set / incr / pub-sub | planned |
| v25.4 | mysql Rune 実質化 | connect / query / execute / transaction | planned |
| v25.5 | mongodb Rune 実質化 | find / insert / update / aggregate | planned |
| v25.6 | dynamodb Rune 実質化 | get / put / query / scan / batch | planned |
| v25.7 | kafka Rune 実質化 | produce / consume / commit | planned |
| v25.8 | elasticsearch Rune 実質化 | index / search / bulk / knn | planned |
| v25.9 | vm.fav Phase 6（CallFn） | ユーザー定義関数ディスパッチ完成 | planned |
| **v26.0** | **Rune Foundation 宣言** | コア 8 Rune + vm.fav Phase 6 完成 | planned |
| v26.1 | kinesis Rune 実質化 | AWS ストリーミング | planned |
| v26.2 | nats Rune 実質化 | 軽量メッセージング | planned |
| v26.3 | rabbitmq Rune 実質化 | AMQP MQ | planned |
| v26.4 | #[streaming] バックプレッシャー対応 | Stream.* 操作 6 関数 | planned |
| v26.5〜v26.7 | ストリーミング E2E デモ 3 本 | kafka→ES / kinesis→s3 / nats→postgres | planned |
| v26.8 | sqs Rune 実質化 | AWS 非同期処理 | planned |
| v26.9 | pulsar Rune 追加 | Kafka 代替 | planned |
| **v27.0** | **Streaming Native 宣言** | 型安全リアルタイムパイプライン完成 | planned |
| v27.1 | delta-lake Rune 追加 | Delta テーブル読み書き | planned |
| v27.2 | iceberg Rune 追加 | Iceberg テーブル操作 | planned |
| v27.3 | clickhouse Rune 追加 | 列指向 DB | planned |
| v27.4 | bigquery Rune 実質化 | Google DWH | planned |
| v27.5 | redshift Rune 追加 | AWS DWH | planned |
| v27.6 | jsonl Rune 追加 | JSON Lines フォーマット | planned |
| v27.7 | fav infer --from delta/iceberg | スキーマから型自動生成 | planned |
| v27.8 | dbt 連携 Rune | dbt モデル参照 | planned |
| v27.9 | sqlite Rune 追加 | 組み込み DB | planned |
| **v28.0** | **Data Lakehouse 宣言** | モダンデータ基盤との統合 | planned |
| v28.1 | prometheus Rune 追加 | メトリクス + #[track] | planned |
| v28.2 | datadog Rune 追加 | APM + ログ + メトリクス | planned |
| v28.3 | OpenTelemetry Rune 強化 | otel.rs を Rune 昇格 | planned |
| v28.4 | fav profile フレームグラフ | --format flamegraph | planned |
| v28.5 | sentry Rune 追加 | エラートラッキング + #[on_error] | planned |
| v28.6 | grafana Rune 追加 | ダッシュボード更新 | planned |
| v28.7〜v28.9 | オブザーバビリティ E2E デモ 3 本 | prometheus+grafana / datadog / sentry | planned |
| **v29.0** | **Observability First 宣言** | 型安全オブザーバビリティ完成 | planned |
| v29.1 | fav publish / Rune Registry 実装 | Lambda + S3 + ES 基盤 | planned |
| v29.2 | mlflow Rune 追加 | ML 実験管理 | planned |
| v29.3 | pinecone Rune 追加 | ベクトル DB | planned |
| v29.4 | vertex-ai / sagemaker Rune 追加 | ML プラットフォーム | planned |
| v29.5 | github Rune 追加 | CI/CD 統合 | planned |
| v29.6 | pagerduty Rune 追加 | インシデント通知 | planned |
| v29.7 | VS Code 拡張 Marketplace 公開 | 公式 LSP 拡張 | planned |
| v29.8 | ドキュメントサイト v3 | 採用フォーカス再構築 | planned |
| v29.9 | コミュニティ Rune コンテスト | Registry 普及プログラム | planned |
| **v30.0** | **Ecosystem Maturity 宣言** | コミュニティドリブン完成 | planned |
