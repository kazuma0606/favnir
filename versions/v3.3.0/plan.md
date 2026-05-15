# Favnir v3.3.0 Implementation Plan

## Overview

v3.3.0 は `db` rune を導入する。
中心設計は **`DB.connect` → `DB.query_raw` → `Schema.adapt<T>`** のチェーン。
`Schema.adapt<T>` は v3.2.0 で実装済みのため、DB 固有の実装は
接続管理・SQL 実行・トランザクション制御のみ。

Total phases: 8

---

## Phase 0: Version Bump

**Goal**: バージョン文字列を `3.3.0` に更新する。

- `fav/Cargo.toml`: `version = "3.3.0"`
- `cargo build` で `env!("CARGO_PKG_VERSION")` 伝播を確認

---

## Phase 1: 型定義 + エフェクト追加

**Goal**: `DbError`, `DbHandle`, `TxHandle`, `effect Db` を型システムに追加する。

### 1-A: `DbError` 型 (`checker.rs`)

- `"DbError"` を stdlib 型として登録（フィールド: `code: String`, `message: String`）
- `"DB"` namespace を checker の stdlib グローバル登録ループに追加
- `"Env"` namespace を同様に追加（`Env.get` / `Env.get_or` 用）

### 1-B: `DbHandle` / `TxHandle` 不透明型 (`backend/vm.rs`)

```rust
// VMValue に追加
VMValue::DbHandle(Arc<Mutex<DbConnection>>)
VMValue::TxHandle(Arc<Mutex<DbTransaction>>)
```

- `DbConnection` は enum: `Sqlite(rusqlite::Connection)` / `Postgres(postgres::Client)`
- `DbTransaction` は enum: `SqliteTx { ... }` / `PostgresTx { ... }`
- `vmvalue_type_name` に `"DbHandle"` / `"TxHandle"` を追加

### 1-C: `effect Db` (`middle/checker.rs`)

- `"Db"` を有効エフェクトセットに追加
- `DB.*` 呼び出し時にエフェクト `Db` を要求するチェックを追加

テスト:
- `db_handle_type_name` — `DB.connect` の戻り値型が `Result<DbHandle, DbError>` になることを確認

---

## Phase 2: SQLite VM プリミティブ (`backend/vm.rs`)

**Goal**: SQLite バックエンドで全 VM プリミティブを実装する。
`rusqlite` クレートは Cargo.toml に既存。

### 2-A: 接続管理

```rust
"DB.connect" => {
    // conn_str をパースしてドライバ種別を判定
    // "sqlite:..." → rusqlite::Connection::open(path)
    // "sqlite::memory:" → rusqlite::Connection::open_in_memory()
    // "postgres://..." → Phase 3 で実装（Phase 2 では E0605 を返す）
    // → VMValue::Ok(VMValue::DbHandle(...))
}

"DB.close" => {
    // DbHandle を drop して接続を閉じる
    // → VMValue::Unit
}
```

### 2-B: クエリ実行

```rust
"DB.query_raw" => {
    // args: [DbHandle, sql: String]
    // rusqlite でクエリ実行 → Vec<HashMap<String,String>>
    // → VMValue::Ok(VMValue::List([...]))
}

"DB.execute_raw" => {
    // args: [DbHandle, sql: String]
    // rusqlite execute → affected rows: i64
    // → VMValue::Ok(VMValue::Int(n))
}

"DB.query_raw_params" => {
    // args: [DbHandle, sql: String, params: List<String>]
    // rusqlite prepare/query_map with bound params
}

"DB.execute_raw_params" => {
    // args: [DbHandle, sql: String, params: List<String>]
}
```

### 2-C: トランザクション

```rust
"DB.begin_tx" => {
    // DbHandle からトランザクションを開始
    // → VMValue::Ok(VMValue::TxHandle(...))
}

"DB.commit_tx" => { ... }
"DB.rollback_tx" => { ... }
"DB.query_in_tx" => { ... }
"DB.execute_in_tx" => { ... }
```

テスト (`backend/vm_stdlib_tests.rs`):
- `db_sqlite_connect_and_close`
- `db_sqlite_create_and_insert`
- `db_sqlite_query_returns_rows`
- `db_sqlite_query_params_bind`
- `db_sqlite_execute_returns_affected_rows`
- `db_sqlite_transaction_commit`
- `db_sqlite_transaction_rollback`

---

## Phase 3: PostgreSQL VM プリミティブ (`backend/vm.rs`)

**Goal**: PostgreSQL バックエンドを追加する。

### 3-A: `postgres` クレート追加

```toml
# fav/Cargo.toml
postgres = "0.19"
```

### 3-B: `DB.connect` の PostgreSQL 分岐

```rust
"postgres://..." => {
    let client = postgres::Client::connect(&conn_str, postgres::NoTls)
        .map_err(|e| schema_error_vm("connection", "DbError", &e.to_string()))?;
    Ok(ok_vm(VMValue::DbHandle(Arc::new(Mutex::new(DbConnection::Postgres(client))))))
}
```

### 3-C: 全プリミティブに PostgreSQL 分岐を追加

`DB.query_raw`, `DB.execute_raw`, `DB.query_raw_params`, `DB.execute_raw_params`,
`DB.begin_tx`, `DB.commit_tx`, `DB.rollback_tx`, `DB.query_in_tx`, `DB.execute_in_tx`

それぞれ `DbConnection::Postgres(client)` に対する実装を追加。

テスト:
- PostgreSQL テストは `#[cfg(feature = "postgres_integration")]` でゲートし、
  通常の `cargo test` では実行しない（CI 環境変数 `TEST_DB_URL` があれば実行）

---

## Phase 4: `Env.get` VM プリミティブ (`backend/vm.rs`)

**Goal**: 環境変数読み取りを追加する（L005 の推奨代替として必要）。

```rust
"Env.get" => {
    // args: [name: String]
    // std::env::var(name) → Ok(String) or Err(DbError相当)
    // エフェクト: Io（環境変数は外部状態）
}

"Env.get_or" => {
    // args: [name: String, default: String]
    // std::env::var(name).unwrap_or(default)
    // エフェクト: なし（デフォルトがあるため純粋）
}
```

- `checker.rs`: `Env.get` / `Env.get_or` の型シグネチャを登録
- `compiler.rs`: `"Env"` を登録ループに追加

テスト:
- `env_get_or_returns_default_when_missing`
- `env_get_or_returns_value_when_set`

---

## Phase 5: `runes/db/db.fav` + `runes/db/db.test.fav`

**Goal**: Favnir 製 rune ファイルを作成する。

### 5-A: `runes/db/db.fav`

- `public fn connect(conn_str)` — `DB.connect` ラッパー
- `public fn query<T>(handle, sql)` — `DB.query_raw` + `Schema.adapt` + エラー変換
- `public fn query_params<T>(handle, sql, params)` — パラメータ付き版
- `public fn execute(handle, sql)` — `DB.execute_raw` ラッパー
- `public fn execute_params(handle, sql, params)` — パラメータ付き版
- `public fn transaction<T>(handle, f)` — begin/commit/rollback の合成
- `public fn close(handle)` — `DB.close` ラッパー

### 5-B: `runes/db/db.test.fav`（SQLite インメモリのみ）

テスト 8 件:
- `test_connect_sqlite_memory`
- `test_create_table_and_insert`
- `test_query_returns_typed_rows`
- `test_query_params_bind`
- `test_execute_returns_affected_rows`
- `test_transaction_commit`
- `test_transaction_rollback_on_error`
- `test_schema_mismatch_returns_err`

---

## Phase 6: checker 統合 + L005 リンタ

**Goal**: 型チェックとリンタ警告を追加する。

### 6-A: エラーコード追加 (`error_catalog.rs`)

- E0601〜E0605 を追加

### 6-B: checker.rs

- `DB.*` 関数の型シグネチャ登録（戻り型 `Result<T, DbError>`）
- `DB.*` 呼び出し時にエフェクト `Db` を推論
- `Env.get` のエフェクト `Io` を推論
- `Env.get_or` はエフェクトなし

### 6-C: L005 リンタ (`lint.rs`)

接続文字列リテラルにパスワードが含まれている可能性を検出:

```rust
// 検出パターン: DB.connect("postgres://user:PASSWORD@...")
// StringLiteral が DB.connect の第1引数で、"://" と "@" を両方含む場合
```

- `LintWarning { code: "L005", message: "hardcoded db credential", hint: "..." }`
- `fav lint` 実行時に報告

テスト:
- `lint_l005_postgres_url_with_password`
- `lint_l005_sqlite_no_warning` — SQLite は L005 を出さない

---

## Phase 7: サンプル + driver.rs 統合テスト

**Goal**: エンドツーエンドで動く examples と Rust 側統合テストを追加する。

### 7-A: `examples/db_demo/`

```
examples/db_demo/
  fav.toml
  src/main.fav    — SQLite CRUD + トランザクション
```

### 7-B: driver.rs 統合テスト（`tests` モジュール内）

- `db_rune_connect_and_query` — インメモリ SQLite で CRUD
- `db_rune_query_params_bind` — プレースホルダーバインド
- `db_rune_transaction_commit`
- `db_rune_transaction_rollback`
- `db_rune_schema_mismatch_returns_err` — カラム型不一致で E0604
- `db_rune_test_file_passes` — `runes/db/db.test.fav` の全テスト実行
- `env_get_or_in_favnir_source` — `Env.get_or` の動作確認

---

## Phase 8: ドキュメント

**Goal**: バージョンドキュメントを作成する。

- `versions/v3.3.0/langspec.md` — v3.2.0 langspec に db rune、Env.get、effect Db を追記
- `versions/v3.3.0/migration-guide.md` — v3.2.0 → v3.3.0（破壊的変更なし）
- `versions/v3.3.0/progress.md` — 全 Phase `[x]`

---

## 依存関係グラフ

```
Phase 0 (version)
    └── Phase 1 (DbError + DbHandle + effect Db)
            ├── Phase 2 (SQLite VM prims)
            │       └── Phase 3 (PostgreSQL VM prims)
            └── Phase 4 (Env.get VM prims)
                    └── Phase 5 (db.fav + db.test.fav)  ← Phase 2 も必要
                            └── Phase 6 (checker + L005)
                                    └── Phase 7 (examples + integration tests)
                                            └── Phase 8 (docs)
```

Phase 2 と Phase 4 は Phase 1 完了後に並行開発可能。
Phase 3 は Phase 2 完了後。
