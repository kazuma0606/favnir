# v25.7.0 実装計画 — kafka Rune 実質化

## 実装順序

### Phase 1: Cargo.toml バンプ（T0）

```toml
[package]
version = "25.7.0"
# rskafka = { version = "0.6", features = ["transport-tls"] } は既存 — 追加 crate 不要
```

---

### Phase 2: E0319 を error_catalog.rs に追加（T1）

`Effect::Stream` / E0319 は v15.4.0 から checker.rs に存在するが `error_catalog.rs` に未登録。

```rust
ErrorEntry {
    code: "E0319",
    title: "undeclared !Stream effect",
    category: "effects",
    description: "A Kafka/stream operation was used in a function that does not declare `!Stream`.",
    example: "fn run(topic: String) -> Result<Unit, String> {\n    Kafka.produce_raw(brokers, topic, \"k\", \"v\")  // E0319: !Stream not declared\n}",
    fix: "Add `!Stream` to the function signature: `fn run(topic: String) -> Result<Unit, String> !Stream`.",
},
```

E0319 の挿入位置: `error_catalog.rs` 内の E0315 エントリの**直後**（E0320 の直前）に挿入する（コード番号順）。

---

### Phase 3: checker.rs 更新（T2）

`("Kafka", "connect_raw")` / `("Kafka", "consume_batch_raw")` / `("Kafka", "create_topic_raw")` の型情報を追加。

既存の `("Kafka", "produce_raw")` / `("Kafka", "consume_one_raw")` のブロック末尾に追加:

```rust
("Kafka", "connect_raw") => {
    self.require_stream_effect(span);
    // KafkaConn(String) は名目型ラッパー — checker は String として扱う（DynamoConn と同パターン）
    Some(Type::Result(Box::new(Type::String), Box::new(Type::String)))
}
("Kafka", "consume_batch_raw") => {
    self.require_stream_effect(span);
    // 戻り値は JSON 配列文字列（payload の Vec<String> を JSON エンコード）
    Some(Type::Result(Box::new(Type::String), Box::new(Type::String)))
}
("Kafka", "create_topic_raw") => {
    self.require_stream_effect(span);
    Some(Type::Result(Box::new(Type::Unit), Box::new(Type::String)))
}
```

---

### Phase 4: vm.rs 更新（T3）

#### 新規ヘルパー関数（既存 Kafka helpers ブロックの末尾に追加）

```rust
/// Kafka ブローカーへの接続確認（list_topics ping）。
fn kafka_connect_sync(brokers: &str) -> Result<(), String> {
    use rskafka::client::ClientBuilder;
    // TODO(v26.x): コネクションプール（現在は毎回接続確立）
    let addrs = kafka_broker_list(brokers);
    if addrs.is_empty() {
        return Err("Kafka.connect_raw: no brokers specified".to_string());
    }
    let username = std::env::var("KAFKA_SASL_USERNAME").ok();
    let password = std::env::var("KAFKA_SASL_PASSWORD").ok();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all().build()
        .map_err(|e| format!("Kafka.connect_raw: tokio: {e}"))?;
    rt.block_on(async {
        let mut builder = ClientBuilder::new(addrs);
        if let (Some(user), Some(pass)) = (username, password) {
            builder = builder.sasl_config(
                rskafka::client::SaslConfig::ScramSha512(
                    rskafka::client::Credentials::new(user, pass)
                )
            );
        }
        let client = builder.build().await
            .map_err(|e| format!("Kafka.connect_raw: connect: {e}"))?;
        client.list_topics().await
            .map_err(|e| format!("Kafka.connect_raw: list_topics: {e}"))?;
        Ok(())
    })
}

/// Kafka topic から最大 max_count 件のメッセージを JSON 配列文字列で返す。
fn kafka_consume_batch_sync(brokers: &str, topic: &str, max_count: i64) -> Result<String, String> {
    use rskafka::client::ClientBuilder;
    use rskafka::client::partition::{OffsetAt, UnknownTopicHandling};
    let addrs = kafka_broker_list(brokers);
    if addrs.is_empty() {
        return Err("Kafka.consume_batch_raw: no brokers specified".to_string());
    }
    let username = std::env::var("KAFKA_SASL_USERNAME").ok();
    let password = std::env::var("KAFKA_SASL_PASSWORD").ok();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all().build()
        .map_err(|e| format!("Kafka.consume_batch_raw: tokio: {e}"))?;
    rt.block_on(async {
        let mut builder = ClientBuilder::new(addrs);
        if let (Some(user), Some(pass)) = (username, password) {
            builder = builder.sasl_config(
                rskafka::client::SaslConfig::ScramSha512(
                    rskafka::client::Credentials::new(user, pass)
                )
            );
        }
        let client = builder.build().await
            .map_err(|e| format!("Kafka.consume_batch_raw: connect: {e}"))?;
        let partition_client = client
            .partition_client(topic, 0, UnknownTopicHandling::Retry)
            .await
            .map_err(|e| format!("Kafka.consume_batch_raw: partition: {e}"))?;
        let latest_offset = partition_client
            .get_offset(OffsetAt::Latest).await
            .map_err(|e| format!("Kafka.consume_batch_raw: offset: {e}"))?;
        if latest_offset == 0 {
            return Ok("[]".to_string());
        }
        let count = max_count.max(1);
        let start_offset = (latest_offset - count).max(0);
        let (records, _) = partition_client
            .fetch_records(start_offset, 1..1_048_576, 5_000).await
            .map_err(|e| format!("Kafka.consume_batch_raw: fetch: {e}"))?;
        let payloads: Vec<serde_json::Value> = records.into_iter()
            .take(count as usize)
            .map(|r| {
                let bytes = r.record.value.unwrap_or_default();
                let s = String::from_utf8_lossy(&bytes).to_string();
                serde_json::Value::String(s)
            })
            .collect();
        serde_json::to_string(&payloads)
            .map_err(|e| format!("Kafka.consume_batch_raw: serialize: {e}"))
    })
}

/// Kafka トピックを作成する（partition 数指定）。
fn kafka_create_topic_sync(brokers: &str, topic: &str, partitions: i32) -> Result<(), String> {
    use rskafka::client::ClientBuilder;
    let addrs = kafka_broker_list(brokers);
    if addrs.is_empty() {
        return Err("Kafka.create_topic_raw: no brokers specified".to_string());
    }
    let username = std::env::var("KAFKA_SASL_USERNAME").ok();
    let password = std::env::var("KAFKA_SASL_PASSWORD").ok();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all().build()
        .map_err(|e| format!("Kafka.create_topic_raw: tokio: {e}"))?;
    rt.block_on(async {
        let mut builder = ClientBuilder::new(addrs);
        if let (Some(user), Some(pass)) = (username, password) {
            builder = builder.sasl_config(
                rskafka::client::SaslConfig::ScramSha512(
                    rskafka::client::Credentials::new(user, pass)
                )
            );
        }
        let client = builder.build().await
            .map_err(|e| format!("Kafka.create_topic_raw: connect: {e}"))?;
        // controller_client() は rskafka v0.6 で同期関数（.await 不要）
        let controller_client = client.controller_client()
            .map_err(|e| format!("Kafka.create_topic_raw: controller: {e}"))?;
        controller_client
            .create_topic(topic, partitions, 1_i16, 5_000).await
            .map_err(|e| format!("Kafka.create_topic_raw: topic={}: {e}", topic))?;
        Ok(())
    })
}
```

> **rskafka v0.6 API 確認ポイント**:
> - `client.list_topics()` → `Result<Vec<Topic>, _>` （ping として使用）
> - `client.controller_client()` → async か sync かを vm.rs 内で確認（`controller_client().await` の可能性あり）
> - `controller_client.create_topic(name, partitions, replication_factor, timeout_ms)` の引数型（`i32` vs `i16`）を確認

#### 新規 VM primitives（Mongo.aggregate_raw の後ではなく、既存 Kafka.consume_one_raw の後に追加）

```rust
// ── Kafka 新規 primitives (v25.7.0) ──────────────────────────────────────────
#[cfg(not(target_arch = "wasm32"))]
"Kafka.connect_raw" => {
    let mut it = args.into_iter();
    let brokers_arg = vm_string(it.next().ok_or("Kafka.connect_raw: missing brokers")?, "Kafka.connect_raw")?;
    let brokers = kafka_resolve_brokers(&brokers_arg);
    match kafka_connect_sync(&brokers) {
        Ok(()) => Ok(ok_vm(VMValue::Str(brokers))),
        Err(e) => Ok(err_vm(VMValue::Str(e))),
    }
}
#[cfg(target_arch = "wasm32")]
"Kafka.connect_raw" => Ok(err_vm(VMValue::Str("Kafka not supported on wasm32".into()))),

#[cfg(not(target_arch = "wasm32"))]
"Kafka.consume_batch_raw" => {
    let mut it = args.into_iter();
    let brokers_arg  = vm_string(it.next().ok_or("Kafka.consume_batch_raw: missing brokers")?,   "Kafka.consume_batch_raw")?;
    let topic        = vm_string(it.next().ok_or("Kafka.consume_batch_raw: missing topic")?,     "Kafka.consume_batch_raw")?;
    let max_count    = match it.next().ok_or("Kafka.consume_batch_raw: missing max_count")? {
        VMValue::Int(n) => n,
        _ => return Err("Kafka.consume_batch_raw: max_count must be Int".to_string()),
    };
    let brokers = kafka_resolve_brokers(&brokers_arg);
    match kafka_consume_batch_sync(&brokers, &topic, max_count) {
        Ok(s) => Ok(ok_vm(VMValue::Str(s))),
        Err(e) => Ok(err_vm(VMValue::Str(e))),
    }
}
#[cfg(target_arch = "wasm32")]
"Kafka.consume_batch_raw" => Ok(err_vm(VMValue::Str("Kafka not supported on wasm32".into()))),

#[cfg(not(target_arch = "wasm32"))]
"Kafka.create_topic_raw" => {
    let mut it = args.into_iter();
    let brokers_arg  = vm_string(it.next().ok_or("Kafka.create_topic_raw: missing brokers")?,    "Kafka.create_topic_raw")?;
    let topic        = vm_string(it.next().ok_or("Kafka.create_topic_raw: missing topic")?,      "Kafka.create_topic_raw")?;
    let partitions   = match it.next().ok_or("Kafka.create_topic_raw: missing partitions")? {
        VMValue::Int(n) => n as i32,
        _ => return Err("Kafka.create_topic_raw: partitions must be Int".to_string()),
    };
    let brokers = kafka_resolve_brokers(&brokers_arg);
    match kafka_create_topic_sync(&brokers, &topic, partitions) {
        Ok(()) => Ok(ok_vm(VMValue::Unit)),
        Err(e) => Ok(err_vm(VMValue::Str(e))),
    }
}
#[cfg(target_arch = "wasm32")]
"Kafka.create_topic_raw" => Ok(err_vm(VMValue::Str("Kafka not supported on wasm32".into()))),
```

---

### Phase 5: runes/kafka/kafka.fav 全面更新（T4）

```favnir
// runes/kafka/kafka.fav — Kafka Rune (v25.7.0)
//
// 使い方:
//   import rune "kafka"
//
// 環境変数:
//   KAFKA_BOOTSTRAP_BROKERS — カンマ区切りのブローカーアドレス（例: "localhost:9092"）
//   KAFKA_SASL_USERNAME     — SASL 認証ユーザー名（省略可）
//   KAFKA_SASL_PASSWORD     — SASL 認証パスワード（省略可）
//
// ローカル開発:
//   docker run -p 9092:9092 redpandadata/redpanda:latest \
//       redpanda start --overprovisioned --node-id 0 \
//       --kafka-addr 0.0.0.0:9092 --advertise-kafka-addr localhost:9092

// ブローカーアドレス文字列ラッパー型
// "" → KAFKA_BOOTSTRAP_BROKERS 環境変数 → "localhost:9092"
type KafkaConn(String)

public fn connect(brokers: String) -> Result<KafkaConn, String> !Stream {
    Kafka.connect_raw(brokers)
}

public fn produce(conn: KafkaConn, topic: String, key: String, value: String) -> Result<Unit, String> !Stream {
    Kafka.produce_raw(conn, topic, key, value)
}

public fn consume_one(conn: KafkaConn, topic: String, group_id: String) -> Result<String, String> !Stream {
    Kafka.consume_one_raw(conn, topic, group_id)
}

// 最大 max_count 件のメッセージ payload を JSON 配列文字列で返す。
// 注意: group_id は現時点では rskafka v0.6 の制約により Consumer Group オフセット管理に未使用（v26.x で対応予定）。
public fn consume_batch(conn: KafkaConn, topic: String, group_id: String, max_count: Int) -> Result<String, String> !Stream {
    Kafka.consume_batch_raw(conn, topic, max_count)
}

public fn create_topic(conn: KafkaConn, topic: String, partitions: Int) -> Result<Unit, String> !Stream {
    Kafka.create_topic_raw(conn, topic, partitions)
}
```

---

### Phase 6: E2E デモ作成（T5）

`examples/kafka_events_etl.fav` — spec.md の内容どおり作成。

---

### Phase 7: ドキュメント作成（T6）

`site/content/docs/runes/kafka.mdx` — MongoDB / DynamoDB mdx を参考に全 API 記載。

---

### Phase 8: CHANGELOG 更新（T7）

`CHANGELOG.md` に `[v25.7.0]` エントリ追加。

---

### Phase 9: ベンチマーク + テスト（T8〜T10）

`benchmarks/v25.7.0.json`:

```json
{
  "version": "25.7.0",
  "test_count": 2021,
  "timestamp": "2026-06-25"
}
```

`fav/src/driver.rs` に `v257000_tests` モジュール（7 件）:

```rust
#[cfg(test)]
mod v257000_tests {
    /// ast.rs に Effect::Stream、error_catalog.rs に E0319、
    /// checker.rs に require_stream_effect が存在することを一括確認
    #[test]
    fn e0319_and_stream_effect_exist() {
        let ast_src = include_str!("ast.rs");
        assert!(ast_src.contains("Stream,"), "Effect::Stream missing in ast.rs");
        let cat_src = include_str!("error_catalog.rs");
        assert!(cat_src.contains("E0319"), "E0319 missing in error_catalog.rs");
        let chk_src = include_str!("middle/checker.rs");
        assert!(chk_src.contains("require_stream_effect"), "require_stream_effect missing in checker.rs");
    }

    #[test]
    fn kafka_connect_raw_in_vm() {
        let src = include_str!("backend/vm.rs");
        assert!(src.contains("\"Kafka.connect_raw\""), "Kafka.connect_raw missing in vm.rs");
    }

    #[test]
    fn kafka_consume_batch_raw_in_vm() {
        let src = include_str!("backend/vm.rs");
        assert!(src.contains("\"Kafka.consume_batch_raw\""), "Kafka.consume_batch_raw missing in vm.rs");
    }

    #[test]
    fn kafka_create_topic_raw_in_vm() {
        let src = include_str!("backend/vm.rs");
        assert!(src.contains("\"Kafka.create_topic_raw\""), "Kafka.create_topic_raw missing in vm.rs");
    }

    #[test]
    fn kafka_rune_has_connect_and_produce() {
        let src = include_str!("../../runes/kafka/kafka.fav");
        assert!(src.contains("fn connect"), "connect missing in kafka.fav");
        assert!(src.contains("fn produce"), "produce missing in kafka.fav");
        assert!(src.contains("type KafkaConn"), "KafkaConn missing in kafka.fav");
    }

    #[test]
    fn kafka_rune_has_consume_and_create() {
        let src = include_str!("../../runes/kafka/kafka.fav");
        assert!(src.contains("fn consume_one"), "consume_one missing in kafka.fav");
        assert!(src.contains("fn consume_batch"), "consume_batch missing in kafka.fav");
        assert!(src.contains("fn create_topic"), "create_topic missing in kafka.fav");
    }

    #[test]
    fn kafka_events_etl_example_exists() {
        let src = include_str!("../../examples/kafka_events_etl.fav");
        assert!(src.contains("import rune \"kafka\""), "import rune missing in example");
        assert!(src.contains("produce"), "produce missing in example");
        assert!(src.contains("consume_batch"), "consume_batch missing in example");
    }
}
```

---

## 注意事項

- `rskafka::client::Client::list_topics()` の戻り型は `Result<Vec<Topic>, ClientError>` — ping としてのみ使用（戻り値は捨てる）
- `client.controller_client()` は rskafka v0.6 で**同期関数**（`await` 不要）。`client.controller_client().map_err(...)? ` と呼ぶ
- `create_topic` の partition 型は `i32`（`n as i32`）。replication_factor は `1_i16`（`i16` 型）、timeout_ms は `5_000_i32`
- `kafka_resolve_brokers` は既存ヘルパー（空文字列の場合に環境変数を参照）— 新 primitives でも再利用。`kafka_resolve_brokers` は空文字列に対して必ず `"localhost:9092"` を返すため、`kafka_connect_sync` 内の "no brokers" チェックは dead code だが防御的に残す
- `KafkaConn(String)` の VM 互換性: Rune で `conn: KafkaConn` を primitive に渡すと、checker は `String` として扱い VM は `VMValue::Str` を渡す（`DynamoConn` / `MongoConn` と完全に同じパターン）
- `Effect::Stream` は既存 → ast.rs / fmt.rs 等の match 更新は不要
- `consume_batch_raw` でトピックが空の場合（latest_offset == 0）は `Ok("[]")` を返す（err ではなく ok）
- `create_topic` でトピックが既に存在する場合、rskafka は `TopicAlreadyExists` エラーを返す可能性がある — `err_vm` で伝播させる
