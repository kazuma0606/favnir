# Favnir v11.5.0 仕様書

作成日: 2026-06-06
テーマ: `!Postgres` エフェクト + Fav ネイティブ Postgres 操作

---

## 背景と目的

v11.4.0 で `!AWS` → boto3 変換が完了した。
v11.5.0 では Fav に `!Postgres` エフェクトを追加し、
PostgreSQL に対して `Postgres.execute_raw` / `Postgres.query_raw` で直接操作できるようにする。

Snowflake 統合（v10.2.0〜v10.8.0）と同じパターンで実装する:
1. Effect 型追加（8 ファイル + error_catalog）
2. VM プリミティブ実装（tokio-postgres ベース）
3. fav.toml `[postgres]` セクション
4. checker.fav 更新
5. Rune 実装
6. `fav infer --from postgres`

---

## Postgres プリミティブ一覧

| Primitive | シグネチャ | 説明 |
|---|---|---|
| `Postgres.execute_raw` | `(sql: String, params: String) -> Result<Unit, String> !Postgres` | INSERT/UPDATE/DELETE 実行 |
| `Postgres.query_raw` | `(sql: String, params: String) -> Result<String, String> !Postgres` | SELECT → JSON 文字列で返す |
| `Postgres.infer_table_raw` | `(table: String) -> Result<String, String> !Postgres` | テーブル列情報を JSON で返す（`fav infer` 用） |

- `params` は JSON 配列文字列（例: `"[\"val1\", 42]"`）。空の場合は `"[]"` を渡す
- `query_raw` の戻り値は `[{"col": "val", ...}, ...]` 形式の JSON 文字列

---

## fav.toml `[postgres]` セクション

```toml
[postgres]
host     = "localhost"
port     = 5432
dbname   = "mydb"
user     = "postgres"
password = "${POSTGRES_PASSWORD}"
sslmode  = "require"
```

- `${}` 形式の env var 展開（`expand_env_vars` 既存関数を再利用）
- 設定がない場合は `DATABASE_URL` 環境変数（libpq 互換）にフォールバック

---

## Effect::Postgres 追加（8 ファイル + error_catalog）

### ast.rs
```rust
pub enum Effect {
    // ...既存...
    Snowflake,
    Postgres,   // ← 追加
}
```

### parser.rs
```rust
"Snowflake" => { self.advance(); Effect::Snowflake }
"Postgres"  => { self.advance(); Effect::Postgres  }  // ← 追加
```

### fmt.rs / lineage.rs / driver.rs / ast_lower_checker.rs
それぞれ `Snowflake` のパターンの直後に `Postgres` を追加。

### checker.rs
- builtin NS ホワイトリスト 2 箇所に `"Postgres"` を追加
- effects ホワイトリスト 2 箇所に `"Postgres"` を追加
- `require_postgres_effect` 関数を追加（E0315）
- `("Postgres", "execute_raw")` / `("Postgres", "query_raw")` の型シグネチャ追加

### reachability.rs
```rust
Effect::Postgres => {
    effects_required.insert("Postgres".to_string());
}
```

### error_catalog.rs — E0315 追加

```rust
ErrorEntry {
    code: "E0314",  // ← E0314 の直後に
    ...
},
ErrorEntry {
    code: "E0315",
    title: "undeclared !Postgres effect",
    category: "effects",
    description: "A Postgres operation was used in a function that does not declare `!Postgres`.",
    example: "fn run(sql: String) -> Result<String, String> {\n    Postgres.query_raw(sql, \"[]\")  // E0315\n}",
    fix: "Add `!Postgres` to the function signature.",
},
```

---

## vm.rs プリミティブ実装

依存: `tokio-postgres = "0.7"` を `[target.cfg(not(target_arch = \"wasm32\"))].dependencies` に追加。

接続文字列の構築:

```rust
fn build_pg_conn_str(config: &RunConfig) -> String {
    if let Some(pg) = &config.postgres {
        format!(
            "host={} port={} dbname={} user={} password={} sslmode={}",
            pg.host, pg.port, pg.dbname, pg.user, pg.password,
            pg.sslmode.as_deref().unwrap_or("prefer")
        )
    } else {
        std::env::var("DATABASE_URL").unwrap_or_default()
    }
}
```

`Postgres.execute_raw(sql, params_json)` の実装:
- params_json を `serde_json::Value::Array` として解析
- `tokio::runtime::Builder::new_current_thread().build().block_on(...)` でブロッキング実行
- 戻り値: `Ok(())` → `Value::String("ok")` でラップ、エラー → `Err(msg)`

`Postgres.query_raw(sql, params_json)` の実装:
- 各行を `serde_json::Map` に変換（列名 → 値）
- 全行を JSON 配列文字列として返す

---

## fav.toml [postgres] セクション

```rust
#[derive(Deserialize)]
pub struct PostgresConfig {
    pub host:     String,
    pub port:     u16,
    pub dbname:   String,
    pub user:     String,
    pub password: String,
    pub sslmode:  Option<String>,
}

pub struct RunConfig {
    // ...既存...
    pub postgres: Option<PostgresConfig>,
}
```

`load_run_config` で `[postgres]` セクションを読み込み、`expand_env_vars` で `${}` を展開。

---

## checker.fav 更新

```
fn postgres_fn(name: String) -> Bool { ... }
```

`builtin_ret_ty` / `ns_to_effect` に Postgres エントリを追加（Snowflake のパターンと同様）。

---

## runes/postgres/postgres.fav

```
fn execute<T>(sql: String, params: List<T>) -> Result<Unit, String> !Postgres {
  let params_json = Json.encode_raw(params)
  Postgres.execute_raw(sql, params_json)
}

fn query<T>(sql: String, params: List<T>) -> Result<List<T>, String> !Postgres {
  let params_json = Json.encode_raw(params)
  match Postgres.query_raw(sql, params_json) {
    Err(e) => Err(e)
    Ok(json) =>
      match Schema.adapt_raw(json, T.name()) {
        Err(e) => Err(e)
        Ok(rows) => Ok(rows)
      }
  }
}
```

---

## fav infer --from postgres --table <name>

`Postgres.infer_table_raw(table)` を使って `information_schema.columns` を参照し、
型マッピング（varchar→String, int4/int8→Int, float4/float8→Float, bool→Bool）で
Fav の `type` 定義を生成する。

---

## テスト設計

| テスト名 | 検証内容 |
|---|---|
| `postgres_execute_requires_effect` | `!Postgres` なし → E0315 |
| `postgres_execute_with_effect_ok` | `!Postgres` あり → エラーなし |
| `postgres_query_raw_type` | `query_raw` の戻り型が `Result<String, String>` |
| `postgres_lineage_shows_effect` | `fav explain --lineage` に `!Postgres` が含まれる |
| `fav_toml_postgres_section_parsed` | `[postgres]` セクションが `RunConfig` に読み込まれる |

---

## バージョン更新

- `fav/Cargo.toml`: `version = "11.5.0"`
- `tokio-postgres = "0.7"` を native-only dependencies に追加
