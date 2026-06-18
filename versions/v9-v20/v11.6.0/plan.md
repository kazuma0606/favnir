# Favnir v11.6.0 実装計画

作成日: 2026-06-06

---

## 実装順序

```
Phase A: emit_python.rs — PyEmitter に Postgres フラグ追加
    ↓
Phase B: emit_python.rs — Postgres NS ディスパッチ（execute_raw / query_raw）
    ↓
Phase C: emit_python.rs — _pg_connect / _pg_execute / _pg_query ヘルパー emit
    ↓
Phase D: driver.rs — pyproject.toml に psycopg2-binary 依存追加
    ↓
Phase E: テスト（v11600_tests）
    ↓
Phase F: バージョン更新・コミット
```

---

## Phase A — PyEmitter フラグ追加

### A-1: `struct PyEmitter` にフィールド追加

`needs_boto3` の直後:
```rust
needs_psycopg2:      bool,
needs_pg_helpers:    bool,
```

### A-2: `PyEmitter::new()` 初期化

```rust
needs_psycopg2:      false,
needs_pg_helpers:    false,
```

### A-3: `copy_flags_from_sub` に追加

```rust
self.needs_psycopg2   = sub.needs_psycopg2;
self.needs_pg_helpers = sub.needs_pg_helpers;
```

### A-4: `emit_imports` に追加

`needs_boto3` の直後:
```rust
if self.needs_psycopg2 {
    self.line("import psycopg2");
    self.line("import psycopg2.extras");
    self.line("import os as _os");
}
```

### A-5: `emit_helpers_and_classes` に追加

`emit_aws_sqs_helpers` の直後:
```rust
if self.needs_pg_helpers { self.emit_pg_helpers(); }
```

---

## Phase B — Postgres NS ディスパッチ

`fav/src/emit_python.rs` の `emit_call_expr` 内、`("Snowflake", name)` ブランチの直後に追加:

```rust
// ── Postgres ─────────────────────────────────────────────
("Postgres", "execute_raw") if a.len() == 2 => {
    self.needs_psycopg2 = true;
    self.needs_pg_helpers = true;
    self.needs_json = true;
    return format!("_pg_execute({}, {})", a[0], a[1])
}
("Postgres", "query_raw") if a.len() == 2 => {
    self.needs_psycopg2 = true;
    self.needs_pg_helpers = true;
    self.needs_json = true;
    return format!("_pg_query({}, {})", a[0], a[1])
}
("Postgres", name) => {
    self.needs_psycopg2 = true;
    self.needs_pg_helpers = true;
    return format!("_pg_{}({})", to_snake(name), a.join(", "))
}
```

---

## Phase C — `emit_pg_helpers` 実装

`fav/src/emit_python.rs` に新規メソッドを追加（`emit_aws_sqs_helpers` の後）:

```rust
fn emit_pg_helpers(&mut self) {
    self.blank();
    // _pg_connect
    self.line("def _pg_connect():");
    self.indent();
    self.line("_url = _os.environ.get(\"DATABASE_URL\")");
    self.line("if _url:");
    self.indent();
    self.line("return psycopg2.connect(_url)");
    self.dedent();
    self.line("return psycopg2.connect(");
    self.indent();
    self.line("host=_os.environ.get(\"PGHOST\", \"localhost\"),");
    self.line("port=int(_os.environ.get(\"PGPORT\", \"5432\")),");
    self.line("dbname=_os.environ.get(\"PGDATABASE\", \"postgres\"),");
    self.line("user=_os.environ.get(\"PGUSER\", \"postgres\"),");
    self.line("password=_os.environ.get(\"PGPASSWORD\", \"\"),");
    self.dedent();
    self.line(")");
    self.dedent();
    self.blank();
    // _pg_execute
    self.line("def _pg_execute(_sql, _params_json):");
    self.indent();
    self.line("_params = _json_mod.loads(_params_json) if isinstance(_params_json, str) else _params_json");
    self.line("_conn = _pg_connect()");
    self.line("try:");
    self.indent();
    self.line("with _conn.cursor() as _cur:");
    self.indent();
    self.line("_cur.execute(_sql, _params)");
    self.dedent();
    self.line("_conn.commit()");
    self.line("return (\"Ok\", None)");
    self.dedent();
    self.line("except Exception as _e:");
    self.indent();
    self.line("return (\"Err\", str(_e))");
    self.dedent();
    self.line("finally:");
    self.indent();
    self.line("_conn.close()");
    self.dedent();
    self.dedent();
    self.blank();
    // _pg_query
    self.line("def _pg_query(_sql, _params_json):");
    self.indent();
    self.line("_params = _json_mod.loads(_params_json) if isinstance(_params_json, str) else _params_json");
    self.line("_conn = _pg_connect()");
    self.line("try:");
    self.indent();
    self.line("with _conn.cursor(cursor_factory=psycopg2.extras.RealDictCursor) as _cur:");
    self.indent();
    self.line("_cur.execute(_sql, _params)");
    self.line("_rows = [dict(_r) for _r in _cur.fetchall()]");
    self.dedent();
    self.line("return (\"Ok\", _json_mod.dumps(_rows))");
    self.dedent();
    self.line("except Exception as _e:");
    self.indent();
    self.line("return (\"Err\", str(_e))");
    self.dedent();
    self.line("finally:");
    self.indent();
    self.line("_conn.close()");
    self.dedent();
    self.dedent();
}
```

---

## Phase D — driver.rs pyproject.toml 更新

`cmd_transpile` の pyproject.toml 生成ブロック（`boto3_dep` の直後）に追加:

```rust
let psycopg2_dep = if py_src.contains("import psycopg2") {
    "    \"psycopg2-binary>=2.9\",\n"
} else {
    ""
};
let content = format!(
    "[project]\nname = \"transpiled\"\nversion = \"0.1.0\"\n\
     requires-python = \">=3.11\"\ndependencies = [\n{}{}]\n\n\
     [build-system]\nrequires = [\"hatchling\"]\n\
     build-backend = \"hatchling.build\"\n",
    boto3_dep, psycopg2_dep
);
```

---

## Phase E — テスト（v11600_tests）

`fav/src/driver.rs` 末尾、`v11500_tests` の後に `v11600_tests` モジュールを追加。

### テスト用 Fav コード（共通ヘルパー）

```rust
fn pg_fav_src(body: &str) -> String {
    format!(
        "fn run(sql: String) -> Result<String, String> !Postgres {{\n{}\n}}",
        body
    )
}
```

### 各テスト

```rust
#[test]
fn transpile_postgres_execute_raw() {
    // Postgres.execute_raw(sql, "[]") → _pg_execute(sql, "[]")
}

#[test]
fn transpile_postgres_query_raw() {
    // Postgres.query_raw(sql, "[]") → _pg_query(sql, "[]")
}

#[test]
fn transpile_postgres_imports_psycopg2() {
    // import psycopg2 が含まれる
}

#[test]
fn transpile_postgres_pg_connect_helper() {
    // def _pg_connect() が含まれる
}

#[test]
fn transpile_postgres_pyproject_psycopg2_dep() {
    // pyproject.toml に psycopg2-binary が含まれる
}

#[test]
fn transpile_postgres_pipeline_smoke() {
    // !Postgres パイプラインの Python 出力が psycopg2 呼び出しを含む
}
```

---

## Phase F — バージョン更新・コミット

- `fav/Cargo.toml`: `version = "11.6.0"`
- `cargo build` で `Cargo.lock` 更新
- `git commit & push` — CI 確認
