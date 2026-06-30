# v26.6.0 実装計画 — ストリーミング E2E デモ（kinesis → s3）

## 実装方針

- 新規 Cargo 依存・Rust コードは追加しない（既存の kinesis / aws Rune を使用）
- `examples/streaming/kinesis_to_s3.fav` を新規作成する
- `examples/streaming/docker-compose.yml` に `localstack` サービスを**追加**（既存の kafka / elasticsearch サービスは変更しない）
- `site/content/docs/streaming/kinesis-to-s3.mdx` を新規作成

---

## 実装ステップ

### Step 0: 事前確認

```bash
grep 'version = ' fav/Cargo.toml                              # "26.5.0" であること
cat benchmarks/v26.5.0.json                                   # "test_count":2078 であること
cargo test --bin fav 2>&1 | tail -3                           # 2078 件 PASS であること
ls examples/streaming/kinesis_to_s3.fav 2>/dev/null || echo "not found"  # 未存在であること
grep 'localstack' examples/streaming/docker-compose.yml || echo "not found"  # 未存在であること
```

### Step 1: `fav/Cargo.toml` bump（26.5.0 → 26.6.0）

```toml
version = "26.6.0"
```

### Step 2: `examples/streaming/kinesis_to_s3.fav` 新規作成

spec.md §1 の内容を実装。3 ステージ + `seq ArchivePipeline`:

```favnir
import rune "kinesis"
import rune "aws"

// ... (コメント・環境変数説明)

stage FetchClickEvents: Unit -> Result<String, String> !Stream = |_| {
    bind conn <- Kinesis.connect("")
    bind iter <- Kinesis.get_shard_iterator(conn, "clickstream", "shardId-000000000000", "TRIM_HORIZON")
    Kinesis.get_records(conn, iter, 1000)
}

stage SerializeBatch: String -> Result<String, String> !Pure = |records_json| {
    if String.length(records_json) > 2
    then Result.ok(records_json)
    else Result.err("empty batch — skipping")
}

stage UploadToS3: String -> Result<String, String> !AWS = |batch_json| {
    bind key <- Result.ok("archive/clickstream-batch.json")
    bind _   <- S3.put_object("clickstream-archive", key, batch_json)
    Result.ok("archived to s3://clickstream-archive/" ++ key)
}

seq ArchivePipeline = FetchClickEvents |> SerializeBatch |> UploadToS3
```

> **Kinesis Rune の名前空間確認**: `import rune "kinesis"` → `Kinesis.*`（`kinesis.fav` の `public fn connect/get_shard_iterator/get_records` が対応）
> **S3 Rune の名前空間確認**: `import rune "aws"` → `S3.*`（`runes/aws/s3.fav` の `public fn put_object` が対応。既存の `s3_csv_to_parquet.fav` で確認済み）

### Step 3: `examples/streaming/docker-compose.yml` に localstack 追加

既存ファイルを Read してから Edit で `localstack` サービスを末尾に追加する:

```yaml
  localstack:
    image: localstack/localstack:latest
    environment:
      - SERVICES=kinesis,s3
      - AWS_DEFAULT_REGION=us-east-1
      - AWS_ACCESS_KEY_ID=test
      - AWS_SECRET_ACCESS_KEY=test
    ports:
      - "4566:4566"
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:4566/_localstack/health"]
      interval: 10s
      timeout: 5s
      retries: 10
```

> v26.5.0 で作成済みのファイルを **Edit（追記）** する。既存の kafka / elasticsearch サービスは変更しない。

### Step 4: `site/content/docs/streaming/kinesis-to-s3.mdx` 新規作成

既存の `site/content/docs/streaming/kafka-to-elasticsearch.mdx` の形式（見出し構成・コードブロック言語指定）に合わせて作成する。

- パイプライン概要（FetchClickEvents → SerializeBatch → UploadToS3）
- LocalStack 起動手順（`docker compose ... up -d --wait` または healthcheck 完了確認を明示）
- 環境変数一覧（KINESIS_ENDPOINT / AWS_* 系）
- 各ステージ解説
- スコープ外

### Step 5: `CHANGELOG.md` 更新

```markdown
## [v26.6.0] — 2026-06-27 — ストリーミング E2E デモ（kinesis → s3）

### Added
- `examples/streaming/kinesis_to_s3.fav` — Kinesis → S3 クリックイベントアーカイブデモ（FetchClickEvents / SerializeBatch / UploadToS3 + `seq ArchivePipeline`）
- `examples/streaming/docker-compose.yml` に `localstack` サービス追加（Kinesis / S3 ローカルエミュレーション）
- `site/content/docs/streaming/kinesis-to-s3.mdx` — E2E デモドキュメント
```

### Step 6: `benchmarks/v26.6.0.json` 新規作成

```json
{"version":"26.6.0","test_count":2086,"timestamp":"2026-06-27"}
```

### Step 7: `fav/src/driver.rs` に `v266000_tests` 追加

> **前提**: Step 2（`kinesis_to_s3.fav` 作成）と Step 3（docker-compose.yml 更新）が完了していること。
> `include_str!` マクロはコンパイル時にファイルを要求するため、ファイルが存在しない状態でこのステップを実行するとコンパイルエラーになる。

`v265000_tests` の直後に追加（8 件）:

```rust
// ── v266000_tests (v26.6.0) — kinesis → s3 E2E デモ ─────────────────────────
#[cfg(test)]
mod v266000_tests {
    #[test]
    fn kinesis_to_s3_demo_file_exists() {
        let src = include_str!("../../examples/streaming/kinesis_to_s3.fav");
        assert!(!src.is_empty(), "kinesis_to_s3.fav must not be empty");
    }
    #[test]
    fn kinesis_to_s3_demo_has_get_records() {
        let src = include_str!("../../examples/streaming/kinesis_to_s3.fav");
        assert!(src.contains("get_records"), "demo must call get_records");
    }
    #[test]
    fn kinesis_to_s3_demo_has_put_object() {
        let src = include_str!("../../examples/streaming/kinesis_to_s3.fav");
        assert!(src.contains("put_object"), "demo must call S3.put_object");
    }
    #[test]
    fn kinesis_to_s3_demo_has_archive_pipeline() {
        let src = include_str!("../../examples/streaming/kinesis_to_s3.fav");
        assert!(src.contains("ArchivePipeline"), "demo must define ArchivePipeline");
    }
    #[test]
    fn kinesis_to_s3_demo_has_clickstream() {
        let src = include_str!("../../examples/streaming/kinesis_to_s3.fav");
        assert!(src.contains("clickstream"), "demo must reference clickstream");
    }
    #[test]
    fn kinesis_to_s3_demo_has_s3_bucket() {
        let src = include_str!("../../examples/streaming/kinesis_to_s3.fav");
        assert!(src.contains("clickstream-archive"), "demo must reference clickstream-archive bucket");
    }
    #[test]
    fn streaming_docker_compose_has_localstack() {
        let src = include_str!("../../examples/streaming/docker-compose.yml");
        assert!(src.contains("localstack"), "docker-compose.yml must define localstack service");
    }
    #[test]
    fn changelog_has_v26_6_0() {
        let content = include_str!("../../CHANGELOG.md");
        assert!(content.contains("[v26.6.0]"), "CHANGELOG.md must contain '[v26.6.0]'");
    }
}
```

### Step 8: テスト確認

```bash
cd fav && cargo test v266000 --bin fav          # 8/8 PASS
cd fav && cargo test --bin fav -j 8 -- --test-threads=8 2>&1 | tail -4  # 2086 件 PASS
```

---

## ファイル変更一覧

| ファイル | 操作 |
|---|---|
| `fav/Cargo.toml` | version bump 26.5.0 → 26.6.0 |
| `examples/streaming/kinesis_to_s3.fav` | **新規作成**（3 ステージ + seq） |
| `examples/streaming/docker-compose.yml` | `localstack` サービス追記（Edit） |
| `site/content/docs/streaming/kinesis-to-s3.mdx` | **新規作成** |
| `CHANGELOG.md` | `[v26.6.0]` エントリ先頭に追加 |
| `benchmarks/v26.6.0.json` | **新規作成** |
| `fav/src/driver.rs` | `v266000_tests`（8 件）追加 |

---

## 注意事項

- `Kinesis.get_shard_iterator` の第 4 引数 `iter_type` は `"TRIM_HORIZON"` / `"LATEST"` / `"AT_SEQUENCE_NUMBER"` のいずれか。デモでは `"TRIM_HORIZON"`（最古から読む）を使用する。
- `S3.put_object(bucket, key, body)` — 引数順は `(bucket: String, key: String, body: String)`（`runes/aws/s3.fav` line 7 で確認済み）。
- `String.length` は vm.rs に `"String.length"` primitive として実装済みの標準関数。引数は `(str: String) -> Int`。
- `import rune "aws"` は `runes/aws/` ディレクトリ全体をロードし `S3.*` / `DynamoDB.*` 等の名前空間を提供する（既存の `s3_csv_to_parquet.fav` で確認済み）。
- docker-compose.yml は **Edit（追記）** する。v26.5.0 で作成した kafka・elasticsearch サービスは変更しないこと。

## リスクと対応

| リスク | 対応 |
|---|---|
| `String.length` が存在しない場合 | vm.rs で `"String.length"` を事前確認（既存テストで使用されているはず） |
| `import rune "kinesis"` と `import rune "aws"` の名前空間競合 | `Kinesis.*` と `S3.*` は独立した名前空間のため競合しない |
| docker-compose.yml の Edit で kafka/elasticsearch が壊れる | 末尾への追記のみ。既存サービスは変更しない |
