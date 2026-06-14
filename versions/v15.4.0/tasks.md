# v15.4.0 Tasks — Kafka / MSK Rune（`!Stream` エフェクト）

Date: 2026-06-14
Branch: master

---

## Phase A — Cargo バージョン更新 + rskafka 依存追加

- [x] A-1: `fav/Cargo.toml` の `version` を `"15.4.0"` に変更
- [x] A-2: `fav/Cargo.toml` の native-only dependencies に `rskafka = { version = "0.6", features = ["transport-tls"] }` 追加（cmake 不要の pure-Rust）
- [x] A-3: `cargo build` → コンパイルエラーなし確認

---

## Phase B — テスト追加（v154000_tests）

- [x] B-1: `fav/src/driver.rs` に `v154000_tests` モジュール追加（5 テスト）
  - `version_is_15_4_0`
  - `stream_effect_in_ast`
  - `kafka_produce_raw_primitive_exists`
  - `kafka_rune_exists`
  - `kafka_e2e_demo_structure`

---

## Phase C — AST: `Effect::Stream` 追加

- [x] C-1: `fav/src/ast.rs` の `Effect` enum に `Stream` 追加
- [x] C-2: `fav/src/fmt.rs` の Effect match に `Effect::Stream => Some("!Stream")` 追加
- [x] C-3: `fav/src/lint.rs` の Effect match に `Effect::Stream => "Stream"` 追加
- [x] C-4: `fav/src/middle/ast_lower_checker.rs` の Effect match に `ast::Effect::Stream => "Stream"` 追加
- [x] C-5: `fav/src/middle/reachability.rs` の Effect match に `Effect::Stream => { effects_required.insert("Stream") }` 追加
- [x] C-6: `fav/src/lineage.rs` の Display match に `Stream => "!Stream"` 追加

---

## Phase D — 型チェッカー更新（checker.rs）

- [x] D-1: `fav/src/middle/checker.rs` の `BUILTIN_EFFECTS` に `"Stream"` 追加
- [x] D-2: `fav/src/middle/checker.rs` の `BUILTIN_NAMESPACES` に `"Kafka"` 追加
- [x] D-3: `fav/src/middle/checker.rs` の `builtin_ret_ty` に Kafka.* 追加
- [x] D-4: `fav/src/middle/checker.rs` に `require_stream_effect` 関数追加（E0319）
- [x] D-5: `cargo build` → コンパイルエラーなし確認

---

## Phase E — コンパイラ更新（compiler.rs）

- [x] E-1: `fav/src/middle/compiler.rs` の builtin namespace リスト **2 箇所** に `"Kafka"` 追加
- [x] E-2: `cargo build` → コンパイルエラーなし確認

---

## Phase F — lineage.rs 更新

- [x] F-1: `fav/src/lineage.rs` の classify_capability_kind に `Stream` 追加
- [x] F-2: `fav/src/lineage.rs` の format_effects に `Stream => "!Stream"` 追加

---

## Phase G — VM プリミティブ実装（vm.rs）

- [x] G-1: `fav/src/backend/vm.rs` に `kafka_resolve_brokers` / `kafka_broker_list` ヘルパー追加
- [x] G-2: `fav/src/backend/vm.rs` に `kafka_produce_sync` 追加（rskafka ClientBuilder + SCRAM-SHA-512）
- [x] G-3: `fav/src/backend/vm.rs` に `kafka_consume_one_sync` 追加
- [x] G-4: `fav/src/backend/vm.rs` に `"Kafka.produce_raw"` / `"Kafka.consume_one_raw"` primitive 追加
- [x] G-5: `SaslConfig::ScramSha512(Credentials::new(user, pass))` — タプル variant 修正
- [x] G-6: `cargo build` → コンパイルエラーなし確認

---

## Phase H — FavToml: `[kafka]` セクション

- [x] H-1: `fav/src/toml.rs` に `KafkaTomlConfig` struct 追加
- [x] H-2: `fav/src/toml.rs` の `FavToml` に `pub kafka: Option<KafkaTomlConfig>` フィールド追加
- [x] H-3: `fav/src/toml.rs` の `parse_fav_toml` に `"kafka"` セクション解析追加
- [x] H-4: `fav/src/toml.rs` の FavToml 構築に `kafka: kafka_cfg` 追加
- [x] H-5: `fav/src/driver.rs` の `FavToml` struct literal に `kafka: None` 追加
- [x] H-6: `fav/src/middle/checker.rs` の `FavToml` struct literal に `kafka: None` 追加
- [x] H-7: `fav/src/middle/resolver.rs` の `FavToml` struct literal に `kafka: None` 追加
- [x] H-8: `fav/src/driver.rs` に `inject_kafka_config` 関数追加・呼び出し 2 箇所
- [x] H-9: `cargo build` → コンパイルエラーなし確認

---

## Phase I — checker.fav 更新

- [x] I-1: `fav/self/checker.fav` に `kafka_fn` スキーム追加
- [x] I-2: `fav/self/checker.fav` の `ns_to_effect` に `"Kafka" => "Stream"` 追加
- [x] I-3: `fav/self/checker.fav` の `builtin_ret_ty` に `Kafka.*` 追加

---

## Phase J — rune ファイル作成

- [x] J-1: `runes/kafka/kafka.fav` 新規作成（produce / consume_one ラッパー）

---

## Phase K — E2E デモ作成

- [x] K-1: `infra/e2e-demo/kafka/src/pipeline.fav` 作成（4-stage pipeline）
- [x] K-2: `infra/e2e-demo/kafka/terraform/aws/main.tf` 作成（aws_msk_cluster）
- [x] K-3: `infra/e2e-demo/kafka/terraform/aws/variables.tf` 作成
- [x] K-4: `infra/e2e-demo/kafka/terraform/aws/outputs.tf` 作成
- [x] K-5: `infra/e2e-demo/kafka/scripts/run.sh` 作成

---

## Phase L — テスト・コミット

- [x] L-1: `cargo test v154000` → 5/5 PASS
- [x] L-2: `cargo test` → 1568 PASS（リグレッションなし）
- [x] L-3: コミット `43f67cd` — feat: v15.4.0 — Kafka / MSK Rune + !Stream エフェクト完成（PASS=5/5）

---

## 完了条件

| 確認項目 | 状態 |
|---|---|
| `Cargo.toml version == "15.4.0"` | [x] |
| `rskafka 0.6` 依存が追加されている | [x] |
| `cargo test v154000` 全テストパス（5/5） | [x] |
| `cargo test` 全件パス（リグレッションなし） | [x] |
| `ast.rs` に `Effect::Stream` が存在する | [x] |
| `vm.rs` に `Kafka.produce_raw` primitive が存在する | [x] |
| `runes/kafka/kafka.fav` が存在する | [x] |
| `infra/e2e-demo/kafka/` の必須ファイルが存在する | [x] |
