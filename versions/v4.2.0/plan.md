# Favnir v4.2.0 実装計画 — DB / HTTP / gRPC Rune 2.0

作成日: 2026-05-16

---

## Phase 0: バージョン更新（5 分）

- `fav/Cargo.toml` version を `"4.2.0"` に変更
- `fav/src/main.rs` のヘルプ文字列・バージョン表示を更新

---

## Phase 1: DB Rune 2.0 — Favnir 側

**変更ファイル**: `runes/db/*.fav`
**前提**: `DB.begin_tx`, `DB.commit_tx`, `DB.rollback_tx`, `DB.execute_in_tx` は実装済み

### 1-A: `transaction.fav` 新規作成

- `with_transaction(conn, f)`: `DB.begin_tx` → `f(tx)` → commit/rollback
- `savepoint(conn, name)`: `SAVEPOINT <name>` SQL を直接実行
- `release_savepoint(conn, name)`: `RELEASE SAVEPOINT <name>`
- `rollback_to_savepoint(conn, name)`: `ROLLBACK TO SAVEPOINT <name>`

戻り値型は `Result<List<Map<String, String>>, DbError>` に固定（非ジェネリック）。

### 1-B: `query.fav` 拡張

既存の `query`, `query_params`, `execute`, `execute_params` を残しつつ追加:

- `query_one(handle, sql)`: `DB.query_raw` → `List.head` → `Some/None` → `Result`
- `paginate(handle, sql, page, size)`: `String.concat` で LIMIT/OFFSET を組み立て → `DB.query_raw`
- `batch_insert(handle, sql_template, rows)`: `List.fold_left` でループ INSERT

### 1-C: `migration.fav` 新規作成

- `ensure_migrations_table(conn)`: `CREATE TABLE IF NOT EXISTS _fav_migrations ...`
- `applied_migrations(conn)`: `SELECT name FROM _fav_migrations ORDER BY id ASC`
- `mark_applied(conn, name)`: `INSERT INTO _fav_migrations`

### 1-D: `db.fav` barrel 更新

新関数をすべて export に追加。

---

## Phase 2: HTTP Rune 2.0 — VM プリミティブ追加（Rust）

**変更ファイル**: `fav/src/backend/vm.rs`

### 2-A: 新規 VM プリミティブ

`vm_call_builtin` の `"Http.*"` 分岐に追加:

```
"Http.put_raw"           → ureq::put(url).set_content_type(ct).send_string(body)
"Http.delete_raw"        → ureq::delete(url).call()
"Http.patch_raw"         → ureq::patch(url).set_content_type(ct).send_string(body)
"Http.get_raw_headers"   → ureq::get(url) + headers map + .call()
"Http.post_raw_headers"  → ureq::post(url) + headers map + .send_string(body)
```

戻り値は全て `Result<HttpResponse, HttpError>`（既存パターンに合わせる）。

### 2-B: checker.rs 登録

`check_builtin_apply` の `"Http"` アームに新関数シグネチャを追加:
- `put_raw`, `delete_raw`, `patch_raw`: `Result<HttpResponse, HttpError>`
- `get_raw_headers`, `post_raw_headers`: 同上

### 2-C: `String.base64_encode` VM プリミティブ（オプション）

`auth.basic()` で使用。Rust 側では `base64::encode`（既存依存があれば）または
`data-encoding` クレートで実装。実装困難な場合はスタブとして
`"Basic " + raw_credentials` を返す placeholder にする。

---

## Phase 3: HTTP Rune 2.0 — Favnir 側

**変更ファイル**: `runes/http/*.fav`

### 3-A: `client.fav` 新規作成

- `put(url, body)`: `Http.put_raw(url, body, "application/json")`
- `delete(url)`: `Http.delete_raw(url)`
- `patch(url, body)`: `Http.patch_raw(url, body, "application/json")`
- `get_with_headers(url, headers)`: `Http.get_raw_headers(url, headers)`
- `post_with_headers(url, body, headers)`: `Http.post_raw_headers(url, body, "application/json", headers)`

### 3-B: `retry.fav` 新規作成

- `with_retry(n, f)`: 再帰関数 `attempt(n)` を内部に定義
- `retry_get(url, n)`: `with_retry(n, |_| Http.get_raw(url))`
- `retry_post(url, body, n)`: `with_retry(n, |_| Http.post_raw(url, body, "application/json"))`

### 3-C: `auth.fav` 新規作成

- `bearer(token)`: `Map.set((), "Authorization", "Bearer " + token)`
- `basic(user, pass)`: `Map.set((), "Authorization", "Basic " + encode(user:pass))`
- `api_key(key)`: `Map.set((), "X-Api-Key", key)`

### 3-D: `http.fav` barrel 更新

新モジュールを use で追加。

---

## Phase 4: gRPC フィールド名修正（Rust）

**変更ファイル**: `fav/src/backend/vm.rs`

### 4-A: `proto_bytes_to_string_map` → `proto_bytes_to_named_map`

`type_name: &str` と `type_metas: &HashMap<String, TypeMeta>` を受け取り、
フィールド番号 → フィールド名のマッピングで decode する。

既存の `proto_bytes_to_string_map` は後方互換のため残す
（内部的に `"field{n}"` を使い続ける）。

### 4-B: `Grpc.decode_raw` を修正

`Grpc.decode_raw(type_name, encoded)` が `proto_bytes_to_named_map` を使うよう変更。
これで `grpc.decode` が実フィールド名を返すようになる。

> 破壊的変更の影響: 既存の grpc テスト（`field1`/`field2` を期待しているもの）を更新する。

### 4-C: `Grpc.call_typed_raw` 新規追加

```rust
"Grpc.call_typed_raw" => {
    // args: response_type_name, host, method, payload
    // 既存 call_raw の処理 + decode 時に type_name を使用
}
```

### 4-D: checker.rs 登録

`check_builtin_apply` の `"Grpc"` アームに:
- `call_typed_raw`: `Result<Map<String, String>, RpcError>`

### 4-E: `runes/grpc/client.fav` 更新

- `call_typed(response_type, host, method, payload)`: `Grpc.call_typed_raw(...)` を呼ぶ

---

## Phase 5: `fav db migrate` CLI（Rust, driver.rs）

**変更ファイル**: `fav/src/driver.rs`, `fav/src/main.rs`

### 5-A: `main.rs` に `db migrate` サブコマンドを追加

```
fav db migrate           → cmd_db_migrate(file, false, false)
fav db migrate --status  → cmd_db_migrate_status(file)
fav db migrate --rollback→ cmd_db_migrate_rollback(file)
```

### 5-B: `cmd_db_migrate(db_url, migrations_dir)` 実装

```rust
pub fn cmd_db_migrate(db_url: &str, migrations_dir: &str, dry_run: bool) {
    // 1. SQLite 接続（rusqlite 直接使用）
    // 2. _fav_migrations テーブルを確保
    // 3. migrations_dir/*.sql をアルファベット昇順で列挙
    // 4. 未適用のファイルだけを実行
    // 5. _fav_migrations に記録
}
```

### 5-C: `cmd_db_migrate_status(db_url, migrations_dir)` 実装

適用済み・未適用を表形式で出力。

### 5-D: `FavToml` に `[database]` セクションを追加

```rust
pub struct DatabaseConfig {
    pub url: String,
    pub migrations: Option<String>, // "migrations" がデフォルト
}
```

---

## Phase 6: テスト追加

### 6-A: `runes/db/db.test.fav` 拡張

1. `test_with_transaction_commit`
2. `test_with_transaction_rollback_on_fn_err`
3. `test_paginate_returns_subset`
4. `test_query_one_found`
5. `test_query_one_not_found_is_err`
6. `test_batch_insert_multiple_rows`
7. `test_migration_mark_applied_and_list`
8. `test_savepoint_rollback`

（計 8 件追加、既存 8 件との合計 16 件）

### 6-B: `runes/http/http.test.fav` 拡張

1. `test_retry_get_no_retry_on_success`（ローカルサーバー不使用 — 失敗 URL でリトライ消費確認）
2. `test_bearer_header_format`
3. `test_basic_header_format`
4. `test_api_key_header_format`
5. `test_put_raw_err_on_bad_host`
6. `test_delete_raw_err_on_bad_host`

（計 6 件追加）

### 6-C: `runes/grpc/grpc.test.fav` 拡張

1. `test_decode_returns_named_fields`（`Grpc.decode_raw` でフィールド名を確認）

（計 1 件追加。既存 encode/decode テストは更新が必要）

### 6-D: `vm_stdlib_tests.rs` 追加

1. `http_put_raw_returns_err_on_bad_host`
2. `http_delete_raw_returns_err_on_bad_host`
3. `http_patch_raw_returns_err_on_bad_host`
4. `grpc_call_typed_raw_returns_named_fields`（type_metas に型を登録してから decode）

### 6-E: `driver.rs` 統合テスト追加

1. `db_migrate_status_empty` — migration ディレクトリなし
2. `db_migrate_applies_one_file` — `001_*.sql` を適用
3. `db_rune_with_transaction_commit_in_source` — inline Favnir source
4. `db_rune_paginate_in_source` — inline Favnir source
5. `http_rune_put_err_in_source` — inline Favnir source
6. `http_rune_bearer_header_in_source`

---

## Phase 7: examples + docs

- `examples/db_demo/main.fav` に `db.with_transaction` / `db.paginate` のデモを追加
- `examples/http_demo/main.fav` に `http.with_retry` / `http.bearer` のデモを追加
- `examples/grpc_e2e_demo/` を `call_typed` に更新

---

## 実装順序と依存関係

```
Phase 0 (version bump)
  ↓
Phase 1 (DB Rune Favnir) — 独立（VM 変更なし）
  ↓
Phase 2 (HTTP VM primitives) → Phase 3 (HTTP Rune Favnir)
  ↓
Phase 4 (gRPC fix) — 独立（ただし既存テストの修正が必要）
  ↓
Phase 5 (fav db migrate CLI) — Phase 1 の migration.fav に対応
  ↓
Phase 6 (テスト) — 全 Phase 完了後
  ↓
Phase 7 (examples)
```

Phase 1 と Phase 2/3 は並列実施可能。Phase 4 は独立実施可能。

---

## リスクと対策

| リスク | 影響 | 対策 |
|--------|------|------|
| gRPC フィールド名修正が既存テストを壊す | 既存 grpc encode/decode テストの修正 | Phase 4 着手前に影響範囲を洗い出す |
| `with_transaction` の型がジェネリックでない | 戻り値型が固定になる | `List<Map<String, String>>` に固定、v5.x でジェネリック化 |
| `Http.put_raw` が ureq の API 変更で動かない | HTTP PUT/DELETE が使えない | ureq の現バージョンの PUT/DELETE API を確認してから実装 |
| `fav db migrate` が CLI に複雑さをもたらす | メンテナンスコスト増 | Rust 側は薄いラッパー、ロジックは migration.fav に寄せる |

---

## 完了条件チェックリスト

- [x] `cargo build` が通る
- [x] 既存 808 件が全て pass
- [x] 新規テスト 20 件以上が pass
- [x] `db.with_transaction` が commit/rollback ともに動く（rune テスト）
- [x] `http.with_retry` が動く（rune テスト）
- [x] `grpc.decode` がフィールド名を返す（rune テスト）
- [x] `fav db migrate` が実際の SQLite ファイルにマイグレーションを適用できる
