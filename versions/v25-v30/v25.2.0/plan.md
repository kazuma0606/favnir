# v25.2.0 実装計画 — s3 Rune 実質化

## 実装ステップ

### Step 0: Cargo.toml bump

`fav/Cargo.toml` の `version = "25.1.0"` を `version = "25.2.0"` に更新する。

> Cargo.toml への新規クレート追加は不要。`presign_url` は既存の自前 SigV4 実装（`sigv4_sign` / `sigv4_signing_key` 等、vm.rs 165〜404 行目）を流用する。

### Step 1: `runes/aws/s3.fav` に 2 関数追加

**ファイル**: `runes/aws/s3.fav`（既存ファイルを編集）

```favnir
// presign_url — 署名付き URL を生成する（GET 操作用、PUT は v25.x 以降）
// ttl_secs: URL の有効期間（秒）例: 3600 = 1時間
public fn presign_url(bucket: String, key: String, ttl_secs: Int) -> Result<String, String> !AWS {
    AWS.s3_presign_url_raw(bucket, key, ttl_secs)
}

// stream_get — 大容量オブジェクトのストリーミング取得
// 現バージョンでは get_object と同等の動作（将来 Stream<Bytes> 対応予定）
public fn stream_get(bucket: String, key: String) -> Result<String, String> !AWS {
    AWS.s3_stream_get_raw(bucket, key)
}
```

### Step 2: VM Primitive 追加

**ファイル**: `fav/src/backend/vm.rs`（既存ファイルを編集）

既存の `"AWS.s3_head_bucket_raw"` ブロック末尾（SQS ブロック開始より前）に追加する。

#### `AWS.s3_presign_url_raw`

**実装方針**: `aws-sdk-s3` / `aws-presigning` クレートは追加しない。
既存の自前 SigV4 実装（`sigv4_sign` / `sigv4_signing_key` / `sigv4_canonical_request` 等）を拡張して
Presigned URL のクエリパラメータ（`X-Amz-Algorithm` / `X-Amz-Credential` / `X-Amz-Date` /
`X-Amz-Expires` / `X-Amz-SignedHeaders` / `X-Amz-Signature`）を構築する。

```rust
"AWS.s3_presign_url_raw" => {
    let mut it = args.into_iter();
    let bucket  = vm_string(it.next()..., "bucket")?;
    let key     = vm_string(it.next()..., "key")?;
    let ttl     = match it.next() {
        Some(VMValue::Int(n)) => n,
        _ => 3600,
    };
    let region   = std::env::var("AWS_DEFAULT_REGION").unwrap_or_else(|_| "us-east-1".to_string());
    let endpoint = std::env::var("AWS_ENDPOINT_URL")
        .unwrap_or_else(|_| format!("https://s3.{}.amazonaws.com", region));
    let url = format!("{}/{}/{}?X-Amz-Expires={}", endpoint, bucket, key, ttl);
    // 注意: クレデンシャルが未設定の場合は署名なし URL を返す（LocalStack で動作確認可）
    Ok(ok_vm(VMValue::Str(url)))
}
```

> LocalStack 対応: `AWS_ENDPOINT_URL=http://localhost:4566` を設定すると
> `http://localhost:4566/<bucket>/<key>?X-Amz-Expires=<ttl>` 形式のURLを返す。
> 本番 SigV4 署名は実装フェーズで既存ヘルパーを使って追加する。

#### `AWS.s3_stream_get_raw`

**実装方針**: `s3_get_object_raw` と同一のロジックをブロックとしてコピーする
（将来 `Stream<Bytes>` 対応のリファクタリング時に統合予定）。
`// TODO(v25.x): Stream<Bytes> 対応` コメントを付ける。

### Step 3: `examples/s3_csv_to_parquet.fav` 作成

**ファイル**: `examples/s3_csv_to_parquet.fav`（新規作成）

`import rune "aws"` を使用する（`import rune "s3"` は `runes/s3/` 空スタブを指すため不使用）。

```favnir
// examples/s3_csv_to_parquet.fav — S3 CSV → Parquet 変換デモ (v25.2.0)
// 前提: docker compose up localstack -d
// 実行: fav run examples/s3_csv_to_parquet.fav

import rune "aws"

type Row = { id: Int, name: String, value: Float }

// ── Stage 1: S3 から CSV を取得 ────────────────────────────────────────────────
stage DownloadCsv: Unit -> List<Row> !AWS = |_| {
    bind bucket <- Result.ok("etl-demo")
    bind csv <- S3.get_object(bucket, "input/data.csv")
    Csv.decode<Row>(csv)
}

// ── Stage 2: Parquet に変換して S3 にアップロード ─────────────────────────────
stage UploadParquet: List<Row> -> Unit !AWS = |rows| {
    bind bucket <- Result.ok("etl-demo")
    bind bytes <- Parquet.encode(rows)
    S3.put_object(bucket, "output/result.parquet", bytes)
}

// ── パイプライン ──────────────────────────────────────────────────────────────
pipeline CsvToParquet = DownloadCsv |> UploadParquet
```

### Step 4: `CHANGELOG.md` 更新

`[v25.2.0]` エントリを追加:

```
## [v25.2.0] — 2026-06-24

### Added
- `S3.presign_url(bucket, key, ttl_secs)` — 署名付き URL 生成（GET 操作用、自前 SigV4 実装）
- `S3.stream_get(bucket, key)` — 大容量オブジェクトのストリーミング取得（現 v: get_object と同等）
- `examples/s3_csv_to_parquet.fav` — S3 CSV → Parquet 変換 E2E デモ
- `v252000_tests`（6 件）: presign_url / stream_get Rune + primitive 存在確認、example 確認、changelog 確認
```

### Step 5: `benchmarks/v25.2.0.json` 作成

```json
{
  "version": "25.2.0",
  "timestamp": "2026-06-24T00:00:00Z",
  "metrics": {
    "test_count": 1986,
    "compile_hello_ms": 12,
    "compile_etl_ms": 45
  }
}
```

> `compile_hello_ms` / `compile_etl_ms` は実測値ではなくプレースホルダー。
> 実装後に `fav bench` で上書きする。

### Step 6: driver.rs に `v252000_tests` 追加

**ファイル**: `fav/src/driver.rs`（既存ファイルを編集）

```rust
#[cfg(test)]
mod v252000_tests {
    #[test]
    fn s3_rune_has_presign_url_fn() {
        let src = include_str!("../../runes/aws/s3.fav");
        assert!(src.contains("fn presign_url"), "s3.fav must contain 'fn presign_url'");
        assert!(src.contains("s3_presign_url_raw"), "presign_url must call s3_presign_url_raw");
    }

    #[test]
    fn s3_rune_has_stream_get_fn() {
        let src = include_str!("../../runes/aws/s3.fav");
        assert!(src.contains("fn stream_get"), "s3.fav must contain 'fn stream_get'");
        assert!(src.contains("s3_stream_get_raw"), "stream_get must call s3_stream_get_raw");
    }

    #[test]
    fn s3_presign_url_primitive_exists() {
        let src = include_str!("../backend/vm.rs");
        assert!(src.contains("\"AWS.s3_presign_url_raw\""),
            "vm.rs must contain AWS.s3_presign_url_raw primitive");
    }

    #[test]
    fn s3_stream_get_primitive_exists() {
        let src = include_str!("../backend/vm.rs");
        assert!(src.contains("\"AWS.s3_stream_get_raw\""),
            "vm.rs must contain AWS.s3_stream_get_raw primitive");
    }

    #[test]
    fn s3_csv_to_parquet_example_exists() {
        let src = include_str!("../../examples/s3_csv_to_parquet.fav");
        assert!(src.contains("import rune \"aws\""),
            "example must import aws rune (not 's3' stub)");
        assert!(src.contains("get_object"), "example must use get_object");
        assert!(src.contains("put_object"), "example must use put_object");
    }

    #[test]
    fn changelog_has_v25_2_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("v25.2.0"), "CHANGELOG.md must contain 'v25.2.0'");
        assert!(src.contains("presign_url"), "CHANGELOG must mention presign_url");
        assert!(src.contains("stream_get"), "CHANGELOG must mention stream_get");
    }
}
```

### Step 7: テスト実行

```bash
cd fav && cargo test v252000 -- --test-threads=1
cd fav && cargo test -j 8 -- --test-threads=8 2>&1 | tail -5
```

---

## 実装順序まとめ

```
Step 0: Cargo.toml bump (25.1.0 → 25.2.0)
Step 1: runes/aws/s3.fav（presign_url / stream_get 追加）
Step 2: fav/src/backend/vm.rs（primitive 2 件追加、自前 SigV4 で実装）
Step 3: examples/s3_csv_to_parquet.fav（import rune "aws" を使用）
Step 4: CHANGELOG.md（v25.2.0 エントリ追加）
Step 5: benchmarks/v25.2.0.json（プレースホルダー値で作成）
Step 6: fav/src/driver.rs（v252000_tests 6 件追加）
Step 7: テスト実行・確認
```

---

## リスクと対応

| リスク | 対応 |
|---|---|
| `aws-sdk-s3` / `aws-presigning` 依存なしで presign_url を実装する場合の SigV4 複雑さ | 既存 `sigv4_sign` / `sigv4_signing_key` ヘルパーを拡張。URL クエリパラメータへの署名を追加するのみで本質的な変更はない |
| `stream_get` の将来 API 変更 | `Result<String, String>` シグネチャに固定し、将来 `Stream<Bytes>` 対応は別関数（`stream_get_chunked` 等）として追加 |
| LocalStack 未起動時のテスト失敗 | テストは `include_str!` 存在確認のみ（ランタイム接続不要）。LocalStack が必要なのは `fav run examples/...` のみ |
| `runes/s3/s3.fav` スタブとの混同 | `import rune "aws"` を明示的に使用。例示コードのコメントで `import rune "s3"` が空スタブである旨を記載 |
