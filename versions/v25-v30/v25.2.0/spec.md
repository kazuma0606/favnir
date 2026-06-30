# v25.2.0 仕様書 — s3 Rune 実質化

## 概要

| 項目 | 内容 |
|---|---|
| バージョン | v25.2.0 |
| フェーズ | Rune Foundation（v25.1〜v26.0） |
| テーマ | s3 Rune の「動く Rune」5 条件達成 |
| 依存関係 | v25.1.0（examples/ ディレクトリ構造の確認のため） |
| 目標テスト数 | 1986 件（+6 件 ≥ ロードマップ最小 5 件） |

---

## 背景と目的

v25.1.0 で postgres Rune を実質化した。次は「ほぼすべての ETL が S3 を経由する」という
データエンジニアリングの現実に対応するため、s3 Rune を「動く Rune」の 5 条件を満たすよう実質化する。

既存の `runes/aws/s3.fav` は `get_object` / `put_object` / `delete_object` / `list_objects` /
`bucket_exists` の 5 関数を持つが、ロードマップが要求する `presign_url` と `stream_get` が未実装。
また LocalStack を使った E2E デモも存在しない。

> **ロードマップとの差異**:
> ロードマップ v25.2 節では `presign_url` を「GET / PUT」両対応と記載しているが、
> v25.2.0 では GET のみに限定する（PUT 対応は v25.x 以降）。実装コストの集中を避けるためのスコープ調整。
>
> またロードマップのコード例では `!Io` エフェクトを使用しているが、
> 既存 S3 Rune は `!AWS` を使用する（`runes/aws/s3.fav` 3 行目）。
> `!Io` はロードマップ上の誤記であり、正しくは `!AWS`。

---

## 既存実装の現状

| 関数 | 状態 | 備考 |
|---|---|---|
| `S3.get_object(bucket, key)` | 実装済み | `AWS.s3_get_object_raw` primitive あり |
| `S3.put_object(bucket, key, body)` | 実装済み | `AWS.s3_put_object_raw` primitive あり |
| `S3.delete_object(bucket, key)` | 実装済み | `AWS.s3_delete_object_raw` primitive あり |
| `S3.list_objects(bucket, prefix)` | 実装済み | `AWS.s3_list_objects_raw` primitive あり |
| `S3.bucket_exists(bucket)` | 実装済み | `AWS.s3_head_bucket_raw` primitive あり |
| `S3.presign_url(bucket, key, ttl)` | **未実装** | v25.2.0 で追加 |
| `S3.stream_get(bucket, key)` | **未実装** | v25.2.0 で追加 |

---

## 「動く Rune」5 条件

| # | 条件 | 対象 |
|---|---|---|
| 1 | connect | `AWS_*` 環境変数 or LocalStack エンドポイント経由で接続。`S3.bucket_exists` による接続確認を含む |
| 2 | read | `S3.get_object` / `S3.list_objects` / `S3.stream_get` |
| 3 | write | `S3.put_object` / `S3.delete_object` |
| 4 | error | `Result[T, String]` 統一、エラーメッセージに bucket/key を含む |
| 5 | test | `v252000_tests` 6 件 PASS + `examples/s3_csv_to_parquet.fav` E2E デモ |

---

## 機能仕様

### 1. `S3.presign_url(bucket, key, ttl_secs)` 追加

```
public fn presign_url(bucket: String, key: String, ttl_secs: Int) -> Result<String, String> !AWS {
    AWS.s3_presign_url_raw(bucket, key, ttl_secs)
}
```

- 署名付き URL を生成（GET 操作用。PUT 対応は本バージョン外）
- `ttl_secs`: URL の有効期間（秒）
- 返値: 署名付き HTTPS URL 文字列
- LocalStack: `http://localhost:4566/<bucket>/<key>?X-Amz-Signature=...` 形式
- 実装: vm.rs 既存の自前 SigV4 署名（`sigv4_sign` 等）を流用して URL 生成

### 2. `S3.stream_get(bucket, key)` 追加

```
public fn stream_get(bucket: String, key: String) -> Result<String, String> !AWS {
    AWS.s3_stream_get_raw(bucket, key)
}
```

- 大容量オブジェクトをチャンク分割して取得（将来対応）
- 現バージョンでは `get_object` と同等の動作（`s3_get_object_raw` と同一ロジック）
- 将来: `Stream<Bytes>` への移行予定（その際は別関数として追加し破壊的変更なし）

### 3. `examples/s3_csv_to_parquet.fav` E2E デモ

E2E デモは AWS Rune 経由で実装する（`import rune "aws"`）:

```favnir
import rune "aws"

stage DownloadCsv: Unit -> List<Row> !AWS = |_| {
    bind csv <- S3.get_object(bucket, "input/data.csv")
    Csv.decode<Row>(csv)
}

stage UploadParquet: List<Row> -> Unit !AWS = |rows| {
    bind bytes <- Parquet.encode(rows)
    S3.put_object(bucket, "output/result.parquet", bytes)
}
```

> **注意**: `import rune "s3"` は `runes/s3/` の空スタブを指すため使用しない。
> AWS Rune 経由（`import rune "aws"`）で `runes/aws/s3.fav` の実装を使用する。

---

## エラーコード

| コード | 名前 | 説明 |
|---|---|---|
| E0301 | EffectMismatch | `!AWS` エフェクトなしで AWS 系 Rune を呼び出した場合（既存） |

---

## やらないこと（スコープ外）

- マルチパートアップロード（大容量バイナリ向け）
- S3 バケット作成 / 削除 / ポリシー設定
- S3 イベント通知の Favnir 側受信
- `S3.presign_url` の PUT 操作対応（ロードマップ記載あり → v25.x 以降に延期）
- ストリーミングの真のチャンク分割実装（`Stream<Bytes>` は型システム未定義）
- `aws-sdk-s3` / `aws-presigning` クレートの新規追加（既存自前 SigV4 で対応）

---

## 完了条件

| # | 条件 |
|---|---|
| 1 | `S3.presign_url` が `runes/aws/s3.fav` に実装済み |
| 2 | `S3.stream_get` が `runes/aws/s3.fav` に実装済み |
| 3 | `AWS.s3_presign_url_raw` VM primitive が `fav/src/backend/vm.rs` に存在する |
| 4 | `AWS.s3_stream_get_raw` VM primitive が `fav/src/backend/vm.rs` に存在する |
| 5 | `examples/s3_csv_to_parquet.fav` が存在し `import rune "aws"` + `get_object` / `put_object` を含む |
| 6 | `CHANGELOG.md` に `[v25.2.0]` エントリが存在する |
| 7 | `cargo test` で v252000_tests 6 件すべて PASS |
| 8 | 総テスト数 ≥ 1986 件 |

---

## 検証コマンド

```bash
cd fav && cargo test v252000 -- --test-threads=1
```
