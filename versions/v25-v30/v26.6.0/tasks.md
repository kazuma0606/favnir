# v26.6.0 タスクリスト — ストリーミング E2E デモ（kinesis → s3）

**状態**: COMPLETE
**開始日**: 2026-06-27
**完了日**: 2026-06-27

---

## タスク

| ID | タスク | 状態 |
|---|---|---|
| T0 | 事前確認: `Cargo.toml` が `26.5.0`、テスト数 2078 件、`kinesis_to_s3.fav` 未存在、docker-compose.yml に `localstack` がないことを確認 | [x] |
| T1 | `fav/Cargo.toml` を `version = "26.6.0"` に bump | [x] |
| T2 | `examples/streaming/kinesis_to_s3.fav` 新規作成（FetchClickEvents / SerializeBatch / UploadToS3 + `seq ArchivePipeline`） | [x] |
| T3 | `examples/streaming/docker-compose.yml` を Edit: `localstack` サービスを末尾に追記（kafka / elasticsearch は変更しない） | [x] |
| T4 | `site/content/docs/streaming/kinesis-to-s3.mdx` 新規作成 | [x] |
| T5 | `CHANGELOG.md` 更新: 先頭に `[v26.6.0]` エントリ追加 | [x] |
| T6 | `benchmarks/v26.6.0.json` 新規作成（test_count: 2086） | [x] |
| T7 | `fav/src/driver.rs` 更新: `v266000_tests`（8 件）を `v265000_tests` の直後に追加 | [x] |
| T7.5 | `cargo test v266000 --bin fav` — 8/8 PASS 確認 | [x] |
| T8 | `cargo test --bin fav` — 2086 件 PASS 確認（リグレッションなし） | [x] |
| T9 | spec-reviewer レビュー実施（実装前・本タスクで完了済み） | [x] |

---

## チェックリスト（完了条件）

- [x] `fav/Cargo.toml` が `version = "26.6.0"` であること
- [x] `examples/streaming/kinesis_to_s3.fav` が存在すること
- [x] デモに `seq ArchivePipeline` が含まれること
- [x] デモに `stage FetchClickEvents` が含まれること
- [x] デモに `stage SerializeBatch` が含まれること
- [x] デモに `stage UploadToS3` が含まれること
- [x] デモに `Kinesis.get_records` 呼び出しが含まれること
- [x] デモに `S3.put_object` 呼び出しが含まれること
- [x] デモに `"clickstream-archive"` が含まれること
- [x] `examples/streaming/docker-compose.yml` に `localstack` サービスが追加されていること
- [x] `site/content/docs/streaming/kinesis-to-s3.mdx` が存在すること
- [x] `CHANGELOG.md` に `[v26.6.0]` エントリが存在すること
- [x] `benchmarks/v26.6.0.json` が存在すること（test_count: 2086）
- [x] `v266000_tests` 8 件すべて PASS
- [x] 総テスト数 ≥ 2086 件

---

## メモ

### Kinesis Rune の API（`runes/kinesis/kinesis.fav` より）

```
connect(endpoint: String) -> Result<KinesisConn, String> !Stream
get_shard_iterator(conn: KinesisConn, stream: String, shard_id: String, iter_type: String) -> Result<ShardIterator, String> !Stream
get_records(conn: KinesisConn, iterator: ShardIterator, limit: Int) -> Result<String, String> !Stream
```

### S3 Rune の API（`runes/aws/s3.fav` より）

```
put_object(bucket: String, key: String, body: String) -> Result<Unit, String> !AWS
```
`import rune "aws"` → `S3.put_object(...)` として使用（`s3_csv_to_parquet.fav` で確認済み）。

### `include_str!` パス（`fav/src/driver.rs` 基準）

```rust
include_str!("../../examples/streaming/kinesis_to_s3.fav")
include_str!("../../examples/streaming/docker-compose.yml")
include_str!("../../CHANGELOG.md")
```

`fav/src/driver.rs` から `../` で `fav/`、さらに `../` でプロジェクトルート（`favnir/`）に出る。

### docker-compose.yml の Edit 方針

v26.5.0 で作成したファイルに `localstack` サービスを **末尾追記**した。
kafka・elasticsearch サービスは変更しなかった。

---

## コードレビュー指摘（実装後に記入）

| 指摘 | 対応 |
|---|---|
| [HIGH] `import rune "aws"` が `S3.*` を提供するか未検証 | `runes/aws/aws.fav` に `use s3.*` があることを確認 → 問題なし |
| [MED] LocalStack `latest` タグは再現性リスク・v3 で `SERVICES` 廃止 | `localstack/localstack:3.4` に固定 |
| [LOW] MDX パイプライン構成図 `UploadToS3` 型表記 `String → String` が不正確 | `(String)` に統一 |
| [MED] `Kinesis.connect("")` 空文字列フォールバック保証 | 既存 `Kafka.connect("")` と同じ慣用パターン（VM primitive が env var フォールバック実装済み）— 変更なし |
| [LOW] 実行手順にリソース作成手順欠落 | kafka-to-elasticsearch.mdx と同スタイル（スコープ外として省略）— 変更なし |
| [LOW] `bind key <- Result.ok(...)` コメントの一貫性 | 教育目的のデモとして有益 — 変更なし |
