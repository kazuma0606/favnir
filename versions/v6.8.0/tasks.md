# Favnir v6.8.0 Tasks

Date: 2026-05-27

## Goal

既知の Rune ドキュメント不備を解消する。
`db` / `http` の docs 新規作成、`duckdb.mdx` に Parquet/CSV IO を追記。

## Phase A — duckdb.mdx 更新

- [x] A-1: `duckdb.mdx` に `## Parquet / CSV ファイル操作` セクションを追記（read_parquet / read_csv / write_parquet / write_csv）
- [x] A-2: `duckdb.mdx` に `duckdb.query_one` と `duckdb.explain` の説明を追記
- [x] A-3: `duckdb.mdx` の S3 統合コード例が有効な Favnir 構文であることを確認

## Phase B — db.mdx 新規作成

- [x] B-1: `site/content/docs/runes/db.mdx` を新規作成（接続管理セクション: connect / close）
- [x] B-2: db.mdx にクエリセクションを追加（query / execute / query_one / query_params / execute_params）
- [x] B-3: db.mdx に paginate セクションを追加（page/size による LIMIT/OFFSET）
- [x] B-4: db.mdx に batch_insert セクションを追加
- [x] B-5: db.mdx に with_transaction セクションを追加（begin/commit/rollback の自動管理）
- [x] B-6: db.mdx に savepoint セクションを追加

## Phase C — http.mdx 新規作成

- [x] C-1: `site/content/docs/runes/http.mdx` を新規作成（基本 HTTP メソッド: get / post / post_json / get_body）
- [x] C-2: http.mdx に拡張クライアントセクションを追加（put / delete / patch / get_with_headers / post_with_headers）
- [x] C-3: http.mdx に with_retry セクションを追加
- [x] C-4: http.mdx に認証ヘルパーセクションを追加（bearer / basic / api_key）

## Phase D — 最終確認

- [x] D-1: 全コード例を目視確認（有効な Favnir 構文）
- [x] D-2: このファイルを完了状態に更新

## 完了条件まとめ

- `duckdb.mdx` に Parquet/CSV IO セクション追加（query_one/explain も追記） ✓
- `db.mdx` 新規作成（connection / query / paginate / batch_insert / transaction / savepoint） ✓
- `http.mdx` 新規作成（get/post/put/delete/patch / retry / bearer/basic/api_key） ✓
