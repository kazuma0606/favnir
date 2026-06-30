# v26.1.0 仕様書 — kinesis Rune 実質化

## 概要

| 項目 | 内容 |
|---|---|
| バージョン | v26.1.0 |
| フェーズ | Streaming Native（v26.1〜v27.0） |
| テーマ | kinesis Rune の「動く Rune」5 条件達成 |
| 依存関係 | v25.2.0（LocalStack AWS 認証基盤）、v25.7.0（kafka Rune — API 設計参照） |
| 目標テスト数 | 2046 件（+5 件、ロードマップ最小 5 件以上） |

---

## 背景と目的

v26.0.0 時点で kinesis Rune は `runes/kinesis/` ディレクトリも存在せず、未実装。

v26.1.0 では kinesis Rune が「動く Rune の 5 条件」を満たすよう実質化する。
LocalStack（v25.2.0 で整備済みの AWS 認証基盤）を使ってすべての関数を検証する。

### kafka Rune との API 設計対応

| kafka Rune（v25.7.0） | kinesis Rune（v26.1.0） | 備考 |
|---|---|---|
| `KafkaConn(String)` — 名目型ラッパー | `KinesisConn(String)` — 名目型ラッパー | 同パターン |
| `!Stream` エフェクト | `!Stream` エフェクト | 共通 |
| `fn connect(brokers: String)` | `fn connect(endpoint: String)` | 形式統一 |
| `fn produce(conn, topic, key, value)` | `fn put_record(conn, stream, key, data)` | Kinesis API 名に準拠 |

> **ロードマップのシグネチャ差異**: ロードマップ `v26.1` 節では `put_record(stream, key, data)` と `conn` なしで記述されているが、kafka Rune との API 一貫性を優先して `conn` を第 1 引数に追加した。同様に `get_records` はロードマップの `[T]` 型パラメータなし・`limit` 追加とし、戻り値は JSON 文字列（`Result<String, String>`）で統一する（`VMValue::Str` への単純マッピング）。

### 「動く Rune」5 条件

| # | 条件 | 対象 |
|---|---|---|
| 1 | connect | `Kinesis.connect(endpoint)` — KinesisConn（endpoint ラッパー）を返す |
| 2 | read | `Kinesis.get_records(shard, iterator)` — シャードからレコードを取得 |
| 3 | write | `Kinesis.put_record(conn, stream, key, data)` — レコードを送信 |
| 4 | error | `Result<T, String>` 統一、LocalStack 接続失敗時に適切な err を返す |
| 5 | test | `v261000_tests` 5 件 PASS + `runes/kinesis/kinesis.fav` に 5 関数が実装済み |

---

## 機能仕様

### 1. 型定義

`runes/kinesis/kinesis.fav` に定義する型:

```favnir
// エンドポイント URL ラッパー型
// "" の場合: KINESIS_ENDPOINT 環境変数 → "http://localhost:4566"（LocalStack デフォルト）
type KinesisConn(String)

// シャードイテレータ文字列ラッパー型
type ShardIterator(String)

// Kinesis レコード
type KinesisRecord = {
  partition_key: String
  data:          String
  sequence_num:  String
}
```

### 2. 実装する関数（5 件）

| 関数 | シグネチャ | 内容 |
|---|---|---|
| `connect` | `(endpoint: String) -> Result<KinesisConn, String>` | エンドポイントを検証して KinesisConn を返す |
| `put_record` | `(conn: KinesisConn, stream: String, key: String, data: String) -> Result<String, String>` | 1 件送信（シーケンス番号を返す） |
| `put_records` | `(conn: KinesisConn, stream: String, records: List<KinesisRecord>) -> Result<Int, String>` | バッチ送信（成功件数を返す、最大 500 件・10 MB） |
| `get_shard_iterator` | `(conn: KinesisConn, stream: String, shard_id: String, iter_type: String) -> Result<ShardIterator, String>` | イテレータ取得（`"LATEST"` / `"TRIM_HORIZON"` / `"AT_SEQUENCE_NUMBER"`） |
| `get_records` | `(conn: KinesisConn, iterator: ShardIterator, limit: Int) -> Result<String, String>` | レコード取得（最大 `limit` 件、JSON 配列文字列で返す） |

> `get_records` の戻り値は `Result<String, String>`（JSON 配列文字列）。
> `List<KinesisRecord>` への変換は呼び出し元が `Json.decode` 等で行う（kafka の `consume_batch_raw` と同パターン）。
>
> `consume[T]` 継続消費ループ（ロードマップ記載）は v26.1.0 スコープ外。詳細は下記「スコープ外」節を参照。

### 3. VM Primitive（5 件）

`fav/src/backend/vm.rs` に追加:

| primitive 名 | 処理内容 |
|---|---|
| `"Kinesis.connect_raw"` | エンドポイント文字列を検証（`http://` または環境変数）して KinesisConn Record を返す |
| `"Kinesis.put_record_raw"` | KinesisConn + stream + key + data を受け取りレコード送信（AWS SDK / LocalStack 呼び出し） |
| `"Kinesis.put_records_raw"` | KinesisConn + stream + List<KinesisRecord> を受け取りバッチ送信 |
| `"Kinesis.get_shard_iterator_raw"` | KinesisConn + stream + shard_id + iter_type を受け取り ShardIterator 文字列を返す |
| `"Kinesis.get_records_raw"` | KinesisConn + ShardIterator + limit を受け取り JSON 配列文字列を返す |

LocalStack との接続は `KINESIS_ENDPOINT` 環境変数（デフォルト: `http://localhost:4566`）で制御。
AWS 認証情報は `AWS_ACCESS_KEY_ID` / `AWS_SECRET_ACCESS_KEY` 環境変数を使用（LocalStack では任意値で可）。

### 4. `runes/kinesis/kinesis.fav` — Favnir ラッパー

kafka.fav と同じシングルファイルパターン:

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

### 5. `site/content/docs/runes/kinesis.mdx` — 新規作成

5 条件クリア状況・API ドキュメント・LocalStack 実行手順を記載。

---

## エラー処理

すべての primitive は `Result<T, String>` を返す:
- 接続失敗 → `Result.err("Kinesis.connect_raw: ...")`
- レコード送信失敗 → `Result.err("Kinesis.put_record_raw: ...")`
- イテレータ取得失敗 → `Result.err("Kinesis.get_shard_iterator_raw: ...")`
- `limit <= 0` → `Result.err("Kinesis.get_records_raw: limit must be > 0")`

---

## スコープ外（v26.x 以降）

- **`consume[T]` 継続消費ループ**（ロードマップ記載）— `get_shard_iterator + get_records` のポーリングループとして v26.2〜v26.9 のいずれかで実装予定。v26.1.0 では `get_records` 単発呼び出しまでを提供する
- Enhanced Fan-Out（EFO）コンシューマー（専用スループットの HTTP/2 プッシュ配信）
- Server-Side Encryption（KMS 鍵管理）
- `Stream.*` 操作との統合（`v26.4` で実装予定）
- kinesis_to_s3 E2E デモ（`v26.6` で実装予定）
- シャード分割・統合（`SplitShard` / `MergeShards`）

---

## Rust テスト（v261000_tests、6 件）

| テスト名 | 内容 | 期待値 |
|---|---|---|
| `kinesis_rune_has_connect_fn` | `runes/kinesis/kinesis.fav` に `fn connect` が含まれる | assert |
| `kinesis_rune_has_put_record_fn` | `runes/kinesis/kinesis.fav` に `fn put_record` が含まれる | assert |
| `kinesis_rune_has_put_records_fn` | `runes/kinesis/kinesis.fav` に `fn put_records` が含まれる | assert |
| `kinesis_rune_has_get_shard_iterator_fn` | `runes/kinesis/kinesis.fav` に `fn get_shard_iterator` が含まれる | assert |
| `kinesis_rune_has_get_records_fn` | `runes/kinesis/kinesis.fav` に `fn get_records` が含まれる | assert |
| `changelog_has_v26_1_0` | `CHANGELOG.md` に `[v26.1.0]` が含まれる | assert |

---

## 完了条件

- [ ] `fav/Cargo.toml` が `version = "26.1.0"` であること
- [ ] `runes/kinesis/kinesis.fav` が存在し、5 関数（connect / put_record / put_records / get_shard_iterator / get_records）が定義されていること
- [ ] `fav/src/backend/vm.rs` に 5 件の Kinesis primitive（`connect_raw` / `put_record_raw` / `put_records_raw` / `get_shard_iterator_raw` / `get_records_raw`）が追加されていること
- [ ] `site/content/docs/runes/kinesis.mdx` が存在すること
- [ ] `CHANGELOG.md` に `[v26.1.0]` エントリが存在すること
- [ ] `v261000_tests` 6 件すべて PASS
- [ ] 総テスト数 ≥ 2047 件

---

## テスト件数

- v26.0.0 完了時: 2041 件
- v26.1.0 追加: 6 件（v261000_tests）
- **目標**: 2041 + 6 = **2047 件**
