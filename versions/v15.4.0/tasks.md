# v15.4.0 Tasks — Kafka / MSK Rune（`!Stream` エフェクト）

Date: 2026-06-14
Branch: master

---

## Phase A — Cargo バージョン更新 + rdkafka 依存追加

- [ ] A-1: `fav/Cargo.toml` の `version` を `"15.4.0"` に変更
- [ ] A-2: `fav/Cargo.toml` の native-only dependencies に `rdkafka = { version = "0.36", features = ["cmake-build", "sasl", "ssl"] }` 追加
- [ ] A-3: `cargo build` → コンパイルエラーなし確認（rdkafka リンク成功）

---

## Phase B — テスト追加（v154000_tests）

- [ ] B-1: `fav/src/driver.rs` に `v154000_tests` モジュール追加（5 テスト）
  - `version_is_15_4_0`
  - `stream_effect_in_ast`
  - `kafka_produce_raw_primitive_exists`
  - `kafka_rune_exists`
  - `kafka_e2e_demo_structure`

---

## Phase C — AST: `Effect::Stream` 追加

- [ ] C-1: `fav/src/ast.rs` の `Effect` enum に `Stream` 追加
- [ ] C-2: `fav/src/fmt.rs` の Effect match に `Effect::Stream => Some("!Stream")` 追加
- [ ] C-3: `fav/src/lint.rs` の Effect match に `Effect::Stream => "Stream"` 追加
- [ ] C-4: `fav/src/middle/ast_lower_checker.rs` の Effect match に `ast::Effect::Stream => "Stream"` 追加
- [ ] C-5: `fav/src/middle/reachability.rs` の Effect match に `Effect::Stream => { effects_required.insert("Stream") }` 追加
- [ ] C-6: `fav/src/lineage.rs` の Display match に `Stream => "!Stream"` 追加

---

## Phase D — 型チェッカー更新（checker.rs）

- [ ] D-1: `fav/src/middle/checker.rs` の `BUILTIN_EFFECTS` に `"Stream"` 追加
- [ ] D-2: `fav/src/middle/checker.rs` の `BUILTIN_NAMESPACES` に `"Kafka"` 追加
- [ ] D-3: `fav/src/middle/checker.rs` の `builtin_ret_ty` に Kafka.* 追加
  - `("Kafka", "produce_raw")` → `"Result<Unit, String>"`
  - `("Kafka", "consume_one_raw")` → `"Result<String, String>"`
- [ ] D-4: `fav/src/middle/checker.rs` に `require_stream_effect` 関数追加（E0319）
- [ ] D-5: `cargo build` → コンパイルエラーなし確認

---

## Phase E — コンパイラ更新（compiler.rs）

- [ ] E-1: `fav/src/middle/compiler.rs` の builtin namespace リスト **2 箇所** に `"Kafka"` 追加
  - 1 箇所目: ~line 193付近（`"BigQuery"` の後）
  - 2 箇所目: ~line 436付近（`"BigQuery"` の後）
- [ ] E-2: `cargo build` → コンパイルエラーなし確認

---

## Phase F — lineage.rs 更新

- [ ] F-1: `fav/src/lineage.rs` の `EffectKind` enum に `StreamRead` / `StreamWrite` 追加
- [ ] F-2: `fav/src/lineage.rs` に `collect_kafka_call_kinds` 関数追加
  - `Kafka.produce_raw` → `StreamWrite`
  - `Kafka.consume_one_raw` → `StreamRead`
- [ ] F-3: `analyze_lineage` で Kafka エフェクトを収集するよう更新

---

## Phase G — VM プリミティブ実装（vm.rs）

- [ ] G-1: `fav/src/backend/vm.rs` に `kafka_config_from_env` ヘルパー関数追加
  - `KAFKA_BOOTSTRAP_BROKERS` / `KAFKA_SASL_USERNAME` / `KAFKA_SASL_PASSWORD` / `KAFKA_USE_TLS` 環境変数を読む
- [ ] G-2: `fav/src/backend/vm.rs` に `"Kafka.produce_raw"` primitive 追加
  - 引数: `(brokers: String, topic: String, key: String, value: String)`
  - rdkafka `FutureProducer` で produce → `Result.ok(Unit)` または `Result.err(String)`
- [ ] G-3: `fav/src/backend/vm.rs` に `"Kafka.consume_one_raw"` primitive 追加
  - 引数: `(brokers: String, topic: String, group_id: String)`
  - rdkafka `StreamConsumer` でサブスクライブ → 1 メッセージ取得（タイムアウト 10秒）
  - `Result.ok(payload_string)` または `Result.err(String)`
- [ ] G-4: `cargo build` → コンパイルエラーなし確認

---

## Phase H — FavToml: `[kafka]` セクション

- [ ] H-1: `fav/src/toml.rs` に `KafkaTomlConfig` struct 追加
  ```rust
  pub struct KafkaTomlConfig {
      pub bootstrap_brokers: Option<String>,
      pub sasl_mechanism: Option<String>,
      pub sasl_username: Option<String>,
      pub sasl_password: Option<String>,
  }
  ```
- [ ] H-2: `fav/src/toml.rs` の `FavToml` に `pub kafka: Option<KafkaTomlConfig>` フィールド追加
- [ ] H-3: `fav/src/driver.rs` の `FavToml` struct literal（約 3 箇所）に `kafka: None` 追加
- [ ] H-4: `fav/src/middle/checker.rs` の `FavToml` struct literal（約 2 箇所）に `kafka: None` 追加
- [ ] H-5: `fav/src/middle/resolver.rs` の `FavToml` struct literal（約 3 箇所）に `kafka: None` 追加
- [ ] H-6: `fav/src/driver.rs` に `inject_kafka_config(toml: &FavToml)` 関数追加
- [ ] H-7: `cargo build` → コンパイルエラーなし確認

---

## Phase I — checker.fav 更新

- [ ] I-1: `fav/self/checker.fav` に `kafka_fn` スキーム追加
- [ ] I-2: `fav/self/checker.fav` の `ns_to_effect` に `"Kafka" => "Stream"` 追加
- [ ] I-3: `fav/self/checker.fav` の `builtin_ret_ty` に `Kafka.*` 追加
  - `"Kafka.produce_raw"` → `"Result<Unit, String>"`
  - `"Kafka.consume_one_raw"` → `"Result<String, String>"`

---

## Phase J — rune ファイル作成

- [ ] J-1: `runes/kafka/kafka.fav` 新規作成
  - `produce !Stream (topic, key, value) -> Result<Unit, String>`
  - `consume_one !Stream (topic, group_id) -> Result<String, String>`

---

## Phase K — E2E デモ作成

- [ ] K-1: `infra/e2e-demo/kafka/src/pipeline.fav` 作成
  - 4 ステージ: `ExtractFromRds |> TransformRows |> ProduceToKafka |> ConsumeAndWrite`
  - エフェクト: `!Db !Stream`
- [ ] K-2: `infra/e2e-demo/kafka/terraform/aws/main.tf` 作成
  - `aws_msk_cluster`（kafka 3.6.0, kafka.t3.small, SASL/SCRAM + TLS）
  - `aws_secretsmanager_secret` に SASL 認証情報を保存
  - outputs: `bootstrap_brokers_sasl_scram`
- [ ] K-3: `infra/e2e-demo/kafka/terraform/aws/variables.tf` 作成
- [ ] K-4: `infra/e2e-demo/kafka/terraform/aws/outputs.tf` 作成
- [ ] K-5: `infra/e2e-demo/kafka/scripts/seed.sh` 作成（RDS にサンプルデータ投入）
- [ ] K-6: `infra/e2e-demo/kafka/scripts/run.sh` 作成（pipeline.fav 実行 + PASS/FAIL 判定）
- [ ] K-7: `infra/e2e-demo/kafka/scripts/verify.sh` 作成（Kafka topic メッセージ確認）
- [ ] K-8: `infra/e2e-demo/kafka/README.md` 作成

---

## Phase L — テスト・コミット

- [ ] L-1: `cargo test v154000` → 5/5 パス確認
- [ ] L-2: `cargo test` → 全件パス（リグレッションなし）確認
- [ ] L-3: （任意）E2E デモ実行
  - `terraform apply` → MSK クラスター起動（約 20 分）
  - `scripts/seed.sh` → RDS にデータ投入
  - `scripts/run.sh` → PASS=1 確認
  - `scripts/verify.sh` → Kafka topic 確認
  - `terraform destroy` → **必須**（MSK は常時課金）
  - `infra/e2e-demo/kafka/trail/run-output.txt` に証跡保存
- [ ] L-4: コミット
  ```
  feat: v15.4.0 — Kafka / MSK Rune（!Stream エフェクト）
  ```

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `Cargo.toml version == "15.4.0"` | [ ] |
| `rdkafka` 依存が追加されている | [ ] |
| `cargo test v154000` 全テストパス（5/5） | [ ] |
| `cargo test` 全件パス（リグレッションなし） | [ ] |
| `ast.rs` に `Effect::Stream` が存在する | [ ] |
| `vm.rs` に `Kafka.produce_raw` primitive が存在する | [ ] |
| `runes/kafka/kafka.fav` が存在する | [ ] |
| `infra/e2e-demo/kafka/` の必須ファイルが存在する | [ ] |
| `fav run pipeline.fav` で produce → consume が実行される | [ ] |
| `terraform destroy` 完了（MSK 課金停止） | [ ] |

---

## 参照ファイル

| ファイル | 目的 |
|---|---|
| `versions/v15.4.0/spec.md` | 仕様・スコープ |
| `versions/v15.4.0/plan.md` | 各フェーズの具体的な変更内容 |
| `versions/roadmap-v15.1-v16.0.md` | v15.4.0 セクション |
| `fav/src/ast.rs` | `Effect` enum — 追加対象 |
| `fav/src/backend/vm.rs` | 既存 `BigQuery.*` primitive — 参考パターン |
| `fav/src/middle/checker.rs` | `BUILTIN_EFFECTS` / `builtin_ret_ty` |
| `fav/src/middle/compiler.rs` | builtin namespace リスト（2 箇所） |
| `fav/src/lineage.rs` | `EffectKind` enum |
| `fav/src/toml.rs` | `GcpTomlConfig` — 参考パターン |
| `infra/e2e-demo/bigquery/` | E2E デモ構造の参考 |
