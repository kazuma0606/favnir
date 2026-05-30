# Favnir v6.8.0 Plan — Rune エコシステム補完

作成日: 2026-05-27

## 実装方針

すべてドキュメント追加のみ。Rune の実装コードは変更しない。

## Phase 順序

```
A. duckdb.mdx 更新（Parquet/CSV IO 追記）
B. db.mdx 新規作成（connection/query/transaction/paginate/batch_insert）
C. http.mdx 新規作成（get/post/put/delete/retry/auth）
D. 最終確認
```

## Phase A — duckdb.mdx 更新

`runes/duckdb/io.fav` に定義された以下の関数を既存 duckdb.mdx に追記:
- `duckdb.read_parquet(conn, path)` → `List<Map<String,String>>`
- `duckdb.read_csv(conn, path)` → `List<Map<String,String>>`
- `duckdb.write_parquet(conn, sql, path)` → `Int`
- `duckdb.write_csv(conn, sql, path)` → `Int`
- `duckdb.query_one(conn, sql)` → `Result<Map<String,String>, DbError>`
- `duckdb.explain(conn, sql)` → クエリ実行計画

## Phase B — db.mdx 新規作成

`runes/db/` の API をドキュメント化:
- `db.connect(url)` / `db.close(handle)` — 接続管理
- `db.query` / `db.execute` / `db.query_one` / `db.query_params` / `db.execute_params`
- `db.paginate(handle, sql, page, size)` — LIMIT/OFFSET ページネーション
- `db.batch_insert(handle, template, rows)` — 一括挿入
- `db.with_transaction(handle, fn)` — トランザクション（begin/commit/rollback）
- `db.savepoint` / `db.release_savepoint` / `db.rollback_to_savepoint`

## Phase C — http.mdx 新規作成

`runes/http/` の API をドキュメント化:
- `http.get(url)` / `http.post(url, body)` / `http.post_json(url, body)` / `http.get_body(url)`
- `http.put` / `http.delete` / `http.patch` / `http.get_with_headers` / `http.post_with_headers`
- `http.with_retry(max_attempts, fn)` — リトライ付きリクエスト
- `http.bearer(token)` / `http.basic(user, pass)` / `http.api_key(key)` — 認証ヘルパー

## Phase D — 最終確認

- コード例を目視確認（有効な Favnir 構文）
- tasks.md を完了状態に更新
