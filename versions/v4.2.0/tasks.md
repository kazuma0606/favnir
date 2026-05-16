# Favnir v4.2.0 タスクリスト — DB / HTTP / gRPC Rune 2.0

作成日: 2026-05-16

---

## Phase 0: バージョン更新

- [x] `fav/Cargo.toml` の version を `"4.2.0"` に更新
- [x] `fav/src/main.rs` のヘルプ文字列・バージョン表示を `4.2.0` に更新

---

## Phase 1: DB Rune 2.0 — Favnir 側

### 1-A: `runes/db/transaction.fav` 新規作成

- [x] `with_transaction(conn, f)` を実装
  - `DB.begin_tx(conn)` → `f(tx)` → 成功: `DB.commit_tx(tx)` / 失敗: `DB.rollback_tx(tx)`
  - 戻り値型: `Result<List<Map<String, String>>, DbError> !Db`
- [x] `savepoint(conn, name)` を実装（`SAVEPOINT <name>` SQL）
- [x] `release_savepoint(conn, name)` を実装
- [x] `rollback_to_savepoint(conn, name)` を実装

### 1-B: `runes/db/query.fav` 拡張

- [x] `query_one(handle, sql)` を追加
  - `DB.query_raw` → `List.head` → `Some(row)` → `Ok(row)`, `None` → `Err(DbError {...})`
- [x] `paginate(handle, sql, page, size)` を追加
  - `offset = page * size` → `String.concat` で LIMIT/OFFSET を組み立て → `DB.query_raw`
- [x] `batch_insert(handle, sql_template, rows)` を追加
  - `List.fold_left(rows, Ok(0), |acc, params| ...)` で反復 INSERT

### 1-C: `runes/db/migration.fav` 新規作成

- [x] `ensure_migrations_table(conn)` を実装（private、`CREATE TABLE IF NOT EXISTS _fav_migrations ...`）
- [x] `applied_migrations(conn)` を実装（`SELECT name FROM _fav_migrations ORDER BY id ASC`）
- [x] `mark_applied(conn, name)` を実装（`INSERT INTO _fav_migrations`）

### 1-D: `runes/db/db.fav` barrel 更新

- [x] `query_one`, `paginate`, `batch_insert` を use に追加
- [x] `with_transaction`, `savepoint`, `release_savepoint`, `rollback_to_savepoint` を use に追加
- [x] `applied_migrations`, `mark_applied` を use に追加

---

## Phase 2: HTTP Rune 2.0 — VM プリミティブ追加（Rust）

### 2-A: `fav/src/backend/vm.rs` に HTTP VM プリミティブ追加

- [x] `Http.put_raw(url, body, content_type)` を追加
  - `ureq::put(url).content_type(ct).send_string(body)` → `HttpResponse` or `HttpError`
- [x] `Http.delete_raw(url)` を追加
  - `ureq::delete(url).call()` → `HttpResponse` or `HttpError`
- [x] `Http.patch_raw(url, body, content_type)` を追加
  - ureq の `method("PATCH")` または同等の API を使用
- [x] `Http.get_raw_headers(url, headers)` を追加
  - `ureq::get(url)` に Map<String,String> のヘッダーを設定してから `.call()`
- [x] `Http.post_raw_headers(url, body, content_type, headers)` を追加
  - 同様に POST + ヘッダー

### 2-B: `fav/src/middle/checker.rs` に新 HTTP プリミティブのシグネチャ登録

- [x] `check_builtin_apply` の `"Http"` アームに `put_raw`, `delete_raw`, `patch_raw`, `get_raw_headers`, `post_raw_headers` を追加

### 2-C: `String.base64_encode` VM プリミティブ（`auth.basic` 用）

- [x] `String.base64_encode(s)` VM プリミティブを追加（Rust 標準 `base64`、または簡易実装）
  - `fav/Cargo.toml` に `base64 = "0.22"` を追加
  - `vm_call_builtin` の `"String"` アームに追加
- [x] checker.rs に `String.base64_encode: String -> String` を登録

---

## Phase 3: HTTP Rune 2.0 — Favnir 側

### 3-A: `runes/http/client.fav` 新規作成

- [x] `put(url, body)` → `Http.put_raw(url, body, "application/json")`
- [x] `delete(url)` → `Http.delete_raw(url)`
- [x] `patch(url, body)` → `Http.patch_raw(url, body, "application/json")`
- [x] `get_with_headers(url, headers)` → `Http.get_raw_headers(url, headers)`
- [x] `post_with_headers(url, body, headers)` → `Http.post_raw_headers(url, body, "application/json", headers)`

### 3-B: `runes/http/retry.fav` 新規作成

- [x] `with_retry(max_attempts, f)` を実装（内部再帰 `attempt(n)`）
- [x] `retry_get(url, max_attempts)` を実装
- [x] `retry_post(url, body, max_attempts)` を実装

### 3-C: `runes/http/auth.fav` 新規作成

- [x] `bearer(token)` → `Map.set((), "Authorization", "Bearer " + token)`
- [x] `basic(username, password)` → `Map.set((), "Authorization", "Basic " + base64(user:pass))`
- [x] `api_key(key)` → `Map.set((), "X-Api-Key", key)`

### 3-D: `runes/http/http.fav` barrel 更新

- [x] `use client.{ put, delete, patch, get_with_headers, post_with_headers }` を追加
- [x] `use retry.{ with_retry, retry_get, retry_post }` を追加
- [x] `use auth.{ bearer, basic, api_key }` を追加

---

## Phase 4: gRPC フィールド名修正（Rust）

### 4-A: 影響範囲の洗い出し（先行確認）

- [x] `grpc.test.fav` で `field1`/`field2` を使っているテストを特定
- [x] `vm_stdlib_tests.rs` で位置キーを使っているテストを特定
- [x] `driver.rs` の grpc 統合テストで位置キーを使っているものを特定

### 4-B: `fav/src/backend/vm.rs` に `proto_bytes_to_named_map` を追加

- [x] `proto_bytes_to_named_map(bytes, type_name, type_metas)` を実装
  - `type_metas.get(type_name)` でフィールドリストを取得
  - フィールド番号（1-indexed）→ フィールド名に対応させてデコード
  - フィールドが見つからない場合は `"field{n}"` にフォールバック
- [x] `Grpc.decode_raw(type_name, encoded)` のデコード部分を `proto_bytes_to_named_map` に変更

### 4-C: `Grpc.call_typed_raw` 新規 VM プリミティブ追加

- [x] `Grpc.call_typed_raw(response_type, host, method, payload)` を `vm_call_builtin` に追加
  - 既存 `call_raw` のロジックを流用 + `proto_bytes_to_named_map` でデコード

### 4-D: checker.rs 登録

- [x] `Grpc.call_typed_raw` のシグネチャを `check_builtin_apply` の `"Grpc"` アームに追加

### 4-E: `runes/grpc/client.fav` 更新

- [x] `call_typed(response_type, host, method, payload)` を追加

### 4-F: `runes/grpc/grpc.fav` barrel 更新

- [x] `call_typed` を use に追加

### 4-G: 影響テストの修正

- [x] `grpc.test.fav` の decode テストを実フィールド名を期待するよう更新
- [x] `vm_stdlib_tests.rs` の grpc_encode_decode_roundtrip を更新
- [x] `driver.rs` の grpc encode/decode テストを更新

---

## Phase 5: `fav db migrate` CLI

### 5-A: `fav/src/toml.rs` に `DatabaseConfig` 追加

- [x] `DatabaseConfig { url: String, migrations: Option<String> }` 構造体を追加
- [x] `FavToml` に `pub database: Option<DatabaseConfig>` フィールドを追加

### 5-B: `fav/src/main.rs` に CLI サブコマンド追加

- [x] `fav db migrate` を解析して `cmd_db_migrate(db_url, migrations_dir)` を呼ぶ
- [x] `fav db migrate --status` → `cmd_db_migrate_status(db_url, migrations_dir)` を呼ぶ
- [x] `fav db migrate --rollback` → `cmd_db_migrate_rollback(db_url, migrations_dir)` を呼ぶ

### 5-C: `fav/src/driver.rs` に実装

- [x] `cmd_db_migrate(db_url, migrations_dir, dry_run)` を実装
  - rusqlite で `db_url` に接続
  - `_fav_migrations` テーブルを確保
  - `migrations_dir/*.sql` をアルファベット昇順で列挙
  - 未適用のファイルだけを SQL 実行 → `_fav_migrations` に記録
- [x] `cmd_db_migrate_status(db_url, migrations_dir)` を実装
  - 適用済み・未適用を表形式で出力
- [x] `cmd_db_migrate_rollback(db_url, migrations_dir)` を実装
  - `-- @down` セクションをパースして実行（`-- @down` がなければスキップ）

---

## Phase 6: テスト追加

### 6-A: `runes/db/db.test.fav` 拡張

- [x] `test_with_transaction_commit` — コミット後に INSERT が残っている
- [x] `test_with_transaction_rollback_on_fn_err` — Err を返すとロールバック
- [x] `test_paginate_returns_subset` — page=0 size=2 で最初の 2 件だけ返る
- [x] `test_query_one_found` — 1 行を返す
- [x] `test_query_one_not_found_is_err` — 0 行のとき Err
- [x] `test_batch_insert_multiple_rows` — 複数行を一括 INSERT
- [x] `test_migration_mark_applied_and_list` — mark_applied → applied_migrations で確認
- [x] `test_savepoint_rollback` — savepoint → rollback_to_savepoint でロールバック

### 6-B: `runes/http/http.test.fav` 拡張

- [x] `test_retry_get_exhausted_returns_err` — 失敗 URL で n 回リトライ後 Err
- [x] `test_bearer_header_format` — `{ "Authorization": "Bearer abc" }`
- [x] `test_basic_header_format` — `{ "Authorization": "Basic ..." }`
- [x] `test_api_key_header_format` — `{ "X-Api-Key": "my-key" }`
- [x] `test_put_err_on_bad_host` — 接続失敗は Err
- [x] `test_delete_err_on_bad_host` — 接続失敗は Err

### 6-C: `runes/grpc/grpc.test.fav` 拡張

- [x] `test_decode_returns_named_fields` — `grpc.decode("User", encoded)` が `{ "id": ..., "name": ... }` を返す

### 6-D: `vm_stdlib_tests.rs` 追加

- [x] `http_put_raw_returns_err_on_bad_host`
- [x] `http_delete_raw_returns_err_on_bad_host`
- [x] `http_patch_raw_returns_err_on_bad_host`
- [x] `grpc_decode_raw_returns_named_fields`（型メタ登録後にデコード）
- [x] `grpc_call_typed_raw_returns_err_on_bad_host`（接続失敗を確認）

### 6-E: `driver.rs` 統合テスト追加

- [x] `db_migrate_status_no_dir_prints_empty` — `migrations/` ディレクトリが存在しない
- [x] `db_migrate_applies_one_file` — `001_create_users.sql` を適用して確認
- [x] `db_rune_with_transaction_commit_in_source` — inline Favnir source でテスト
- [x] `db_rune_paginate_in_source` — inline Favnir source でテスト
- [x] `http_rune_bearer_header_in_source` — `http.bearer("tok")` → 正しいヘッダー
- [x] `http_rune_retry_exhausted_in_source` — リトライ 1 回後 Err

---

## Phase 7: examples + docs

- [x] `examples/db_demo/main.fav` に `db.with_transaction` / `db.paginate` のデモを追記
- [x] `examples/http_demo/main.fav` に `http.with_retry` / `http.bearer` のデモを追記
- [x] `examples/grpc_e2e_demo/` の `call` を `call_typed` に更新（フィールド名が正しいことを示す）

---

## 完了条件

- [x] `cargo build` が警告なしで通る
- [x] 既存 808 件が全て pass
- [x] 新規テスト 20 件以上が pass
- [x] `db.with_transaction` が commit/rollback ともに正しく動く
- [x] `db.paginate` が正しい LIMIT/OFFSET を発行する
- [x] `http.with_retry` が失敗 URL に対して n 回リトライする
- [x] `http.bearer(token)` が `{ "Authorization": "Bearer <token>" }` を返す
- [x] `grpc.decode("User", encoded)` が `{ "id": ..., "name": ... }` を返す（位置番号でなく）
- [x] `fav db migrate` が SQLite ファイルにマイグレーションを適用できる
