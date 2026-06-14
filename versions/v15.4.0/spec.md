# v15.4.0 Spec — Kafka / MSK Rune（`!Stream` エフェクト）

Date: 2026-06-14
Branch: master

---

## テーマ

AWS MSK（Managed Streaming for Apache Kafka）を Favnir から操作できるようにする。
`!Stream` エフェクトを追加し、CDC・ストリーミングパイプラインの基礎を作る。
`Kafka.produce_raw` / `Kafka.consume_one_raw` の 2 プリミティブを提供し、
RDS → Kafka → Azure Postgres の CDC 的な 3 クラウド横断デモで動作を実証する。

---

## スコープ

### A: 新規 Cargo 依存

```toml
rdkafka = { version = "0.36", features = ["cmake-build", "sasl", "ssl"] }
```

librdkafka の Rust ラッパー。MSK の SASL/SCRAM + TLS に対応。

### B: AST — `Effect::Stream` 追加

```rust
pub enum Effect {
    // ... 既存 ...
    Stream,
}
```

`!Stream` エフェクトを宣言した関数のみ `Kafka.*` プリミティブを呼び出せる。

### C: VM プリミティブ（2 種類）

| Primitive | シグネチャ | 説明 |
|---|---|---|
| `Kafka.produce_raw` | `(brokers, topic, key, value) -> Result<Unit, String>` | Kafka topic にメッセージを produce |
| `Kafka.consume_one_raw` | `(brokers, topic, group_id) -> Result<String, String>` | Kafka topic から 1 メッセージを consume |

**接続情報（環境変数）:**

| 変数 | 説明 | デフォルト |
|---|---|---|
| `KAFKA_BOOTSTRAP_BROKERS` | ブローカーアドレス（カンマ区切り） | — |
| `KAFKA_SASL_USERNAME` | SASL/SCRAM ユーザー名 | — |
| `KAFKA_SASL_PASSWORD` | SASL/SCRAM パスワード | — |
| `KAFKA_USE_TLS` | TLS 有効化フラグ（`"true"` / `"false"`） | `"true"` |

環境変数未設定の場合、ブローカーアドレスを関数引数 `brokers` から取得する。

**認証方式:**
- `KAFKA_SASL_USERNAME` が設定されている場合: SASL/SCRAM-SHA-512
- 設定されていない場合: PLAINTEXT（開発用 / IAM プラグイン非対応の場合）

### D: 型チェッカー更新

- `Effect::Stream` を `checker.rs` の `BUILTIN_EFFECTS` に追加
- `builtin_ret_ty` に `Kafka.*` 追加:
  - `Kafka.produce_raw` → `Result<Unit, String>`
  - `Kafka.consume_one_raw` → `Result<String, String>`
- E0319: `!Stream` エフェクトなしで `Kafka.*` を呼び出した場合のエラーコード

### E: コンパイラ更新

`compiler.rs` の builtin namespace リスト 2 箇所に `"Kafka"` を追加。
（未登録だと `IRExpr::Global(u16::MAX)` → `global index out of bounds` at runtime）

### F: lineage.rs

```rust
EffectKind::StreamRead,
EffectKind::StreamWrite,
```

`Kafka.consume_one_raw` → `StreamRead`、`Kafka.produce_raw` → `StreamWrite`。

### G: checker.fav 更新（セルフホスト型チェック）

```fav
// kafka_fn: Kafka プリミティブのスキーム定義
fn kafka_fn(fname: String) -> String { ... }

// ns_to_effect
"Kafka" => "Stream"

// builtin_ret_ty
"Kafka.produce_raw"    => "Result<Unit, String>"
"Kafka.consume_one_raw" => "Result<String, String>"
```

### H: `fav.toml [kafka]` セクション

```toml
[kafka]
bootstrap_brokers = "broker1:9096,broker2:9096"
sasl_mechanism    = "SCRAM-SHA-512"          # or "PLAIN"
sasl_username     = "${KAFKA_SASL_USERNAME}"
sasl_password     = "${KAFKA_SASL_PASSWORD}"
```

`expand_env_vars` で `${VAR}` を展開。`inject_kafka_config` が環境変数に設定。

### I: Rune ファイル

`runes/kafka/kafka.fav`:

```fav
namespace kafka

fn produce !Stream (topic: String, key: String, value: String) -> Result<Unit, String> {
    Kafka.produce_raw(Env.get("KAFKA_BOOTSTRAP_BROKERS"), topic, key, value)
}

fn consume_one !Stream (topic: String, group_id: String) -> Result<String, String> {
    Kafka.consume_one_raw(Env.get("KAFKA_BOOTSTRAP_BROKERS"), topic, group_id)
}
```

### J: E2E デモ

`infra/e2e-demo/kafka/`:

```
kafka/
├── src/
│   └── pipeline.fav       # RDS → Kafka → Azure Postgres の CDC 的デモ（バッチ処理）
├── terraform/
│   └── aws/
│       └── main.tf         # aws_msk_cluster（kafka 3.6.0, kafka.t3.small）
├── scripts/
│   ├── seed.sh             # AWS RDS にサンプルデータ投入
│   ├── run.sh              # fav run pipeline.fav
│   └── verify.sh           # Kafka topic にメッセージが届いていることを確認
└── README.md
```

**pipeline.fav の 4 ステージ:**

```
ExtractFromRds |> TransformRows |> ProduceToKafka |> ConsumeAndWrite
```

1. `ExtractFromRds`: RDS から最新 N 行を取得
2. `TransformRows`: 各行を JSON 文字列にシリアライズ
3. `ProduceToKafka`: Kafka topic `cdc-rows` に produce
4. `ConsumeAndWrite`: Kafka から consume して Azure Postgres に insert

### K: テスト（v154000_tests — 5 件）

1. `version_is_15_4_0`: Cargo.toml version == "15.4.0"
2. `stream_effect_in_ast`: `ast.rs` に `Stream` が含まれる
3. `kafka_produce_raw_primitive_exists`: `vm.rs` に `Kafka.produce_raw` が含まれる
4. `kafka_rune_exists`: `runes/kafka/kafka.fav` が存在する
5. `kafka_e2e_demo_structure`: `infra/e2e-demo/kafka/` の必須ファイルが存在する

---

## 完了条件

1. `cargo test v154000` → 5/5 パス
2. `cargo test` → リグレッションなし
3. `Cargo.toml version == "15.4.0"`
4. `fav run pipeline.fav` で Kafka への produce/consume が実行される
5. E2E デモ: `scripts/run.sh` PASS=1 FAIL=0（produce → consume ラウンドトリップ）
6. `terraform destroy` 完了（MSK クラスターは常時課金のため必須）

---

## 新規 Cargo 依存

| Crate | バージョン | features |
|---|---|---|
| `rdkafka` | `0.36` | `cmake-build`, `sasl`, `ssl` |

**注意**: Windows 開発環境では `cmake` が必要。CI（Linux）では問題なし。

---

## 既知の制約・スコープ外

- バッチ消費・オフセット管理は v15.5.x 以降
- MSK IAM 認証（IAM プラグイン）は対象外（SASL/SCRAM のみ）
- MSK Serverless は対象外（Provisioned のみ）
- Kafka Schema Registry は対象外
- `fav test` での Kafka テスト（Kafka モック）は対象外
- Azure Event Hubs（Kafka 互換）は対象外（v16.x 以降）
- MSK コストに注意: E2E 完了後は必ず `terraform destroy` を実施

---

## 参照

- `versions/roadmap-v15.1-v16.0.md` — v15.4.0 セクション
- `fav/src/backend/vm.rs` — 既存 `BigQuery.*` primitive（参考パターン）
- `fav/src/ast.rs` — `Effect` enum（追加対象）
- `fav/src/middle/checker.rs` — `BUILTIN_EFFECTS` / `builtin_ret_ty`
- `fav/src/middle/compiler.rs` — builtin namespace リスト
- `fav/src/lineage.rs` — `EffectKind`
- `infra/e2e-demo/bigquery/` — E2E デモ構造の参考
