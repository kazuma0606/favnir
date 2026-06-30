# v26.6.0 仕様書 — ストリーミング E2E デモ（kinesis → s3）

## 概要

| 項目 | 内容 |
|---|---|
| バージョン | v26.6.0 |
| フェーズ | Streaming Native（v26.1〜v27.0） |
| テーマ | Kinesis → S3 イベントアーカイブパイプライン E2E デモ |
| 依存関係 | v25.2.0（S3 Rune / LocalStack）・v26.1.0（kinesis Rune）・v26.4.0（Stream.* 操作）完了後 |
| 目標テスト数 | 2086 件（+8 件）|

---

## 背景と目的

v26.5.0 では kafka → elasticsearch の E2E デモを実装した。
v26.6.0 では Streaming Native フェーズ第 2 E2E デモとして、
Kinesis から消費したクリックイベントを S3 にアーカイブするパイプラインを実装する。

「AWS Kinesis → S3 アーカイブ」はデータエンジニアの典型的なログアーカイブパターン。
LocalStack でローカル実行できることを目標とする。

### 利用する Rune

| Rune | import | 名前空間 | 使用関数 | バージョン |
|---|---|---|---|---|
| kinesis | `import rune "kinesis"` | `Kinesis.*` | `connect` / `get_shard_iterator` / `get_records` | v26.1.0 |
| s3（aws Rune に含まれる） | `import rune "aws"` | `S3.*` | `put_object` | v4.11.0（初期導入）/ v25.2.0（LocalStack 対応追加） |

### ロードマップとの API 設計差異

ロードマップ v26.6 節のデモコードは以下の理想 API を示している:

```favnir
Kinesis.consume[ClickEvent]("clickstream", "archive-consumer")
```

v26.6.0 では実際の Kinesis Rune API（`get_shard_iterator` + `get_records`）を使う:

```
FetchClickEvents(Unit -> String)
  |> SerializeBatch(String -> String)
  |> UploadToS3(String -> String)
```

追加の差異:
- ロードマップ v26.1 節の `Kinesis.get_shard_iterator(stream, shard, type)` は 3 引数だが、実際の `kinesis.fav` では `conn` を第 1 引数に加えた 4 引数（`conn, stream, shard_id, iter_type`）になっている。デモは 4 引数版を使用する。
- ロードマップの `BatchEvents`（`#[streaming(window: 30)]`）は省略する。`window` パラメータのキー区切りがロードマップでは `:` だが実際のパーサーは `=` を使用。また型付きタンブリングウィンドウのステージ連携はスタブ段階では不要。代わりに `SerializeBatch` でバッチの存在チェックのみ行う（スコープ外明記: v27.x）
- ロードマップの `SerializeToParquet` は省略する（Parquet ライブラリはバイナリ依存であり v26.x スコープ外）。JSON バッチをそのまま S3 に保存する

---

## 機能仕様

### 1. `examples/streaming/kinesis_to_s3.fav`

```favnir
import rune "kinesis"
import rune "aws"

// ── Kinesis → S3 クリックイベントアーカイブデモ (v26.6.0) ────────────────────
// 前提: docker compose -f examples/streaming/docker-compose.yml up -d
// 実行: fav run examples/streaming/kinesis_to_s3.fav
//
// 環境変数:
//   KINESIS_ENDPOINT      — Kinesis エンドポイント（省略: "http://localhost:4566"）
//   AWS_ACCESS_KEY_ID     — LocalStack では任意値（例: "test"）
//   AWS_SECRET_ACCESS_KEY — LocalStack では任意値（例: "test"）
//   AWS_DEFAULT_REGION    — 省略: "us-east-1"

// 1. Kinesis からクリックイベントを取得（TRIM_HORIZON から最大 1000 件）
stage FetchClickEvents: Unit -> Result<String, String> !Stream = |_| {
    bind conn <- Kinesis.connect("")
    bind iter <- Kinesis.get_shard_iterator(conn, "clickstream", "shardId-000000000000", "TRIM_HORIZON")
    Kinesis.get_records(conn, iter, 1000)
}

// 2. 空バッチをフィルタリング（スタブ: JSON 長さで判定）
stage SerializeBatch: String -> Result<String, String> !Pure = |records_json| {
    if String.length(records_json) > 2
    then Result.ok(records_json)
    else Result.err("empty batch — skipping")
}

// 3. S3 の clickstream-archive バケットにアーカイブ
// Note: `bind key <- Result.ok(...)` は Favnir に let 束縛がないため Result.ok で値を包む慣用パターン
stage UploadToS3: String -> Result<String, String> !AWS = |batch_json| {
    bind key <- Result.ok("archive/clickstream-batch.json")
    bind _   <- S3.put_object("clickstream-archive", key, batch_json)
    Result.ok("archived to s3://clickstream-archive/" ++ key)
}

seq ArchivePipeline = FetchClickEvents |> SerializeBatch |> UploadToS3
```

### 2. `examples/streaming/docker-compose.yml` 更新

v26.5.0 で作成済みの docker-compose.yml に **LocalStack** サービスを追加する。
LocalStack は Kinesis と S3 の両方をローカルでエミュレートする。

追加するサービス:

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

### 3. `site/content/docs/streaming/kinesis-to-s3.mdx`

- パイプライン構成図
- LocalStack 起動手順
- 環境変数一覧
- 各ステージの解説
- スコープ外（Parquet シリアライズ / BatchEvents ウィンドウ / 型付きデシリアライズ）

---

## スコープ外（v27.x 以降）

- `Kinesis.consume[ClickEvent](...)` スタイルの型付き消費ループ
- `#[streaming(window = 30)]` による実際のタンブリングウィンドウとの連携
- Parquet シリアライズ（`SerializeToParquet` ステージ）
- Kinesis Enhanced Fan-Out / KCL 対応
- S3 マルチパートアップロード（大容量バッチ）

---

## Rust テスト（v266000_tests、8 件）

| テスト名 | 内容 | 期待値 |
|---|---|---|
| `kinesis_to_s3_demo_file_exists` | `examples/streaming/kinesis_to_s3.fav` が存在する | assert |
| `kinesis_to_s3_demo_has_get_records` | デモに `get_records` が含まれる | assert |
| `kinesis_to_s3_demo_has_put_object` | デモに `put_object` が含まれる | assert |
| `kinesis_to_s3_demo_has_archive_pipeline` | デモに `ArchivePipeline` が含まれる | assert |
| `kinesis_to_s3_demo_has_clickstream` | デモに `clickstream` が含まれる | assert |
| `kinesis_to_s3_demo_has_s3_bucket` | デモに `clickstream-archive` が含まれる | assert |
| `streaming_docker_compose_has_localstack` | docker-compose.yml に `localstack` が含まれる | assert |
| `changelog_has_v26_6_0` | `CHANGELOG.md` に `[v26.6.0]` が含まれる | assert |

---

## 完了条件

- [ ] `fav/Cargo.toml` が `version = "26.6.0"` であること
- [ ] `examples/streaming/kinesis_to_s3.fav` が存在すること
- [ ] デモに `seq ArchivePipeline` が含まれること
- [ ] デモに `stage FetchClickEvents` が含まれること
- [ ] デモに `stage SerializeBatch` が含まれること
- [ ] デモに `stage UploadToS3` が含まれること
- [ ] デモに `Kinesis.get_records` 呼び出しが含まれること
- [ ] デモに `S3.put_object` 呼び出しが含まれること
- [ ] デモに `"clickstream-archive"` が含まれること
- [ ] `examples/streaming/docker-compose.yml` に `localstack` サービスが追加されていること
- [ ] `site/content/docs/streaming/kinesis-to-s3.mdx` が存在すること
- [ ] `CHANGELOG.md` に `[v26.6.0]` エントリが存在すること
- [ ] `benchmarks/v26.6.0.json` が存在すること（test_count: 2086）
- [ ] `v266000_tests` 8 件すべて PASS
- [ ] 総テスト数 ≥ 2086 件

---

## テスト件数

- v26.5.0 完了時: 2078 件
- v26.6.0 追加: 8 件（v266000_tests）
- **目標**: 2078 + 8 = **2086 件**

> `benchmarks/v26.5.0.json` で `test_count: 2078` を **Step 0 で確認すること**（実装開始前の前提条件）。
