# v27.9.0 仕様書 — sqlite Rune 追加

## 概要

`runes/sqlite/` を新規作成し、依存ゼロ・ローカル動作の組み込み DB SQLite を
Favnir から型安全に操作できるようにする。ローカル開発・軽量 ETL・テスト用 DB として活用する。

---

## 背景

ロードマップ v27.9「sqlite Rune 追加」より。Data Lakehouse フェーズの第 9 コンポーネント。
`rusqlite --features bundled` で依存ゼロで動作する。
MockDb の代替として Favnir のテスト基盤でも有用。

v27.9.0 は stub 実装。各 primitive は引数検証のみを行い、固定値を返す。
実際の SQLite 操作（`rusqlite` クレート統合）は v28.x に延期する。

---

## Rune API

```favnir
import rune "sqlite"

stage CreateTable: Unit -> Result<Unit, String> !Db = |_| {
    bind db <- SQLite.open(":memory:")
    bind _  <- SQLite.execute(db, "CREATE TABLE data (id INT, name TEXT)", "[]")
    Result.ok(unit)
}

stage InsertRows: Unit -> Result<Int, String> !Db = |_| {
    bind db <- SQLite.open(":memory:")
    SQLite.execute(db, "INSERT INTO data VALUES (1, 'test')", "[]")
}

seq SqliteEtlPipeline = CreateTable
```

---

## 実装対象ファイル

### 1. `fav/src/backend/vm.rs` — 新 primitive 6 件追加

| primitive 名 | シグネチャ（引数） | 実装方針 |
|---|---|---|
| `SQLite.open_raw` | `(path: String)` | stub: 引数検証、固定ハンドル `"sqlite-stub-conn"` 返却 |
| `SQLite.open_memory_raw` | `()` | stub: 引数なし、固定ハンドル `"sqlite-memory-stub-conn"` 返却 |
| `SQLite.query_raw` | `(db: String, sql: String, params: String)` | stub: 引数検証、`"[]"` 返却 |
| `SQLite.execute_raw` | `(db: String, sql: String, params: String)` | stub: 引数検証、影響行数 `0`（Int）返却 |
| `SQLite.execute_many_raw` | `(db: String, sql: String, rows: String)` | stub: 引数検証、影響行数 `0`（Int）返却 |
| `SQLite.close_raw` | `(db: String)` | stub: 引数検証、`Unit` 返却 |

> **挿入位置**:
> - Dbt ブロック末尾（`"Dbt.source_raw"` の wasm32 アーム直後、**行 18040 付近**）に追加
> - Azure Blob Storage ブロック（`// ── Azure Blob Storage primitives (v14.5.0)`）の直前
> - 全 primitive に `#[cfg(not(target_arch = "wasm32"))]` / `#[cfg(target_arch = "wasm32")]` ガード付き

> **戻り値の設計**:
> - `open_raw` / `open_memory_raw`: 接続ハンドル識別子文字列を返す（`ok_vm(VMValue::Str(...))` — postgres/ClickHouse/Redshift と同パターン）
> - `query_raw`: `"[]"`（空 JSON 配列 = 空行セット）
> - `execute_raw` / `execute_many_raw`: `ok_vm(VMValue::Int(0))`（影響行数 0 — Redshift.execute_raw と同パターン）
> - `close_raw`: `ok_vm(VMValue::Unit)`

### 2. `runes/sqlite/sqlite.fav` — 新規作成（6 関数）

```favnir
// runes/sqlite/sqlite.fav — SQLite Rune (v27.9.0)
// 依存ゼロ・ローカル動作の組み込み DB。ローカル開発・軽量 ETL・テスト用 DB として活用する。
// v27.9.0 stub — rusqlite 統合は v28.x 以降

public fn open(path: String) -> Result<String, String> !Db {
    SQLite.open_raw(path)
}

public fn open_memory() -> Result<String, String> !Db {
    SQLite.open_memory_raw()
}

public fn query(db: String, sql: String, params: String) -> Result<String, String> !Db {
    SQLite.query_raw(db, sql, params)
}

public fn execute(db: String, sql: String, params: String) -> Result<Int, String> !Db {
    SQLite.execute_raw(db, sql, params)
}

public fn execute_many(db: String, sql: String, rows: String) -> Result<Int, String> !Db {
    SQLite.execute_many_raw(db, sql, rows)
}

public fn close(db: String) -> Result<Unit, String> !Db {
    SQLite.close_raw(db)
}
```

**エフェクト**: `!Db`（SQLite はデータベース操作のため）

### 3. `examples/sqlite_etl.fav` — 新規作成

```favnir
// examples/sqlite_etl.fav — SQLite 軽量 ETL パイプラインデモ（v27.9.0 stub）

import rune "sqlite"

stage CreateTable: Unit -> Result<Unit, String> !Db = |_| {
    bind db <- SQLite.open(":memory:")
    bind _  <- SQLite.execute(db, "CREATE TABLE data (id INT, name TEXT)", "[]")
    Result.ok(unit)
}

stage InsertRows: Unit -> Result<Int, String> !Db = |_| {
    bind db <- SQLite.open(":memory:")
    SQLite.execute(db, "INSERT INTO data VALUES (1, 'test')", "[]")
}

seq SqliteEtlPipeline = CreateTable
```

### 4. `site/content/docs/runes/sqlite.mdx` — 新規作成

SQLite Rune の使用方法・API リファレンス・v28.x 予定事項を記載。

### 5. `fav/self/checker.fav` — `ns_to_effect` 更新（T10）

`SQLite` は新規 namespace。`!Db` エフェクトに対応するため `ns_to_effect` に追加する:

```favnir
if ns == "SQLite" { "Db" } else { "" }
```

`"Dbt" => "Db"` ブロックの直後（`else { "" }` の直前）に追加する。

> **実施タイミング**: T10 は T9（全テスト実行）より前に完了すること（v27.6.0 の教訓）。

---

## テスト

### driver.rs v279000_tests（16 件）

| テスト名 | 内容 |
|---|---|
| `sqlite_rune_has_open_fn` | `runes/sqlite/sqlite.fav` に `fn open(` が含まれること |
| `sqlite_rune_has_open_memory_fn` | `runes/sqlite/sqlite.fav` に `fn open_memory(` が含まれること |
| `sqlite_rune_has_query_fn` | `runes/sqlite/sqlite.fav` に `fn query(` が含まれること |
| `sqlite_rune_has_execute_fn` | `runes/sqlite/sqlite.fav` に `fn execute(` が含まれること |
| `sqlite_rune_has_execute_many_fn` | `runes/sqlite/sqlite.fav` に `fn execute_many(` が含まれること |
| `sqlite_rune_has_close_fn` | `runes/sqlite/sqlite.fav` に `fn close(` が含まれること |
| `sqlite_rune_uses_db_effect` | `runes/sqlite/sqlite.fav` に `!Db` が含まれること |
| `vm_has_sqlite_open_raw` | `vm.rs` に `SQLite.open_raw` が含まれること |
| `vm_has_sqlite_open_memory_raw` | `vm.rs` に `SQLite.open_memory_raw` が含まれること |
| `vm_has_sqlite_query_raw` | `vm.rs` に `SQLite.query_raw` が含まれること |
| `vm_has_sqlite_execute_raw` | `vm.rs` に `SQLite.execute_raw` が含まれること |
| `vm_has_sqlite_execute_many_raw` | `vm.rs` に `SQLite.execute_many_raw` が含まれること |
| `vm_has_sqlite_close_raw` | `vm.rs` に `SQLite.close_raw` が含まれること |
| `sqlite_example_has_pipeline` | `examples/sqlite_etl.fav` に `SqliteEtlPipeline` が含まれること |
| `changelog_has_v27_9_0` | `CHANGELOG.md` に `[v27.9.0]` が含まれること |
| `checker_has_sqlite_effect` | `checker.fav` に `"SQLite"` が含まれること |

### `cargo test sqlite` 期待値

- `cargo test v279000 --bin fav` — 16/16 PASS
- `cargo test sqlite --bin fav` — 15 件 PASS（`changelog_has_v27_9_0` はテスト名に `sqlite` を含まないため除外）
- ロードマップ要件「4 件以上 PASS」を超過

> 詳細な `include_str!` パス対応表は tasks.md メモを参照。

---

## 完了条件

- [ ] `fav/Cargo.toml` が `version = "27.9.0"` であること
- [ ] `fav/src/backend/vm.rs` に `SQLite.open_raw` が含まれること
- [ ] `fav/src/backend/vm.rs` に `SQLite.open_memory_raw` が含まれること
- [ ] `fav/src/backend/vm.rs` に `SQLite.query_raw` が含まれること
- [ ] `fav/src/backend/vm.rs` に `SQLite.execute_raw` が含まれること
- [ ] `fav/src/backend/vm.rs` に `SQLite.execute_many_raw` が含まれること
- [ ] `fav/src/backend/vm.rs` に `SQLite.close_raw` が含まれること
- [ ] `runes/sqlite/sqlite.fav` に `public fn open(` が含まれること
- [ ] `runes/sqlite/sqlite.fav` に `public fn open_memory(` が含まれること
- [ ] `runes/sqlite/sqlite.fav` に `public fn query(` が含まれること
- [ ] `runes/sqlite/sqlite.fav` に `public fn execute(` が含まれること
- [ ] `runes/sqlite/sqlite.fav` に `public fn execute_many(` が含まれること
- [ ] `runes/sqlite/sqlite.fav` に `public fn close(` が含まれること
- [ ] `runes/sqlite/sqlite.fav` に `!Db` エフェクトが使われていること
- [ ] `examples/sqlite_etl.fav` に `SqliteEtlPipeline` が含まれること
- [ ] `site/content/docs/runes/sqlite.mdx` が存在すること
- [ ] `fav/self/checker.fav` の `ns_to_effect` に `"SQLite"` が登録されていること
- [ ] `CHANGELOG.md` に `[v27.9.0]` エントリが存在すること
- [ ] `benchmarks/v27.9.0.json` が存在すること（test_count: 2220）
- [ ] `v279000_tests` 16 件すべて PASS
- [ ] 総テスト数 ≥ 2220 件

---

## 設計注記

### `execute_raw` / `execute_many_raw` の戻り値が `Int`

`SQLite.execute` / `execute_many` は影響行数（affected rows）を返す。
これは `Redshift.execute_raw` が `ok_vm(VMValue::Int(0))` を返すのと同一パターン。
`Unit` ではなく `Int` を選択する理由: バッチ処理での成功行数確認に有用。

### `open_memory_raw` の引数なし設計

インメモリ DB はパス引数が不要。既存 vm.rs の primitive でも引数なし primitive は多数存在する。
Rust 側では `args.into_iter()` の要素を使わず直接 stub 値を返す。

---

## スコープ外（v28.x 以降）

- `rusqlite --features bundled` を使った実 SQLite 操作
- `SQLite.query[T]` の型パラメータによるデシリアライズ
- `SQLite.execute_many` でのバッチトランザクション最適化
- `NULL` 値 / `BLOB` 型の Favnir 型マッピング
- WAL モード設定（`PRAGMA journal_mode=WAL`）
