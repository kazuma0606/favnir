# v26.5.0 タスクリスト — ストリーミング E2E デモ（kafka → elasticsearch）

**状態**: COMPLETE
**開始日**: 2026-06-27
**完了日**: 2026-06-27

---

## タスク

| ID | タスク | 状態 |
|---|---|---|
| T0 | 事前確認: `Cargo.toml` が `26.4.0`、テスト数 2070 件、`examples/streaming/` が未存在であることを確認 | [x] |
| T1 | `fav/Cargo.toml` を `version = "26.5.0"` に bump | [x] |
| T2 | `examples/streaming/kafka_to_elasticsearch.fav` 新規作成（FetchLogs / FilterErrors / IndexToES + `seq LogPipeline`） | [x] |
| T3 | `examples/streaming/docker-compose.yml` 新規作成（kafka: Redpanda / elasticsearch: 8.12.0） | [x] |
| T4 | `site/content/docs/streaming/kafka-to-elasticsearch.mdx` 新規作成 | [x] |
| T5 | `CHANGELOG.md` 更新: 先頭に `[v26.5.0]` エントリ追加 | [x] |
| T6 | `benchmarks/v26.5.0.json` 新規作成（test_count: 2078） | [x] |
| T7 | `fav/src/driver.rs` 更新: `v265000_tests`（8 件）を `v264000_tests` の直後に追加 | [x] |
| T7.5 | `cargo test v265000 --bin fav` — 8/8 PASS 確認 | [x] |
| T8 | `cargo test --bin fav` — 2078 件 PASS 確認（リグレッションなし） | [x] |
| T9 | spec-reviewer レビュー実施（実装前・本タスクで完了済み） | [x] |

---

## チェックリスト（完了条件）

- [x] `fav/Cargo.toml` が `version = "26.5.0"` であること
- [x] `examples/streaming/kafka_to_elasticsearch.fav` が存在すること
- [x] `examples/streaming/kafka_to_elasticsearch.fav` に `seq LogPipeline` が含まれること
- [x] `examples/streaming/kafka_to_elasticsearch.fav` に `stage FetchLogs` が含まれること
- [x] `examples/streaming/kafka_to_elasticsearch.fav` に `stage FilterErrors` が含まれること
- [x] `examples/streaming/kafka_to_elasticsearch.fav` に `stage IndexToES` が含まれること
- [x] `examples/streaming/kafka_to_elasticsearch.fav` に `consume` が含まれること
- [x] `examples/streaming/kafka_to_elasticsearch.fav` に `Elasticsearch.bulk` が含まれること
- [x] `examples/streaming/kafka_to_elasticsearch.fav` に `"logs-index"` が含まれること
- [x] `examples/streaming/docker-compose.yml` が存在すること
- [x] `examples/streaming/docker-compose.yml` に `kafka` サービスが含まれること
- [x] `examples/streaming/docker-compose.yml` に `elasticsearch` サービスが含まれること
- [x] `site/content/docs/streaming/kafka-to-elasticsearch.mdx` が存在すること
- [x] `CHANGELOG.md` に `[v26.5.0]` エントリが存在すること
- [x] `benchmarks/v26.5.0.json` が存在すること（test_count: 2078）
- [x] `v265000_tests` 8 件すべて PASS
- [x] 総テスト数 ≥ 2078 件

---

## メモ

### `import rune` の名前空間ルール

`import rune "kafka"` → `Kafka.*` 名前空間（既存例: `kafka_events_etl.fav` を参照）。
`import rune "elasticsearch"` → `Elasticsearch.*` 名前空間。
2 つの Rune を同一ファイルに import しても名前空間が独立するため衝突しない。

### `include_str!` パス（`fav/src/driver.rs` 基準）

```rust
include_str!("../../examples/streaming/kafka_to_elasticsearch.fav")
include_str!("../../examples/streaming/docker-compose.yml")
```

`fav/src/driver.rs` から `../` で `fav/`、さらに `../` でプロジェクトルート（`favnir/`）に出て、`examples/streaming/` を参照。

### `changelog_has_v26_5_0` テストを除外した理由

8 件のテストに CHANGELOG チェックを含めず、代わりに `kafka_to_es_demo_has_log_pipeline` でデモの存在を確認する。CHANGELOG エントリは T5 で手動追加する（テストのカウント固定化のため）。

---

## コードレビュー指摘（実装後に記入）

| 指摘 | 対応 |
|---|---|
| [HIGH] docker-compose.yml の YAML リスト形式で `--node-id 0` 等のフラグ+値が 1 要素として結合されており Redpanda 起動時に不正フラグエラーが発生する | フラグと値を別リスト要素に分割（`- --node-id` / `- "0"` 等）して修正済み |
| [LOW] `version: "3.8"` は Docker Compose Spec では非推奨 | `version:` 行を削除して修正済み |
