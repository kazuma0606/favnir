# v25.7.0 仕様書 — kafka Rune 実質化

## 概要

| 項目 | 内容 |
|---|---|
| バージョン | v25.7.0 |
| フェーズ | Rune Foundation（v25.1〜v26.0） |
| テーマ | kafka Rune の「動く Rune」5 条件達成 |
| 依存関係 | なし（rskafka v0.6 は既存 Cargo.toml に存在） |
| 目標テスト数 | 2021 件（v25.6.0: 2014 件 + 7 件） |

---

## 背景と目的

v25.6.0 で dynamodb Rune を実質化した。次はストリーミングパイプラインの中核である Kafka を実質化する。

既存の `runes/kafka/kafka.fav` は v15.4.0 で追加された `produce` / `consume_one` の 2 関数のみ（`KafkaConn` 型なし）。
vm.rs には `Kafka.produce_raw` / `Kafka.consume_one_raw`（v15.4.0）が存在する。

v25.7.0 では `KafkaConn` ラッパーを導入し、`connect` / `produce` / `consume_one` / `consume_batch` / `create_topic` の 5 関数を実装する。
また `Kafka.connect_raw` / `Kafka.consume_batch_raw` / `Kafka.create_topic_raw` の 3 primitives を追加する。

> **既存 primitives の互換性**: `Kafka.produce_raw(brokers, topic, key, value)` / `Kafka.consume_one_raw(brokers, topic, group_id)` は変更しない。Rune 側を KafkaConn ベースに刷新するのみ。

---

## 「動く Rune」5 条件

| # | 条件 | 対象 |
|---|---|---|
| 1 | connect | `KAFKA_BOOTSTRAP_BROKERS` 環境変数（例: `localhost:9092`）または Redpanda 経由で接続確立 |
| 2 | read | `Kafka.consume_one` / `Kafka.consume_batch` — メッセージ文字列 / JSON 配列文字列 |
| 3 | write | `Kafka.produce` / `Kafka.create_topic` — メッセージ送信 / トピック作成 |
| 4 | error | `Result<T, String>` 統一、エラーメッセージにトピック名を含む |
| 5 | test | `v257000_tests` 7 件 PASS + `examples/kafka_events_etl.fav` E2E デモ |

---

## 既存実装の現状

| ファイル | 状態 | 備考 |
|---|---|---|
| `runes/kafka/kafka.fav` | 2 関数（KafkaConn なし） | v15.4.0 で追加 |
| `Effect::Stream` | **既存** | `!Stream` として使用。E0319 エラーコード（checker.rs に存在、error_catalog.rs に未登録） |
| `Kafka.produce_raw` | 既存（v15.4.0） | `(brokers, topic, key, value)` → `Result<Unit, String>` |
| `Kafka.consume_one_raw` | 既存（v15.4.0） | `(brokers, topic, group_id)` → `Result<String, String>` |
| `rskafka = { version = "0.6" }` | 既存 Cargo.toml | `transport-tls` feature 付き |

---

## 機能仕様

### 型定義

```favnir
// ブローカーアドレス文字列ラッパー型
// "" → KAFKA_BOOTSTRAP_BROKERS 環境変数 → "localhost:9092"
type KafkaConn(String)
```

### 追加関数一覧

| 関数 | シグネチャ | 内容 |
|---|---|---|
| `Kafka.connect` | `(brokers: String) -> Result<KafkaConn, String> !Stream` | ブローカー接続確認（list_topics ping） |
| `Kafka.produce` | `(conn: KafkaConn, topic: String, key: String, value: String) -> Result<Unit, String> !Stream` | メッセージ送信 |
| `Kafka.consume_one` | `(conn: KafkaConn, topic: String, group_id: String) -> Result<String, String> !Stream` | 最新メッセージ 1 件取得 |
| `Kafka.consume_batch` | `(conn: KafkaConn, topic: String, group_id: String, max_count: Int) -> Result<String, String> !Stream` | 最大 max_count 件を JSON 配列文字列で返す |
| `Kafka.create_topic` | `(conn: KafkaConn, topic: String, partitions: Int) -> Result<Unit, String> !Stream` | トピック作成（partition 数指定） |

> **戻り値**:
> - `consume_one` は文字列（メッセージ payload）。トピックが空の場合 `Result.err("empty")`。
> - `consume_batch` は JSON 配列文字列（例: `["msg1", "msg2"]`）。最大 max_count 件の payload を返す。

---

## エラーコード追加仕様（E0319 — error_catalog.rs への登録）

`Effect::Stream` / E0319 は v15.4.0 から checker.rs に存在するが、`error_catalog.rs` に未登録。v25.7.0 で登録する。

| コード | 名前 | 説明 |
|---|---|---|
| E0319 | UndeclaredStreamEffect | `!Stream` エフェクトなしで Kafka 系 Rune を呼び出した場合 |

---

## Kafka クライアント実装方針

- `rskafka = { version = "0.6", features = ["transport-tls"] }` を再利用（追加 crate なし）
- ブローカー解決ロジック:
  1. `KafkaConn` の文字列が空 → `KAFKA_BOOTSTRAP_BROKERS` 環境変数 → `"localhost:9092"`
  2. それ以外 → `KafkaConn` の文字列をそのまま使用
- SASL 認証: `KAFKA_SASL_USERNAME` / `KAFKA_SASL_PASSWORD` 環境変数（省略可）
- 全 primitive は `tokio::runtime::Builder::new_current_thread().enable_all().build()?.block_on(async { ... })` で非同期を同期化
- `cfg(not(target_arch = "wasm32"))` ガードを全 Kafka 新規 primitive に付与

### rskafka v0.6 API 使用箇所

| 操作 | rskafka API |
|---|---|
| 接続確認（ping） | `client.list_topics().await` |
| メッセージ送信 | 既存 `kafka_produce_sync` ヘルパー再利用 |
| 1 件消費 | 既存 `kafka_consume_one_sync` ヘルパー再利用 |
| バッチ消費 | `partition_client.fetch_records(start, bytes_range, wait_ms).await` |
| トピック作成 | `client.controller_client()?.create_topic(name, partitions, 1_i16, 5_000).await`（`controller_client()` は同期関数 — `.await` 不要） |

### VM primitives 一覧（新規 3 件 + 既存 2 件）

| primitive 名 | 引数 | 戻り値 |
|---|---|---|
| `Kafka.connect_raw` | `brokers: String` | `Result<String, String>`（KafkaConn ラッパー） |
| `Kafka.produce_raw` | `brokers: String, topic: String, key: String, value: String` | `Result<Unit, String>` | ← 既存 |
| `Kafka.consume_one_raw` | `brokers: String, topic: String, group_id: String` | `Result<String, String>` | ← 既存 |
| `Kafka.consume_batch_raw` | `brokers: String, topic: String, max_count: Int` | `Result<String, String>`（JSON 配列） |
| `Kafka.create_topic_raw` | `brokers: String, topic: String, partitions: Int` | `Result<Unit, String>` |

> **connect_raw の戻り型**（checker レベル）: `Result<String, String>`。
> `KafkaConn(String)` は名目型ラッパーであり checker は String として扱う（DynamoConn / MongoConn と同パターン）。

---

## `examples/kafka_events_etl.fav`

```favnir
import rune "kafka"

// ── Kafka を使ったイベント ETL デモ (v25.7.0) ─────────────────────────────
// 前提: docker run -p 9092:9092 redpandadata/redpanda:latest \
//           redpanda start --overprovisioned --node-id 0 \
//           --kafka-addr 0.0.0.0:9092 --advertise-kafka-addr localhost:9092
// 実行: fav run examples/kafka_events_etl.fav

stage PublishEvent: String -> Result<Unit, String> !Stream = |event_json| {
    bind conn <- Kafka.connect("localhost:9092")
    bind _    <- Kafka.create_topic(conn, "events", 1)
    Kafka.produce(conn, "events", "event-key", event_json)
}

stage ConsumeEvents: Unit -> Result<String, String> !Stream = |_| {
    bind conn <- Kafka.connect("localhost:9092")
    Kafka.consume_batch(conn, "events", "etl-group", 10)
}

seq EventsETL = PublishEvent |> ConsumeEvents
```

---

## やらないこと（スコープ外）

- Consumer Group オフセット管理（`group_id` は引数として受け取るが、rskafka v0.6 はネイティブ Consumer Group を未サポート。オフセットは Latest から fetch）
- スキーマレジストリ（Avro / Protobuf デシリアライズ）
- Exactly-Once Semantics（EOS / transactional producer）
- 複数 partition のラウンドロビン送信（partition 0 固定）
- `Kafka.seek` / `Kafka.commit`（rskafka v0.6 では直接的な consumer group API がないため延期）

> **ロードマップとの差分**: ロードマップには `consume[T](topic, group_id, fn)` や `seek` / `commit` が記載されているが、rskafka v0.6 の低レベル API では Consumer Group の完全実装は次フェーズ（v26.x）に延期する。`consume_one` / `consume_batch` で Latest オフセットから取得する形で「read 条件」を達成する。

---

## 完了条件

| # | 条件 |
|---|---|
| 1 | `Kafka.connect` が `runes/kafka/kafka.fav` に実装済み |
| 2 | `Kafka.consume_one` / `Kafka.consume_batch` が実装済み（read 系） |
| 3 | `Kafka.produce` / `Kafka.create_topic` が実装済み（write 系） |
| 4 | `Kafka.connect_raw` / `Kafka.consume_batch_raw` / `Kafka.create_topic_raw` が `fav/src/backend/vm.rs` に存在する |
| 5 | E0319 が `fav/src/error_catalog.rs` に存在する |
| 6 | `examples/kafka_events_etl.fav` が存在し `import rune "kafka"` + `produce` + `consume_batch` を含む |
| 7 | `CHANGELOG.md` に `[v25.7.0]` エントリが存在する |
| 8 | `site/content/docs/runes/kafka.mdx` に新規 API が記載済み |
| 9 | `cargo test v257000` で 7 件すべて PASS |
| 10 | 総テスト数 ≥ 2021 件 |

---

## 設計判断

### KafkaConn(String) の checker 互換性

`connect_raw` は `Result<String, String>` を返す。Rune の `connect` は `Result<KafkaConn, String>` を返すが、`KafkaConn(String)` は名目型ラッパーであり checker 内で `String` として扱われるため `fav check` はエラーにならない（v25.6.0 の DynamoConn / v25.5.0 の MongoConn と同パターン）。

### Consumer Group オフセット管理の制約

rskafka v0.6 は Kafka プロトコルの低レベル実装であり、`JoinGroup` / `SyncGroup` / `OffsetFetch` 等の Consumer Group プロトコルは未対応。`consume_one` / `consume_batch` は Latest オフセットから fetch する（= 最新 N 件を毎回取得）。本番用途では `commit` / `seek` が必要だが v25.7.0 のスコープ外とする。

### group_id 引数の設計

`consume_one(conn, topic, group_id)` / `consume_batch(conn, topic, group_id, max_count)` の `group_id` は引数として受け取るが、rskafka v0.6 では Consumer Group API が未実装のため現時点では使用しない（将来の v26.x 実装時に利用）。`group_id` を引数に含めることで API 互換性を確保する。

### connect のたびに list_topics ping

`Kafka.connect` は呼び出しのたびに `list_topics()` を実行する（= ブローカー接続確認）。stage 内で毎回 connect を呼ぶと API 呼び出しが 2 倍になる。コネクションプールは v26.x で対応予定（vm.rs に TODO コメント追記）。

---

## 検証コマンド

```bash
cd fav && cargo test v257000 -- --test-threads=1
cd fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -5
```
