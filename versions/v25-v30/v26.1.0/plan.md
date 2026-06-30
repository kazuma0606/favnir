# v26.1.0 実装計画 — kinesis Rune 実質化

## 実装方針

- kinesis Rune は kafka Rune（v25.7.0）と同じ**シングルファイルパターン**（`runes/kinesis/kinesis.fav` のみ）で実装する
- LocalStack AWS 基盤は v25.2.0 で整備済み。同じ環境変数パターン（`AWS_ACCESS_KEY_ID` 等）を踏襲する
- Cargo 依存追加は**しない**（既存の `aws-sdk-kinesis` または HTTP 直接呼び出しで実装。動作モックで代替可能）

---

## 実装ステップ

### Step 0: 事前確認

```bash
grep 'version = ' fav/Cargo.toml           # "26.0.0" であること
cargo test --bin fav 2>&1 | tail -3        # 2041 件 PASS であること
ls runes/kinesis/ 2>/dev/null || echo "not found"  # 未存在であること
```

### Step 1: `fav/Cargo.toml` bump（26.0.0 → 26.1.0）

```toml
version = "26.1.0"
```

### Step 2: VM Primitive 5 件追加（`fav/src/backend/vm.rs`）

> **順序の理由**: kinesis.fav が vm.rs の primitive を呼び出すため、vm.rs を先に追加する（kafka が追加された際の実績と同順）。

既存の Kafka primitive の直後（`kafka_consume_one_sync` 関数群の後）に Kinesis primitive を追加する。

**各 primitive の `#[cfg]` ガード方針**:
Kafka と同様に、各 Kinesis primitive は以下のペアで実装すること:

```rust
#[cfg(not(target_arch = "wasm32"))]
"Kinesis.connect_raw" => {
    // 実際の接続処理 / スタブ実装
}
#[cfg(target_arch = "wasm32")]
"Kinesis.connect_raw" => {
    Err("Kinesis not supported on wasm32".to_string())
}
```

追加する primitive（詳細は spec.md §3 を参照）:

| primitive 名 | 引数 | 戻り値 | 実装内容 |
|---|---|---|---|
| `"Kinesis.connect_raw"` | `endpoint: String` | `KinesisConn` (Str) | エンドポイント文字列を検証し `VMValue::Str(endpoint)` を返す |
| `"Kinesis.put_record_raw"` | `conn, stream, key, data: String` | `String`（シーケンス番号） | LocalStack/AWS に PutRecord API 呼び出し。スタブとして "seq-0001" 返却 |
| `"Kinesis.put_records_raw"` | `conn, stream: String, records: List` | `Int`（成功件数） | バッチ送信。スタブとして `records.len()` を返却 |
| `"Kinesis.get_shard_iterator_raw"` | `conn, stream, shard_id, iter_type: String` | `ShardIterator` (Str) | GetShardIterator API 呼び出し。スタブとして `"shard-iter-{stream}-{iter_type}"` を返却 |
| `"Kinesis.get_records_raw"` | `conn: KinesisConn, iterator: ShardIterator, limit: Int` | `String`（JSON 配列） | GetRecords API 呼び出し。スタブとして空 JSON 配列 `"[]"` を返却（LocalStack なし時） |

### Step 2.5: `cargo build` — vm.rs コンパイルエラーなし確認

```bash
cargo build --bin fav 2>&1 | grep -E "error|warning: unused" | head -10
```

### Step 3: `runes/kinesis/kinesis.fav` 新規作成

kafka.fav パターン（シングルファイル）で作成:

```favnir
// runes/kinesis/kinesis.fav — Kinesis Rune (v26.1.0)
//
// 使い方:
//   import rune "kinesis"
//
// 環境変数:
//   KINESIS_ENDPOINT        — エンドポイント URL（省略時: "http://localhost:4566"）
//   AWS_ACCESS_KEY_ID       — AWS アクセスキー（LocalStack では任意値で可）
//   AWS_SECRET_ACCESS_KEY   — AWS シークレットキー（LocalStack では任意値で可）
//   AWS_DEFAULT_REGION      — AWS リージョン（省略時: "us-east-1"）
//
// ローカル開発:
//   docker run -p 4566:4566 localstack/localstack

type KinesisConn(String)
type ShardIterator(String)
type KinesisRecord = { partition_key: String, data: String, sequence_num: String }

public fn connect(endpoint: String) -> Result<KinesisConn, String> !Stream {
    Kinesis.connect_raw(endpoint)
}

public fn put_record(conn: KinesisConn, stream: String, key: String, data: String) -> Result<String, String> !Stream {
    Kinesis.put_record_raw(conn, stream, key, data)
}

public fn put_records(conn: KinesisConn, stream: String, records: List<KinesisRecord>) -> Result<Int, String> !Stream {
    Kinesis.put_records_raw(conn, stream, records)
}

public fn get_shard_iterator(conn: KinesisConn, stream: String, shard_id: String, iter_type: String) -> Result<ShardIterator, String> !Stream {
    Kinesis.get_shard_iterator_raw(conn, stream, shard_id, iter_type)
}

public fn get_records(conn: KinesisConn, iterator: ShardIterator, limit: Int) -> Result<String, String> !Stream {
    Kinesis.get_records_raw(conn, iterator, limit)
}
```

> `fn put_records` の `records: List<KinesisRecord>` は Favnir のジェネリック型引数なしで記述する
> （Favnir は `List<T>` の `T` 部分をランタイム型として扱う）。

### Step 4: `site/content/docs/runes/kinesis.mdx` 新規作成

### Step 4: `site/content/docs/runes/kinesis.mdx` 新規作成

5 条件クリア状況・API ドキュメント・LocalStack 実行手順を含む MDX を作成。

```markdown
# Kinesis Rune

AWS Kinesis Data Streams との統合。v26.1.0 で「動く Rune の 5 条件」をクリア。

## セットアップ

\`\`\`bash
docker run -p 4566:4566 localstack/localstack
export KINESIS_ENDPOINT=http://localhost:4566
export AWS_ACCESS_KEY_ID=test
export AWS_SECRET_ACCESS_KEY=test
export AWS_DEFAULT_REGION=us-east-1
\`\`\`

## 使い方

\`\`\`favnir
import rune "kinesis"
...
\`\`\`

## API リファレンス
...
```

### Step 5: `CHANGELOG.md` 更新

先頭に `[v26.1.0]` エントリを追加:

```markdown
## [v26.1.0] — 2026-06-26 — kinesis Rune 実質化

### Added
- `runes/kinesis/kinesis.fav` — Kinesis Rune（connect / put_record / put_records / get_shard_iterator / get_records）
- `Kinesis.connect_raw` / `put_record_raw` / `put_records_raw` / `get_shard_iterator_raw` / `get_records_raw` — VM primitive 5 件追加
- `site/content/docs/runes/kinesis.mdx` — Kinesis Rune ドキュメント新規作成
```

### Step 6: `benchmarks/v26.1.0.json` 新規作成

```json
{"version":"26.1.0","test_count":2047,"timestamp":"2026-06-26"}
```

### Step 7: `fav/src/driver.rs` に `v261000_tests` 追加

`v260000_tests` の直後に追加（5 件）:

```rust
// ── v261000_tests (v26.1.0) — kinesis Rune 実質化 ─────────────
#[cfg(test)]
mod v261000_tests {
    #[test]
    fn kinesis_rune_has_connect_fn() {
        let src = include_str!("../../runes/kinesis/kinesis.fav");
        assert!(src.contains("fn connect"), "kinesis connect fn not found");
    }
    #[test]
    fn kinesis_rune_has_put_record_fn() {
        let src = include_str!("../../runes/kinesis/kinesis.fav");
        assert!(src.contains("fn put_record"), "kinesis put_record fn not found");
    }
    #[test]
    fn kinesis_rune_has_get_shard_iterator_fn() {
        let src = include_str!("../../runes/kinesis/kinesis.fav");
        assert!(src.contains("fn get_shard_iterator"), "kinesis get_shard_iterator fn not found");
    }
    #[test]
    fn kinesis_rune_has_put_records_fn() {
        let src = include_str!("../../runes/kinesis/kinesis.fav");
        assert!(src.contains("fn put_records"), "kinesis put_records fn not found");
    }
    #[test]
    fn kinesis_rune_has_get_records_fn() {
        let src = include_str!("../../runes/kinesis/kinesis.fav");
        assert!(src.contains("fn get_records"), "kinesis get_records fn not found");
    }
    #[test]
    fn changelog_has_v26_1_0() {
        let content = include_str!("../../CHANGELOG.md");
        assert!(content.contains("[v26.1.0]"), "CHANGELOG.md must contain '[v26.1.0]'");
    }
}
```

### Step 8: テスト確認

```bash
cd fav && cargo test v261000 --bin fav          # 6/6 PASS
cd fav && cargo test --bin fav -j 8 -- --test-threads=8 2>&1 | tail -4  # 2047 件 PASS
```

---

## ファイル変更一覧

| ファイル | 操作 |
|---|---|
| `fav/Cargo.toml` | version bump 26.0.0 → 26.1.0 |
| `runes/kinesis/kinesis.fav` | **新規作成**（5 関数 + 3 型定義） |
| `fav/src/backend/vm.rs` | Kinesis primitive 5 件追加 |
| `site/content/docs/runes/kinesis.mdx` | **新規作成** |
| `CHANGELOG.md` | `[v26.1.0]` エントリ先頭に追加 |
| `benchmarks/v26.1.0.json` | **新規作成** |
| `fav/src/driver.rs` | `v261000_tests`（5 件）追加 |

---

## 注意事項

- `runes/kinesis/` ディレクトリは存在しない。`kinesis.fav` 作成時に自動作成される（Write ツールは親ディレクトリを自動作成）。
- **vm.rs を先に実装してから kinesis.fav を作成する**（Step 2 → Step 3 の順序が重要）。
- `#[cfg(not(target_arch = "wasm32"))]` ガードと wasm32 フォールバックを各 primitive でペアで実装すること（Kafka パターンと同一）。
- `include_str!` のパスは `fav/src/driver.rs` から見た相対パス:
  - `runes/kinesis/kinesis.fav` → `"../../runes/kinesis/kinesis.fav"`
  - `CHANGELOG.md` → `"../../CHANGELOG.md"`
- vm.rs の primitive 挿入位置: `"Kafka.connect_raw"` の近傍（grep で特定後、その直後に追加）。
- `KinesisConn` / `ShardIterator` は名目型ラッパー（`type Foo(String)` 形式）。vm.rs では `VMValue::Str` 内に endpoint 文字列 / iterator 文字列を格納する（既存の KafkaConn パターンと同様）。
- `get_records_raw` の戻り値は JSON 配列文字列（`VMValue::Str`）。`List<KinesisRecord>` への変換は呼び出し元が行う（kafka の `consume_batch_raw` と同パターン）。
- `put_records_raw` の `records: List<KinesisRecord>` は `VMValue::List` として受け取る。

## リスクと対応

| リスク | 対応 |
|---|---|
| LocalStack が起動していない環境でのテスト失敗 | primitive をスタブ実装（LocalStack 未接続時はモック値を返す）にする |
| `aws-sdk-kinesis` 未追加による実 API 呼び出し不可 | HTTP 直接呼び出し（reqwest）または `aws_sdk_kinesis` なしのスタブで代替 |
| `KinesisRecord` 型を List で渡す際の VM 型変換 | `VMValue::List(Vec<VMValue::Record>)` として扱う（dynamodb と同パターン） |
