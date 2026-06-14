# v15.4.0 Plan — Kafka / MSK Rune（`!Stream` エフェクト）

Date: 2026-06-14
Branch: master

---

## Phase A — Cargo バージョン更新 + rdkafka 依存追加

### A-1: `fav/Cargo.toml` version 更新

```toml
version = "15.4.0"
```

### A-2: `fav/Cargo.toml` rdkafka 依存追加

`[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` セクションに追加:

```toml
rdkafka = { version = "0.36", features = ["cmake-build", "sasl", "ssl"] }
```

**注意**: `cmake-build` feature は librdkafka を cmake でビルドする。Windows では cmake が必要。

---

## Phase B — テスト追加（v154000_tests）

`fav/src/driver.rs` の末尾付近（v153000_tests の前）に追加:

```rust
// ── v154000_tests (v15.4.0) — Kafka / MSK Rune ───────────────────────────────
#[cfg(test)]
mod v154000_tests {
    use std::fs;
    use std::path::Path;

    #[test]
    fn version_is_15_4_0() {
        let cargo = fs::read_to_string("Cargo.toml").unwrap();
        assert!(cargo.contains("version = \"15.4.0\""), ...);
    }

    #[test]
    fn stream_effect_in_ast() {
        let ast = fs::read_to_string("src/ast.rs").unwrap();
        assert!(ast.contains("Stream"), ...);
    }

    #[test]
    fn kafka_produce_raw_primitive_exists() {
        let vm = fs::read_to_string("src/backend/vm.rs").unwrap();
        assert!(vm.contains("Kafka.produce_raw"), ...);
    }

    #[test]
    fn kafka_rune_exists() {
        assert!(Path::new("../runes/kafka/kafka.fav").exists(), ...);
    }

    #[test]
    fn kafka_e2e_demo_structure() {
        assert!(Path::new("../infra/e2e-demo/kafka/src/pipeline.fav").exists(), ...);
        assert!(Path::new("../infra/e2e-demo/kafka/terraform/aws/main.tf").exists(), ...);
        assert!(Path::new("../infra/e2e-demo/kafka/scripts/run.sh").exists(), ...);
        assert!(Path::new("../infra/e2e-demo/kafka/README.md").exists(), ...);
    }
}
```

---

## Phase C — AST: `Effect::Stream` 追加

### C-1: `fav/src/ast.rs`

`pub enum Effect` に `Stream` を追加:

```rust
pub enum Effect {
    Db,
    AzureDb,
    AzureStorage,
    Gcp,
    Stream,   // ← 追加
    // ... 既存 ...
}
```

### C-2: exhaustive match の更新（コンパイルエラー解消）

以下のファイルで `Effect::Stream` ブランチを追加:

| ファイル | 変更内容 |
|---|---|
| `fav/src/fmt.rs` | `Effect::Stream => Some("!Stream")` |
| `fav/src/lint.rs` | `Effect::Stream => "Stream"` |
| `fav/src/middle/ast_lower_checker.rs` | `ast::Effect::Stream => "Stream"` |
| `fav/src/middle/reachability.rs` | `Effect::Stream => { effects_required.insert("Stream") }` |
| `fav/src/lineage.rs` | `Stream => "!Stream"` in Display |

---

## Phase D — 型チェッカー更新（checker.rs）

### D-1: `Effect::Stream` を `BUILTIN_EFFECTS` に追加

```rust
static BUILTIN_EFFECTS: &[&str] = &[
    // ... 既存 ...
    "Gcp",
    "Stream",  // ← 追加
];
```

### D-2: `builtin_ret_ty` に `Kafka.*` 追加

```rust
("Kafka", "produce_raw")    => "Result<Unit, String>",
("Kafka", "consume_one_raw") => "Result<String, String>",
```

### D-3: `BUILTIN_NAMESPACES` に `"Kafka"` 追加

```rust
static BUILTIN_NAMESPACES: &[&str] = &[
    // ... 既存 ...
    "BigQuery",
    "Kafka",   // ← 追加
];
```

### D-4: `require_stream_effect` 関数追加（E0319）

BigQuery の `require_gcp_effect` パターンを踏襲:

```rust
fn require_stream_effect(fn_effects: &[Effect], span: Span) -> Option<CheckError> {
    if !fn_effects.iter().any(|e| *e == Effect::Stream) {
        Some(CheckError {
            code: "E0319",
            message: "Kafka.* requires !Stream effect declaration",
            span,
        })
    } else {
        None
    }
}
```

---

## Phase E — コンパイラ更新（compiler.rs）

### E-1: builtin namespace リスト 2 箇所に `"Kafka"` 追加

`fav/src/middle/compiler.rs` の builtin globals リストが 2 箇所あり、両方に追加:

```rust
// 1 箇所目（~line 192付近）
"BigQuery",
"Kafka",   // ← 追加
"Postgres",

// 2 箇所目（~line 435付近）
"BigQuery",
"Kafka",   // ← 追加
"Postgres",
```

**注意**: 1 箇所だけ追加すると、一方でコンパイルは通るが実行時に `global index out of bounds` が発生する。必ず 2 箇所追加する。

---

## Phase F — lineage.rs 更新

### F-1: `EffectKind` enum に `StreamRead` / `StreamWrite` 追加

```rust
pub enum EffectKind {
    // ... 既存 ...
    GcpRead,
    GcpWrite,
    StreamRead,  // ← 追加
    StreamWrite, // ← 追加
}
```

### F-2: `collect_kafka_call_kinds` 関数追加

BigQuery の `collect_bigquery_call_kinds` パターンを踏襲:

```rust
fn collect_kafka_call_kinds(program: &Program) -> Vec<EffectKind> {
    // Kafka.produce_raw → StreamWrite
    // Kafka.consume_one_raw → StreamRead
}
```

### F-3: `analyze_lineage` で Kafka エフェクトを収集

---

## Phase G — VM プリミティブ実装（vm.rs）

### G-1: `Kafka.produce_raw` プリミティブ追加

```rust
"Kafka.produce_raw" => {
    // 引数: brokers(String), topic(String), key(String), value(String)
    let brokers = /* env var or arg */;
    let topic   = /* arg */;
    let key     = /* arg */;
    let value   = /* arg */;

    // rdkafka: ClientConfig::new()
    //   .set("bootstrap.servers", &brokers)
    //   .set("security.protocol", if use_tls { "SASL_SSL" } else { "PLAINTEXT" })
    //   .set("sasl.mechanisms", "SCRAM-SHA-512")
    //   .set("sasl.username", &username)
    //   .set("sasl.password", &password)
    //   .create::<FutureProducer>()
    // producer.send(FutureRecord::to(&topic).key(&key).payload(&value), ...)
    Ok(VMValue::Variant("ok".into(), Some(Box::new(VMValue::Unit))))
}
```

### G-2: `Kafka.consume_one_raw` プリミティブ追加

```rust
"Kafka.consume_one_raw" => {
    // 引数: brokers(String), topic(String), group_id(String)
    // StreamConsumer でサブスクライブ、recv() で 1 メッセージ取得
    // タイムアウト: 10秒
    Ok(VMValue::Variant("ok".into(), Some(Box::new(VMValue::Str(payload)))))
}
```

**注意**: `rdkafka` の async API は `tokio` が必要。既存の `tokio` 依存を流用する。

### G-3: `kafka_config_from_env` ヘルパー関数

```rust
fn kafka_config_from_env() -> (String, Option<String>, Option<String>, bool) {
    let brokers  = env_or("KAFKA_BOOTSTRAP_BROKERS", "localhost:9092");
    let username = std::env::var("KAFKA_SASL_USERNAME").ok();
    let password = std::env::var("KAFKA_SASL_PASSWORD").ok();
    let use_tls  = env_or("KAFKA_USE_TLS", "true") == "true";
    (brokers, username, password, use_tls)
}
```

---

## Phase H — FavToml: `[kafka]` セクション

### H-1: `fav/src/toml.rs` に `KafkaTomlConfig` 追加

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KafkaTomlConfig {
    pub bootstrap_brokers: Option<String>,
    pub sasl_mechanism: Option<String>,
    pub sasl_username: Option<String>,
    pub sasl_password: Option<String>,
}
```

### H-2: `FavToml` に `kafka: Option<KafkaTomlConfig>` フィールド追加

**注意**: `FavToml` の struct literal が `driver.rs` / `checker.rs` / `resolver.rs` に複数あり、すべてに `kafka: None` を追加する。

### H-3: `fav/src/driver.rs` に `inject_kafka_config` 関数追加

```rust
fn inject_kafka_config(toml: &FavToml) {
    if let Some(ref kafka) = toml.kafka {
        if let Some(ref brokers) = kafka.bootstrap_brokers {
            std::env::set_var("KAFKA_BOOTSTRAP_BROKERS", expand_env_vars(brokers));
        }
        // sasl_username / sasl_password も同様
    }
}
```

---

## Phase I — checker.fav 更新

### I-1: `kafka_fn` スキーム追加

```fav
fn kafka_fn(fname: String) -> String {
    if fname == "produce_raw" { "(String, String, String, String) -> Result<Unit, String>" }
    else { "(String, String, String) -> Result<String, String>" }
}
```

### I-2: `ns_to_effect` に `"Kafka"` → `"Stream"` 追加

```fav
"Kafka" => "Stream"
```

### I-3: `builtin_ret_ty` に `Kafka.*` 追加

```fav
"Kafka.produce_raw"     => "Result<Unit, String>"
"Kafka.consume_one_raw" => "Result<String, String>"
```

---

## Phase J — rune ファイル作成

### J-1: `runes/kafka/kafka.fav` 新規作成

```fav
namespace kafka

fn produce !Stream (topic: String, key: String, value: String) -> Result<Unit, String> {
    Kafka.produce_raw(Env.get("KAFKA_BOOTSTRAP_BROKERS"), topic, key, value)
}

fn consume_one !Stream (topic: String, group_id: String) -> Result<String, String> {
    Kafka.consume_one_raw(Env.get("KAFKA_BOOTSTRAP_BROKERS"), topic, group_id)
}
```

---

## Phase K — E2E デモ

### K-1: `infra/e2e-demo/kafka/src/pipeline.fav` 作成

4 ステージ pipeline:
```
ExtractFromRds |> TransformRows |> ProduceToKafka |> ConsumeAndWrite
```

### K-2: `infra/e2e-demo/kafka/terraform/aws/main.tf` 作成

```hcl
resource "aws_msk_cluster" "kafka" {
  cluster_name           = "favnir-kafka"
  kafka_version          = "3.6.0"
  number_of_broker_nodes = 2

  broker_node_group_info {
    instance_type  = "kafka.t3.small"
    client_subnets = [...]
    storage_info {
      ebs_storage_info { volume_size = 20 }
    }
  }

  client_authentication {
    sasl { scram = true }
  }

  encryption_info {
    encryption_in_transit { client_broker = "TLS" }
  }
}
```

### K-3: `infra/e2e-demo/kafka/scripts/` 作成

- `seed.sh`: RDS にサンプルデータ投入
- `run.sh`: `fav run src/pipeline.fav`（PASS/FAIL 判定付き）
- `verify.sh`: Kafka topic メッセージ確認

### K-4: `infra/e2e-demo/kafka/README.md` 作成

---

## Phase L — コミット

### L-1: `cargo test v154000` → 5/5 パス最終確認

### L-2: `cargo test` → 全件パス（リグレッションなし）確認

### L-3: E2E デモ実行（任意 — MSK 課金あり）

`terraform apply` → `scripts/run.sh` → PASS=1 確認 → `terraform destroy`

### L-4: コミット

```
feat: v15.4.0 — Kafka / MSK Rune（!Stream エフェクト）
```

---

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/Cargo.toml` | 更新 | version 15.4.0 + rdkafka 依存追加 |
| `fav/src/ast.rs` | 更新 | Effect::Stream 追加 |
| `fav/src/fmt.rs` | 更新 | Effect::Stream exhaustive match |
| `fav/src/lint.rs` | 更新 | Effect::Stream exhaustive match |
| `fav/src/middle/ast_lower_checker.rs` | 更新 | Effect::Stream exhaustive match |
| `fav/src/middle/reachability.rs` | 更新 | Effect::Stream exhaustive match |
| `fav/src/lineage.rs` | 更新 | StreamRead / StreamWrite EffectKind |
| `fav/src/middle/checker.rs` | 更新 | Stream effect + Kafka.* builtin_ret_ty |
| `fav/src/middle/compiler.rs` | 更新 | "Kafka" を builtin リスト 2 箇所に追加 |
| `fav/src/backend/vm.rs` | 更新 | Kafka.produce_raw / consume_one_raw primitive |
| `fav/src/toml.rs` | 更新 | KafkaTomlConfig + FavToml.kafka |
| `fav/src/driver.rs` | 更新 | inject_kafka_config + v154000_tests |
| `fav/self/checker.fav` | 更新 | kafka_fn / ns_to_effect / builtin_ret_ty |
| `runes/kafka/kafka.fav` | 新規 | Kafka rune |
| `infra/e2e-demo/kafka/` | 新規 | E2E デモ一式 |

---

## 実装上の注意点

1. **rdkafka の async**: `FutureProducer` / `StreamConsumer` は tokio ランタイムが必要。既存 `tokio` 依存を流用。VM から呼ぶ際は `tokio::runtime::Runtime::new().unwrap().block_on(...)` でブロッキング実行。

2. **MSK TLS 証明書**: AWS MSK は ACM の証明書を使用。`rdkafka` の `ssl.ca.location` は通常不要（システムの CA バンドルを使用）。

3. **FavToml の struct literal**: `checker.rs` に約 2 箇所、`driver.rs` に約 3 箇所、`resolver.rs` に約 3 箇所存在。すべてに `kafka: None` を追加する（`replace_all` を使うと安全）。

4. **MSK コスト**: `kafka.t3.small` 2ノードで約 $0.10/時間。E2E テスト後は必ず `terraform destroy`。

5. **Windows ビルド**: `rdkafka` の `cmake-build` feature は cmake が必要。`choco install cmake` または CMake 公式インストーラーで対応。
