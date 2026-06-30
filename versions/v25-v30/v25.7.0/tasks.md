# v25.7.0 タスクリスト — kafka Rune 実質化

**状態**: COMPLETE
**開始日**: 2026-06-25
**完了日**: 2026-06-25

---

## タスク

| ID | タスク | 状態 |
|---|---|---|
| T0 | `fav/Cargo.toml` を `version = "25.7.0"` に bump（rskafka v0.6 は既存のため追加 crate 不要） | [x] |
| T1 | `fav/src/error_catalog.rs` 更新（E0319 `UndeclaredStreamEffect` 追加 — **E0315 エントリの直後、E0320 の直前**に挿入） | [x] |
| T2 | `fav/src/middle/checker.rs` 更新（`Kafka.connect_raw` / `Kafka.consume_batch_raw` / `Kafka.create_topic_raw` 型追加、`require_stream_effect` に `\"E0319\"` 設定コメント追記） | [x] |
| T3 | `cargo build` で exhaustive match エラーなし確認（Effect::Stream は既存のため新規アーム追加不要） | [x] |
| T4 | `fav/src/backend/vm.rs` 更新（`kafka_connect_sync` / `kafka_consume_batch_sync` / `kafka_create_topic_sync` ヘルパー + `Kafka.connect_raw` / `Kafka.consume_batch_raw` / `Kafka.create_topic_raw` 3 primitives 追加） | [x] |
| T5 | `runes/kafka/kafka.fav` 全面更新（`type KafkaConn(String)` + `connect` / `produce` / `consume_one` / `consume_batch` / `create_topic` 5 関数） | [x] |
| T6 | `examples/kafka_events_etl.fav` 新規作成（`import rune "kafka"` + PublishEvent / ConsumeEvents / EventsETL pipeline） | [x] |
| T7 | `site/content/docs/runes/kafka.mdx` 新規作成（全 API 記載、Redpanda セットアップ手順含む） | [x] |
| T8 | `CHANGELOG.md` 更新（`[v25.7.0]` エントリ追加） | [x] |
| T9 | `benchmarks/v25.7.0.json` 新規作成（test_count: 2021） | [x] |
| T10 | `fav/src/driver.rs` 更新（`v257000_tests` 7 件追加） | [x] |
| T11 | `cargo test v257000` — 7 件 PASS 確認 | [x] |
| T12 | `cargo test` 総テスト数 ≥ 2021 件 確認（2021 件 = 2020 pass + 1 pre-existing LSP failure） | [x] |
| T13 | spec-reviewer レビュー実施 | [x] |

---

## チェックリスト（完了条件）

- [x] `Kafka.connect` が `runes/kafka/kafka.fav` に存在する
- [x] `Kafka.consume_one` / `Kafka.consume_batch` が実装済み（read 系）
- [x] `Kafka.produce` / `Kafka.create_topic` が実装済み（write 系）
- [x] `Kafka.connect_raw` / `Kafka.consume_batch_raw` / `Kafka.create_topic_raw` が `fav/src/backend/vm.rs` に存在する
- [x] E0319 が `fav/src/error_catalog.rs` に存在する
- [x] `examples/kafka_events_etl.fav` が存在し `import rune "kafka"` + `produce` + `consume_batch` を含む
- [x] `CHANGELOG.md` に `[v25.7.0]` エントリが存在する
- [x] `site/content/docs/runes/kafka.mdx` に新規 API が記載済み
- [x] `cargo test v257000` で 7 件すべて PASS
- [x] 総テスト数 ≥ 2021 件

---

## メモ

- `Effect::Stream` は v15.4.0 から既存（ast.rs / checker.rs に存在）→ 新規 Effect バリアント追加不要
- E0319 は checker.rs の `require_stream_effect` が既に使用中だが、`error_catalog.rs` に未登録 → T1 で登録のみ
- 既存 `Kafka.produce_raw` / `Kafka.consume_one_raw` は変更しない（v15.4.0 互換維持）
- `kafka_resolve_brokers` ヘルパーが既存（空文字列 → `KAFKA_BOOTSTRAP_BROKERS` → `"localhost:9092"`）
- `KafkaConn(String)`: checker は `Result<String, String>` として扱う（DynamoConn / MongoConn と同パターン）
- `consume_batch`: Latest オフセットから最大 max_count 件を取得し JSON 配列文字列で返す。空の場合は `"[]"`
- `group_id` は引数として受け取るが rskafka v0.6 では未使用（v26.x で Consumer Group 実装時に使用）
- `create_topic`: `controller_client()?.create_topic(topic, partitions as i32, 1_i16, 5_000).await` — `controller_client()` は**同期関数**（`.await` 不要）、`replication_factor` は `i16` 型
- `cfg(not(target_arch = "wasm32"))` ガードを全新規 Kafka primitive に付与
- 目標テスト数 2021 件（v25.6.0 終了時 2014 件 + 7 件）
- **ロードマップ差分**: `Kafka.commit` / `Kafka.seek` / `Kafka.consume[T]` はロードマップ記載だが v25.7.0 スコープ外。rskafka v0.6 は Consumer Group プロトコル（JoinGroup/SyncGroup/OffsetFetch）未対応のため v26.x に延期

---

## コードレビュー指摘（code-reviewer）

| 指摘 | 対応 |
|---|---|
| [HIGH] `list_topics()` の戻り値に `let _ =` が抜けている（`#[must_use]` 警告リスク） | `kafka_connect_sync` 内で `let _ = client.list_topics().await...` に修正 |
| [HIGH] 既存 `Kafka.produce_raw` / `Kafka.consume_one_raw` に wasm32 ガードがなく新規 3 件と非対称 | 両 primitive に `#[cfg(not(target_arch = "wasm32"))]` + wasm32 フォールバックアームを追加 |
| [HIGH] `produce` / `consume_one` が `conn: KafkaConn` を primitive の `brokers` に渡すことが不明確 | vm.rs の既存 2 primitive と kafka.fav の 2 関数にコメントを追記 |
| [MED] `fetch_records` の 1 MiB バイト上限が `max_count` 非依存 | `TODO(v26.x)` コメントを追加 |
| [LOW] `max_count <= 0` のとき `max(1)` で隠蔽していた | `if max_count <= 0 { return Ok("[]") }` に修正 |
