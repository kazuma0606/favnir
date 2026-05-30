# Favnir v6.8.0 Spec — Rune エコシステム補完

作成日: 2026-05-27

## テーマ

既知の Rune 不足・ドキュメント不備を解消する。

## 背景

v6.6.0 時点で Rune の実装は揃っているが、サイトドキュメントは一部の Rune（auth/aws/duckdb/env/gen/log）のみカバーされており、
`db` / `http` といった頻繁に使われる Rune のリファレンスが存在しない。
また duckdb.mdx は `io.fav`（Parquet/CSV IO 関数）を未掲載。

## スコープ

### ドキュメント追加

| Rune | ファイル | 現状 |
|------|---------|------|
| `db` | `runes/db/{connection,query,transaction,migration}.fav` | docs なし |
| `http` | `runes/http/{request,client,retry,auth}.fav` | docs なし |
| `duckdb` | `runes/duckdb/io.fav` | 既存 duckdb.mdx に追記 |

### 動作確認

| 対象 | 確認内容 |
|------|---------|
| `duckdb` S3 統合 | `duckdb` + `aws` の組み合わせコード例が有効な Favnir 構文 |
| `db.with_transaction` | `transaction.fav` の API を確認、ドキュメント化 |
| `db.paginate` | `query.fav` の paginate を確認、ドキュメント化 |
| `http.with_retry` | `retry.fav` の with_retry を確認、ドキュメント化 |

### スコープ外（v6.8.0 では実施しない）

- grpc / json / parquet / csv / incremental / stat の docs（利用頻度が低い、後続バージョンに持ち越し）
- Http.serve<T> の実装追加（未実装機能のため v7.x で検討）
- Rune 実装コードの変更

## 完了条件

- `site/content/docs/runes/db.mdx` 新規作成
- `site/content/docs/runes/http.mdx` 新規作成
- `site/content/docs/runes/duckdb.mdx` に Parquet/CSV IO セクションを追記
- 全コード例が有効な Favnir 構文であること
- `fav check` によるサンプルコードの構文確認
