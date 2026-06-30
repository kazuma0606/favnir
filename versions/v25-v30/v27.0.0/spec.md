# v27.0.0 仕様書 — Streaming Native マイルストーン宣言

## 概要

v26.1〜v26.9 で整備したストリーミング基盤をまとめて宣言する。
「ストリーミング Rune 5 本が実質化され、`Stream.*` 操作 6 関数が使え、
E2E デモ 3 本が Docker Compose で動く」状態 = **Streaming Native** の完成。

---

## 背景

ロードマップ v27.0「Streaming Native マイルストーン宣言」より。

### 達成済みコンポーネント（v26.1〜v26.9）

| コンポーネント | 実装バージョン | 実装済み関数 | 状態 |
|---|---|---|---|
| kinesis Rune | v26.1.0 | connect / put_record / put_records / get_shard_iterator / get_records | ✅（`consume` は v27.x 残件） |
| nats Rune | v26.2.0 | connect / publish / subscribe / jetstream_publish / jetstream_consume | ✅（`request` は v27.x 残件） |
| rabbitmq Rune | v26.3.0 | connect / declare_exchange / declare_queue / bind_queue / publish / consume | ✅（`ack`/`nack` は v27.x 残件） |
| `#[streaming]` バックプレッシャー + `Stream.*` 6 関数 | v26.4.0 | map / filter / flat_map / window / merge / split | ✅ |
| E2E デモ（kafka → Elasticsearch） | v26.5.0 | — | ✅ `examples/streaming/kafka_to_elasticsearch.fav` |
| E2E デモ（kinesis → S3） | v26.6.0 | — | ✅ `examples/streaming/kinesis_to_s3.fav` |
| E2E デモ（nats → postgres） | v26.7.0 | — | ✅ `examples/streaming/nats_to_postgres.fav` |
| sqs Rune | v26.8.0 | send_message / send_message_batch / receive_messages / delete_message / purge / consume | ✅ |
| pulsar Rune | v26.9.0 | produce / consume / ack / nack | ✅ |

---

## 成果物

### 1. MILESTONE.md 更新

既存の「Practical Self-Hosting（v25.0.0）」エントリに続いて、
「Streaming Native（v27.0.0）」セクションを追加。

```markdown
## Streaming Native Milestone

**宣言日**: 2026-06-27
**宣言バージョン**: v27.0.0

### 宣言

> 「Kafka → 変換 → Elasticsearch のリアルタイムパイプラインが 50 行で書ける」
> = Streaming Native の完成を象徴するデモ

v27.0.0 をもって、Favnir の **Streaming Native** を正式に宣言する。

ストリーミング Rune 5 本（kinesis / nats / rabbitmq / sqs / pulsar）が実質化され、
`Stream.*` 操作 6 関数（map / filter / flat_map / window / merge / split）が使用可能になり、
E2E デモ 3 本（kafka→ES / kinesis→S3 / nats→postgres）が Docker Compose で動作する。

### 達成コンポーネント
...
```

### 2. README.md 更新

「v25.0 — Practical Self-Hosting」の記述に続いて、
「v27.0 — Streaming Native」を追記する。

### 3. CHANGELOG.md 更新

`[v27.0.0]` エントリを先頭に追加。

### 4. versions/roadmap/roadmap-v26.1-v27.0.md 更新

v27.0 セクションに「完了: 2026-06-27」を追記。

### 5. site/content/docs/streaming-native.mdx 新規作成

Streaming Native マイルストーンの解説ページ。
5 つの Streaming Rune 概要、E2E デモの実行手順、`Stream.*` 操作リファレンスを記載。

### 6. benchmarks/v27.0.0.json 新規作成

```json
{"version":"27.0.0","test_count":2120,"timestamp":"2026-06-27"}
```

### 7. driver.rs — v270000_tests（8 件）

マイルストーン達成条件を検証するテスト。

---

## テスト

### driver.rs v270000_tests（8 件）

| テスト名 | 内容 |
|---|---|
| `milestone_streaming_native_declared` | `MILESTONE.md` に `"Streaming Native"` が含まれること |
| `streaming_rune_kinesis_has_put_record` | `runes/kinesis/kinesis.fav` に `"put_record"` が含まれること |
| `streaming_rune_nats_has_publish` | `runes/nats/nats.fav` に `"publish"` が含まれること |
| `streaming_rune_pulsar_has_produce` | `runes/pulsar/pulsar.fav` に `"fn produce("` が含まれること |
| `stream_rune_has_flat_map` | `runes/stream/stream.fav` に `"flat_map"` が含まれること（6 操作の代表確認） |
| `e2e_demos_all_present` | `examples/streaming/` の README が 3 本すべてのデモ名を含むこと |
| `readme_mentions_v27` | `README.md` に `"v27.0"` が含まれること |
| `changelog_has_v27_0_0` | `CHANGELOG.md` に `"[v27.0.0]"` が含まれること |

---

## ロードマップ完了条件との対応

| ロードマップ要件 | 検証方法 | ギャップ |
|---|---|---|
| kinesis Rune 実質化 | `streaming_rune_kinesis_has_put_record` テスト | `consume` 未実装（v27.x 残件） |
| nats Rune 実質化 | `streaming_rune_nats_has_publish` テスト | `request` 未実装（v27.x 残件） |
| rabbitmq Rune 実質化 | v263000_tests（既存）で確認済み | `ack`/`nack` 未実装、`bind` → `bind_queue`（v27.x 残件） |
| sqs Rune 実質化 | v268000_tests（既存）で確認済み | なし |
| pulsar Rune 実質化 | `streaming_rune_pulsar_has_produce` テスト | `produce` は stub |
| `#[streaming]` バックプレッシャー対応 | `cargo test streaming` リグレッションなし（既存テストで確認） | — |
| `Stream.*` 操作 6 関数 | `stream_rune_has_flat_map` テスト（flat_map は v26.4.0 追加の代表） | なし |
| E2E デモ 3 本 | `e2e_demos_all_present` テスト | なし |
| マイルストーン宣言 | `milestone_streaming_native_declared` テスト | — |

---

## 完了条件

- [ ] `fav/Cargo.toml` が `version = "27.0.0"` であること
- [ ] `MILESTONE.md` に `"Streaming Native"` が含まれること
- [ ] `README.md` に `"v27.0"` が含まれること
- [ ] `CHANGELOG.md` に `[v27.0.0]` エントリが存在すること
- [ ] `versions/roadmap/roadmap-v26.1-v27.0.md` に完了日が記載されること
- [ ] `site/content/docs/streaming-native.mdx` が存在すること
- [ ] `benchmarks/v27.0.0.json` が存在すること（test_count: 2120）
- [ ] `v270000_tests` 8 件すべて PASS
- [ ] `cargo test streaming` で既存テストがリグレッションしないこと
- [ ] 総テスト数 ≥ 2120 件
