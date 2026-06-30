# v27.0.0 実装計画 — Streaming Native マイルストーン宣言

## 前提確認

- `fav/Cargo.toml`: `version = "26.9.0"`
- テスト数: 2112 件
- ストリーミング Rune 5 本（kinesis / nats / rabbitmq / sqs / pulsar）が実装済みであること
- E2E デモ 3 本が `examples/streaming/` に存在すること
- `runes/stream/stream.fav` に 6 関数があること

---

## 実装ステップ

### Step 1: Cargo.toml バージョン bump

`fav/Cargo.toml` の `version` を `"26.9.0"` → `"27.0.0"` に変更。

---

### Step 2: MILESTONE.md 更新

既存の `MILESTONE.md` に "Streaming Native Milestone" セクションを追加。
v25.0.0「Practical Self-Hosting」の後に追記する。

追加内容:

```markdown
---

## Streaming Native Milestone

**宣言日**: 2026-06-27
**宣言バージョン**: v27.0.0

### 宣言

> 「Kafka → 変換 → Elasticsearch のリアルタイムパイプラインが 50 行で書ける」
> = Streaming Native の完成を象徴するデモ

v27.0.0 をもって、Favnir の **Streaming Native** を正式に宣言する。

### 達成コンポーネント（v26.1〜v26.9）

| コンポーネント | バージョン |
|---|---|
| kinesis Rune（connect / put_record / put_records / get_shard_iterator / get_records） | v26.1.0 |
| nats Rune（connect / publish / subscribe / jetstream_publish / jetstream_consume） | v26.2.0 |
| rabbitmq Rune（connect / declare_exchange / declare_queue / bind_queue / publish / consume） | v26.3.0 |
| Stream.* 操作 6 関数（map / filter / flat_map / window / merge / split） | v26.4.0 |
| E2E デモ: kafka → Elasticsearch | v26.5.0 |
| E2E デモ: kinesis → S3 | v26.6.0 |
| E2E デモ: nats → postgres | v26.7.0 |
| sqs Rune（send_message / send_message_batch / receive_messages / delete_message / purge / consume） | v26.8.0 |
| pulsar Rune（produce / consume / ack / nack） | v26.9.0 |

### Streaming Native の定義

「ストリーミング Rune 5 本が実質化され、`Stream.*` 操作 6 関数が使え、
E2E デモ 3 本が Docker Compose で動く」状態。

```bash
# Streaming Native 検証コマンド
docker compose -f examples/streaming/docker-compose.yml up -d
fav run examples/streaming/kafka_to_elasticsearch.fav
fav run examples/streaming/kinesis_to_s3.fav
fav run examples/streaming/nats_to_postgres.fav
```
```

---

### Step 3: README.md 更新

既存の `v25.0` 記述に続いて、`v27.0` のマイルストーンを追記する。
検索キーワード「v25.0」または「Practical Self-Hosting」の付近に追記する。

追加例:

```markdown
- **v27.0 — Streaming Native**: ストリーミング Rune 5 本実質化（kinesis / nats / rabbitmq / sqs / pulsar）、`Stream.*` 操作 6 関数、E2E デモ 3 本
```

---

### Step 4: CHANGELOG.md 更新

先頭に `[v27.0.0]` エントリを追加:

```markdown
## [v27.0.0] — 2026-06-27 — Streaming Native マイルストーン宣言

### Milestone
- **Streaming Native** 宣言: ストリーミング Rune 5 本（kinesis / nats / rabbitmq / sqs / pulsar）実質化完了
- `Stream.*` 操作 6 関数（map / filter / flat_map / window / merge / split）使用可能
- E2E デモ 3 本（kafka→ES / kinesis→S3 / nats→postgres）が Docker Compose で動作

### Added
- `MILESTONE.md` に "Streaming Native Milestone" セクション追加
- `site/content/docs/streaming-native.mdx` — Streaming Native マイルストーン解説ページ
- `README.md` に v27.0 マイルストーン記載
- `versions/roadmap/roadmap-v26.1-v27.0.md` に完了日追記
```

---

### Step 5: roadmap-v26.1-v27.0.md 更新

ファイル先頭の Date 行の下に完了日を追記する:

```markdown
Date: 2026-06-24
**完了日**: 2026-06-27（v27.0.0 — Streaming Native 宣言）
```

---

### Step 6: site/content/docs/streaming-native.mdx 新規作成

Streaming Native マイルストーンの解説ページ。以下を含む:
- Streaming Native の定義
- 5 つの Streaming Rune 概要（import 方法・用途）
- `Stream.*` 操作 6 関数リファレンス
- E2E デモの実行手順
- Docker Compose のサービス一覧

---

### Step 7: benchmarks/v27.0.0.json 新規作成

```json
{"version":"27.0.0","test_count":2120,"timestamp":"2026-06-27"}
```

---

### Step 8: driver.rs に v270000_tests 追加

`v269000_tests` の直後に `v270000_tests` モジュール（8 件）を追加。

```rust
// ── v270000_tests (v27.0.0) — Streaming Native マイルストーン宣言 ─────────
#[cfg(test)]
mod v270000_tests {
    #[test]
    fn milestone_streaming_native_declared() {
        let content = include_str!("../../MILESTONE.md");
        assert!(content.contains("Streaming Native"), "MILESTONE.md must declare Streaming Native");
    }
    #[test]
    fn streaming_rune_kinesis_has_put_record() {
        let src = include_str!("../../runes/kinesis/kinesis.fav");
        assert!(src.contains("put_record"), "kinesis rune must implement put_record");
    }
    #[test]
    fn streaming_rune_nats_has_publish() {
        let src = include_str!("../../runes/nats/nats.fav");
        assert!(src.contains("publish"), "nats rune must implement publish");
    }
    #[test]
    fn streaming_rune_pulsar_has_produce() {
        let src = include_str!("../../runes/pulsar/pulsar.fav");
        assert!(src.contains("fn produce("), "pulsar rune must implement produce");
    }
    #[test]
    fn stream_rune_has_flat_map() {
        let src = include_str!("../../runes/stream/stream.fav");
        assert!(src.contains("flat_map"), "stream rune must implement flat_map (v26.4.0)");
    }
    #[test]
    fn e2e_demos_all_present() {
        let readme = include_str!("../../examples/streaming/README.md");
        assert!(
            readme.contains("kafka_to_elasticsearch") &&
            readme.contains("kinesis_to_s3") &&
            readme.contains("nats_to_postgres"),
            "streaming README must reference all 3 E2E demos"
        );
    }
    #[test]
    fn readme_mentions_v27() {
        let content = include_str!("../../README.md");
        assert!(content.contains("v27.0"), "README.md must mention v27.0 milestone");
    }
    #[test]
    fn changelog_has_v27_0_0() {
        let content = include_str!("../../CHANGELOG.md");
        assert!(content.contains("[v27.0.0]"), "CHANGELOG.md must contain '[v27.0.0]'");
    }
}
```

---

## include_str! パス（fav/src/driver.rs 基準）

| パス | 対象 |
|---|---|
| `../../MILESTONE.md` | `favnir/MILESTONE.md` |
| `../../runes/kinesis/kinesis.fav` | `favnir/runes/kinesis/kinesis.fav` |
| `../../runes/nats/nats.fav` | `favnir/runes/nats/nats.fav` |
| `../../runes/pulsar/pulsar.fav` | `favnir/runes/pulsar/pulsar.fav` |
| `../../runes/stream/stream.fav` | `favnir/runes/stream/stream.fav` |
| `../../examples/streaming/README.md` | `favnir/examples/streaming/README.md` |
| `../../README.md` | `favnir/README.md` |
| `../../CHANGELOG.md` | `favnir/CHANGELOG.md` |

---

## 注意事項

### v25.0.0 との類似性

v27.0.0 はマイルストーン宣言であり、新機能の実装はない。
v25.0.0「Practical Self-Hosting」と同じパターンで進める:
- 既存実装の検証
- MILESTONE.md / README.md / CHANGELOG.md の文書更新
- マイルストーン条件を検証するテストの追加

### `cargo test streaming` リグレッション確認

`cargo test streaming --bin fav` を実行し、v26.4.0 で追加した streaming テストが引き続き通ることを確認する（既存テストの保護）。

### roadmap-v26.1-v27.0.md の完了マーカー

`streaming_native_roadmap_complete` テストは追加しない（ファイル内の文字列確認が特定困難なため）。
ロードマップ更新は文書上の記録として行う。
