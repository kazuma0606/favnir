# Favnir v11.5.0 実装計画

作成日: 2026-06-06

---

## 実装順序

```
Phase A: Effect::Postgres 追加（8 ファイル + error_catalog.rs）
    ↓
Phase B: Cargo.toml に tokio-postgres 追加 + vm.rs プリミティブ実装
    ↓
Phase C: compiler.rs builtin NS リストに "Postgres" 追加
    ↓
Phase D: checker.fav 更新（postgres_fn / builtin_ret_ty / ns_to_effect）
    ↓
Phase E: runes/postgres/postgres.fav Rune 実装
    ↓
Phase F: fav.toml [postgres] セクション解析 + inject_postgres_config
    ↓
Phase G: fav infer --from postgres --table <name>
    ↓
Phase H: テスト（v11500_tests）
    ↓
Phase I: バージョン更新・コミット
```

---

## Phase A — Effect::Postgres 追加（8 ファイル + error_catalog）

### A-1: `fav/src/ast.rs`

`Effect` enum に追加（`Snowflake` の直後）:
```rust
Postgres,
```

### A-2: `fav/src/frontend/parser.rs`

`parse_effect` 関数の `"Snowflake"` ブランチの直後:
```rust
"Postgres" => {
    self.advance();
    Effect::Postgres
}
```

### A-3: `fav/src/fmt.rs`

`effect_to_str` / `Display` の `Snowflake` の直後:
```rust
Effect::Postgres => Some("!Postgres".to_string()),
```

### A-4: `fav/src/lineage.rs`

effect 表示の `Snowflake` の直後:
```rust
Postgres => "!Postgres".into(),
```

### A-5: `fav/src/driver.rs`

effect 表示文字列 2 箇所（長表示 + 短縮名）に追加:
```rust
Postgres => "!Postgres".into(),
// および
ast::Effect::Postgres => "Postgres".into(),
```

### A-6: `fav/src/middle/ast_lower_checker.rs`

lowering の `Snowflake` の直後:
```rust
ast::Effect::Postgres => "Postgres".to_string(),
```

### A-7: `fav/src/middle/checker.rs`

1. builtin NS ホワイトリスト 2 箇所に `"Postgres"` 追加
2. effects ホワイトリスト 2 箇所に `"Postgres"` 追加
3. `require_postgres_effect` 関数追加（E0315）:
```rust
fn require_postgres_effect(&mut self, span: &Span) {
    if !self.has_effect(|e| matches!(e, Effect::Postgres)) {
        self.type_error(
            "E0315",
            "Postgres.* call requires `!Postgres` effect on enclosing fn/stage",
            span,
        );
    }
}
```
4. `("Postgres", "execute_raw")` / `("Postgres", "query_raw")` / `("Postgres", "infer_table_raw")` の型シグネチャ追加

### A-8: `fav/src/middle/reachability.rs`

`Snowflake` の直後:
```rust
Effect::Postgres => {
    effects_required.insert("Postgres".to_string());
}
```

### A-9: `fav/src/error_catalog.rs`

E0314 の直後に E0315 エントリ追加。

---

## Phase B — Cargo.toml + vm.rs プリミティブ

### B-1: `fav/Cargo.toml`

`[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` に追加:
```toml
tokio-postgres = "0.7"
```

（既存の `postgres = { version = "0.19", optional = true }` は残す）

### B-2: `fav/src/backend/vm.rs` — `Postgres.execute_raw`

```rust
("Postgres", "execute_raw") => {
    // args[0] = sql: String, args[1] = params_json: String
    let sql = vm_val_to_string(&args[0]);
    let params_json = vm_val_to_string(&args[1]);
    let conn_str = build_pg_conn_str(&self.config);
    match pg_execute(&conn_str, &sql, &params_json) {
        Ok(())  => Value::Enum("Ok".into(),  vec![Value::Unit]),
        Err(e)  => Value::Enum("Err".into(), vec![Value::Str(e)]),
    }
}
```

### B-3: `fav/src/backend/vm.rs` — `Postgres.query_raw`

```rust
("Postgres", "query_raw") => {
    let sql = vm_val_to_string(&args[0]);
    let params_json = vm_val_to_string(&args[1]);
    let conn_str = build_pg_conn_str(&self.config);
    match pg_query(&conn_str, &sql, &params_json) {
        Ok(json) => Value::Enum("Ok".into(),  vec![Value::Str(json)]),
        Err(e)   => Value::Enum("Err".into(), vec![Value::Str(e)]),
    }
}
```

### B-4: ヘルパー関数

```rust
fn build_pg_conn_str(config: &RunConfig) -> String { ... }
fn pg_execute(conn_str: &str, sql: &str, params_json: &str) -> Result<(), String> { ... }
fn pg_query(conn_str: &str, sql: &str, params_json: &str) -> Result<String, String> { ... }
```

`tokio::runtime::Builder::new_current_thread().enable_all().build()` で block_on 実行。
`params_json` は `serde_json::from_str::<Vec<serde_json::Value>>` で解析し、
tokio-postgres の `&[&(dyn ToSql + Sync)]` に変換。

### B-5: `Postgres.infer_table_raw`

`information_schema.columns` への SELECT で列名・型を取得し JSON で返す。

---

## Phase C — compiler.rs builtin NS 追加

`fav/src/compiler.rs` の builtin namespace リスト 2 箇所に `"Postgres"` を追加
（Snowflake と同じ箇所）。

---

## Phase D — checker.fav 更新

`fav/self/checker.fav` の以下を更新:

```
fn postgres_fn(name: String) -> Bool {
  name == "execute_raw" || name == "query_raw" || name == "infer_table_raw"
}
```

`builtin_ret_ty` に Postgres エントリ追加:
```
else if ns == "Postgres" && postgres_fn(name) {
  if name == "execute_raw" { "Result<Unit,String>" }
  else { "Result<String,String>" }
}
```

`ns_to_effect` に追加:
```
else if ns == "Postgres" { "Postgres" }
```

---

## Phase E — runes/postgres/postgres.fav

`fav/runes/postgres/` ディレクトリを作成し `postgres.fav` を実装:
- `execute(sql, params)` — execute_raw ラッパー
- `query<T>(sql, params)` — query_raw + Schema.adapt ラッパー

`fav/runes/postgres/rune.toml`:
```toml
[rune]
name = "postgres"
version = "0.1.0"
description = "PostgreSQL CRUD operations for Favnir pipelines"
effects = ["!Postgres"]
```

---

## Phase F — fav.toml [postgres] セクション

### F-1: `fav/src/config.rs`（または config 定義箇所）

`RunConfig` に `postgres: Option<PostgresConfig>` 追加:
```rust
#[derive(Deserialize)]
pub struct PostgresConfig {
    pub host:     String,
    pub port:     Option<u16>,
    pub dbname:   String,
    pub user:     String,
    pub password: String,
    pub sslmode:  Option<String>,
}
```

### F-2: `load_run_config` 更新

`[postgres]` セクションを読み込み後、`expand_env_vars` を各フィールドに適用。

---

## Phase G — fav infer --from postgres

### G-1: `driver.rs` の `cmd_infer` 更新

`--from postgres` オプションを認識し、
`Postgres.infer_table_raw(table)` プリミティブを直接呼び出して型定義を生成:

```
type <TableName> = {
  <col>: <FavType>
  ...
}
```

型マッピング（PostgreSQL → Fav）:
| PG 型 | Fav 型 |
|---|---|
| `varchar`, `text`, `char` | `String` |
| `int2`, `int4`, `int8`, `serial` | `Int` |
| `float4`, `float8`, `numeric` | `Float` |
| `bool` | `Bool` |
| その他 | `String` |

---

## Phase H — テスト（driver.rs 末尾 v11500_tests）

```rust
#[test] fn postgres_execute_requires_effect()     // !Postgres なし → E0315
#[test] fn postgres_execute_with_effect_ok()      // !Postgres あり → エラーなし
#[test] fn postgres_query_raw_type()              // query_raw → Result<String, String>
#[test] fn postgres_lineage_shows_effect()        // lineage に !Postgres が含まれる
#[test] fn fav_toml_postgres_section_parsed()     // RunConfig に postgres フィールド
```

---

## Phase I — バージョン更新・コミット

- `fav/Cargo.toml`: `version = "11.5.0"`
- `cargo build` で `Cargo.lock` 更新
- `git commit & push` — CI 確認
