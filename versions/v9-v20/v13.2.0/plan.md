# Favnir v13.2.0 実装計画

Date: 2026-06-09

---

## Phase A — checker.rs: 組み込み capability interface の事前登録

### A-1: `builtin_capability_interfaces()` 関数を追加

対象ファイル: `fav/src/middle/checker.rs`

`InterfaceRegistry::new()` の初期化時に `DbRead` / `DbWrite` / `StorageRead` / `StorageWrite` を
ハードコードで登録する：

```rust
fn builtin_capability_interfaces() -> Vec<(String, InterfaceDefEntry)> {
    let make_fn_ty = |param_tys: Vec<Type>, ret: Type| {
        Type::Fn(param_tys, Box::new(ret))
    };
    let str_ty    = || Type::String;
    let list_str  = || Type::List(Box::new(Type::String));
    let res_str   = || Type::Result(Box::new(Type::String), Box::new(Type::String));
    let res_int   = || Type::Result(Box::new(Type::Int),    Box::new(Type::String));
    let res_unit  = || Type::Result(Box::new(Type::Unit),   Box::new(Type::String));

    vec![
        ("DbRead", vec![
            ("query",  make_fn_ty(vec![str_ty(), list_str()], res_str())),
            ("query1", make_fn_ty(vec![str_ty(), list_str()], res_str())),
        ]),
        ("DbWrite", vec![
            ("execute", make_fn_ty(vec![str_ty(), list_str()], res_int())),
        ]),
        ("StorageRead", vec![
            ("get",  make_fn_ty(vec![str_ty(), str_ty()], res_str())),
            ("list", make_fn_ty(vec![str_ty(), str_ty()],
                     Type::Result(Box::new(list_str()), Box::new(str_ty())))),
        ]),
        ("StorageWrite", vec![
            ("put",    make_fn_ty(vec![str_ty(), str_ty(), str_ty()], res_unit())),
            ("delete", make_fn_ty(vec![str_ty(), str_ty()], res_unit())),
        ]),
    ]
    // InterfaceDefEntry { methods: HashMap, super_interface: None } に変換
}
```

`InterfaceRegistry::new()` の末尾でこれを呼び出し、登録する。

### A-2: checker.fav の組み込み型テーブルに追加

対象ファイル: `fav/self/checker.fav`

`builtin_ret_ty` / `ns_to_effect` の関数に `DbRead` / `DbWrite` / `StorageRead` / `StorageWrite` の
型情報を文字列定数として追加する。

```favnir
fn capability_interface_methods(iface: String, method: String) -> Option<String> {
  match iface {
    "DbRead" => match method {
      "query"  => Option.some("Result<String, String>")
      "query1" => Option.some("Result<String, String>")
      _        => Option.none()
    }
    "DbWrite" => match method {
      "execute" => Option.some("Result<Int, String>")
      _         => Option.none()
    }
    "StorageRead" => match method {
      "get"  => Option.some("Result<String, String>")
      "list" => Option.some("Result<List<String>, String>")
      _      => Option.none()
    }
    "StorageWrite" => match method {
      "put"    => Option.some("Result<Unit, String>")
      "delete" => Option.some("Result<Unit, String>")
      _        => Option.none()
    }
    _ => Option.none()
  }
}
```

### A-3: compiler.fav の組み込み namespace リストに追加

対象ファイル: `fav/self/compiler.fav`

capability interface 型名（`DbRead` 等）をパーサが型名として認識するため、
型パース箇所に追加（既存の `Postgres` / `AWS` 等の組み込み namespace と同列に扱う）。

---

## Phase B — checker.rs: `ctx.db.query(...)` 型チェック

### B-1: ネスト型フィールドアクセス + メソッド呼び出しの型推論

対象ファイル: `fav/src/middle/checker.rs`

`infer_expr` の `Expr::Apply` 処理で以下のパターンを追加：

```rust
// Apply(FieldAccess(FieldAccess(base_expr, field_name), method_name), args)
Expr::Apply(func_expr, args, span) => {
    if let Expr::FieldAccess(inner, method_name, _) = func_expr.as_ref() {
        if let Expr::FieldAccess(base, field_name, _) = inner.as_ref() {
            let base_ty = self.infer_expr(base, env)?;
            // base_ty が interface 型なら capability method call として処理
            if let Type::Named(iface_name) = &base_ty {
                let field_ty = self.interface_registry
                    .lookup_declared_method(iface_name, field_name);
                if let Some(inner_iface_ty) = field_ty {
                    // inner_iface_ty が interface 型なら method を lookup
                    if let Type::Named(inner_iface) = inner_iface_ty {
                        let method_ty = self.interface_registry
                            .lookup_declared_method(inner_iface, method_name);
                        // 引数型チェック + 戻り値型を返す
                    }
                }
            }
        }
    }
    // ... 既存の処理
}
```

### B-2: E0020 エラー定義

対象ファイル: `fav/src/middle/checker.rs`

E0020 は既存の `type_error` / `Diagnostic` 機構を利用：

```rust
self.type_error(
    "E0020",
    format!(
        "type `{}` does not implement interface `{}`",
        actual_ty_name, expected_iface
    ),
    span,
);
```

`get_help_text` に `"E0020"` エントリを追加：

```rust
"E0020" => &[
    "pass a value that implements the required capability interface",
    "available implementations: PostgresDb, SnowflakeDb, S3Storage, MockDb",
],
```

---

## Phase C — Rune 実装ファイルの追加

### C-1: `runes/postgres/postgres_db.fav` を追加

```favnir
type PostgresDb(String)

impl DbRead for PostgresDb {
    fn query(db: PostgresDb, sql: String, params: List<String>) -> Result<String, String> {
        Postgres.query_raw(sql, params)
    }
    fn query1(db: PostgresDb, sql: String, params: List<String>) -> Result<String, String> {
        Postgres.query_raw(sql, params)
    }
}

impl DbWrite for PostgresDb {
    fn execute(db: PostgresDb, sql: String, params: List<String>) -> Result<Int, String> {
        Postgres.execute_raw(sql, params)
    }
}
```

### C-2: `runes/aws/s3_storage.fav` を追加

```favnir
type S3Storage(String)

impl StorageRead for S3Storage {
    fn get(s: S3Storage, bucket: String, key: String) -> Result<String, String> {
        AWS.s3_get_object_raw(bucket, key)
    }
    fn list(s: S3Storage, bucket: String, prefix: String) -> Result<List<String>, String> {
        AWS.s3_list_objects_raw(bucket, prefix)
    }
}

impl StorageWrite for S3Storage {
    fn put(s: S3Storage, bucket: String, key: String, body: String) -> Result<Unit, String> {
        AWS.s3_put_object_raw(bucket, key, body)
    }
    fn delete(s: S3Storage, bucket: String, key: String) -> Result<Unit, String> {
        AWS.s3_delete_object_raw(bucket, key)
    }
}
```

### C-3: `runes/snowflake/snowflake_db.fav` を追加

```favnir
type SnowflakeDb(String)

impl DbRead for SnowflakeDb {
    fn query(db: SnowflakeDb, sql: String, params: List<String>) -> Result<String, String> {
        Snowflake.query_raw(sql)
    }
    fn query1(db: SnowflakeDb, sql: String, params: List<String>) -> Result<String, String> {
        Snowflake.query_raw(sql)
    }
}

impl DbWrite for SnowflakeDb {
    fn execute(db: SnowflakeDb, sql: String, params: List<String>) -> Result<Int, String> {
        Snowflake.execute_raw(sql)
    }
}
```

### C-4: `runes/ctx/mock_db.fav` を追加

```favnir
type MockDb(List<String>)

fn MockDb.empty() -> MockDb { MockDb(List.empty()) }
fn MockDb.seed(rows: List<String>) -> MockDb { MockDb(rows) }

impl DbRead for MockDb {
    fn query(db: MockDb, sql: String, params: List<String>) -> Result<String, String> {
        Result.ok(Json.encode_raw(db))
    }
    fn query1(db: MockDb, sql: String, params: List<String>) -> Result<String, String> {
        Result.ok(Json.encode_raw(db))
    }
}

impl DbWrite for MockDb {
    fn execute(db: MockDb, sql: String, params: List<String>) -> Result<Int, String> {
        Result.ok(0)
    }
}
```

### C-5: `runes/aws/dynamo_db.fav` を追加（stub）

DynamoDB VM primitive が未実装のため、型宣言のみ（呼び出しは実行時エラー）：

```favnir
// DynamoDB stub — 完全実装は v13.x 以降
type DynamoDb(String)

impl DbRead for DynamoDb {
    fn query(db: DynamoDb, sql: String, params: List<String>) -> Result<String, String> {
        Result.err("DynamoDB support not yet implemented")
    }
    fn query1(db: DynamoDb, sql: String, params: List<String>) -> Result<String, String> {
        Result.err("DynamoDB support not yet implemented")
    }
}

impl DbWrite for DynamoDb {
    fn execute(db: DynamoDb, sql: String, params: List<String>) -> Result<Int, String> {
        Result.err("DynamoDB support not yet implemented")
    }
}
```

### C-6: 各 Rune ファイルを `fav check` で検証

```bash
./target/debug/fav check runes/postgres/postgres_db.fav
./target/debug/fav check runes/aws/s3_storage.fav
./target/debug/fav check runes/snowflake/snowflake_db.fav
./target/debug/fav check runes/ctx/mock_db.fav
```

---

## Phase D — lint.rs: W009 deprecated 警告

### D-1: `DEPRECATED_RUNE_CALLS` 定数を定義

対象ファイル: `fav/src/lint.rs`

```rust
/// (namespace, function_name, migration_hint)
const DEPRECATED_RUNE_CALLS: &[(&str, &str, &str)] = &[
    ("Postgres",  "query_raw",          "ctx.db.query(...)"),
    ("Postgres",  "execute_raw",        "ctx.db.execute(...)"),
    ("AWS",       "s3_get_object_raw",  "ctx.storage.get(...)"),
    ("AWS",       "s3_put_object_raw",  "ctx.storage.put(...)"),
    ("AWS",       "s3_list_objects_raw","ctx.storage.list(...)"),
    ("AWS",       "s3_delete_object_raw","ctx.storage.delete(...)"),
    ("Snowflake", "query_raw",          "ctx.db.query(...)"),
    ("Snowflake", "execute_raw",        "ctx.db.execute(...)"),
];
```

### D-2: `check_deprecated_rune_calls` 関数を実装

対象ファイル: `fav/src/lint.rs`

`check_ambient_effects` と同様の AST walk で
`Apply(FieldAccess(Ident(ns), fn_name), ...)` パターンを検出。

```rust
pub fn check_deprecated_rune_calls(program: &Program) -> Vec<LintWarning> {
    let mut warnings = vec![];
    // AST walk...
    warnings
}
```

W009 の `LintWarning { code: "W009", message, hint, span }` を返す。

### D-3: `check_ambient_effects` から W009 呼び出しを統合

対象ファイル: `fav/src/driver.rs`

`cmd_check` の `ambient == true` ブロックで W008 と W009 を両方出力：

```rust
if ambient {
    let w008 = check_ambient_effects(&program);
    let w009 = check_deprecated_rune_calls(&program);
    // w008 + w009 を合わせて出力
}
```

### D-4: `get_help_text` に `"W009"` エントリを追加

```rust
"W009" => &[
    "migrate to capability interface: `chain rows <- ctx.db.query(...)`",
    "direct Rune calls will be an error in v14.0",
],
```

---

## Phase E — テスト追加

### E-1: `v132000_tests` モジュールを `driver.rs` 末尾に追加

```rust
#[cfg(test)]
mod v132000_tests {
    use super::*;

    #[test]
    fn version_is_13_2_0() {
        assert_eq!(env!("CARGO_PKG_VERSION"), "13.2.0");
    }

    #[test]
    fn db_read_interface_registered() {
        // InterfaceRegistry に "DbRead" が存在することを確認
    }

    #[test]
    fn db_read_interface_type_check() {
        // interface CommonCtx { db: DbRead }
        // fn load(ctx: CommonCtx) -> Result<String, String> {
        //   bind rows <- ctx.db.query("SELECT 1", List.empty())
        //   Result.ok(rows)
        // }
        // → type error なし
    }

    #[test]
    fn db_write_rejects_wrong_ctx() {
        // interface ReadOnly { db: DbRead }
        // fn write(ctx: ReadOnly) -> Result<Int, String> {
        //   ctx.db.execute("INSERT ...", List.empty())   ← DbRead に execute はない
        // }
        // → E0020
    }

    #[test]
    fn storage_write_put_type_check() {
        // interface WithStorage { store: StorageWrite }
        // fn save(ctx: WithStorage, body: String) -> Result<Unit, String> {
        //   ctx.store.put("bucket", "key", body)
        // }
        // → type error なし
    }

    #[test]
    fn w009_postgres_direct_deprecated() {
        // "bind rows <- Postgres.query_raw(sql, params)" + --ambient → W009
    }

    #[test]
    fn w009_no_flag_no_warning() {
        // --ambient なしでは W009 なし
    }
}
```

### E-2: バージョン更新

対象ファイル: `fav/Cargo.toml`

```toml
version = "13.2.0"
```

---

## Phase F — ビルド・テスト・コミット

### F-1: cargo build

```bash
cd fav && cargo build
```

### F-2: cargo test

```bash
cargo test
```

### F-3: self-check

```bash
./target/debug/fav check self/compiler.fav
./target/debug/fav check self/checker.fav
./target/debug/fav lint --deny-warnings self/compiler.fav
./target/debug/fav lint --deny-warnings self/checker.fav
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```

### F-4: Rune ファイル検証

```bash
./target/debug/fav check runes/postgres/postgres_db.fav
./target/debug/fav check runes/aws/s3_storage.fav
./target/debug/fav check runes/snowflake/snowflake_db.fav
./target/debug/fav check runes/ctx/mock_db.fav
```

### F-5: W009 件数確認

```bash
./target/debug/fav check --ambient self/compiler.fav
./target/debug/fav check --ambient self/checker.fav
```

### F-6: git commit + push

```bash
git add -p
git commit -m "feat: v13.2.0 — DbRead/DbWrite/StorageRead/StorageWrite capability interface"
git push
```

### F-7: CI 確認

```bash
gh run watch
```

---

## 実装上の注意

### 1. `InterfaceRegistry` への事前登録タイミング

`InterfaceRegistry::new()` 時点でユーザーコードをまだ読んでいないため、
`InterfaceDef` を構造体として直接生成できる。
`super_interface: None` を設定（capability interface は継承を持たない）。

### 2. `ctx.db.query(...)` パターン検出の注意

ネストが 3 レベル（`Apply(FieldAccess(FieldAccess(base, field), method), args)`）になるため、
既存の `Apply(FieldAccess(Ident(ns), fn), args)` パターンとの優先順位に注意。
`FieldAccess` の内側が `Ident` か `FieldAccess` かで分岐する。

### 3. Rune ファイルの `impl NS for Type` 構文

既存の `impl Interface for Type { fn ... }` 構文は v9.12.0 で追加済み。
Rune ファイル内で `Postgres.query_raw` 等の既存 VM primitive を呼ぶため、
`!Postgres` エフェクト宣言が必要か確認する（必要なら `!Postgres` を追加）。

### 4. `runes/ctx/` ディレクトリ

`runes/ctx/` は新規ディレクトリ。`runes/ctx/mock_db.fav` の他、
v13.5.0 で `AppCtx` / `Ctx.build` / `Ctx.mock` を追加予定の場所。

### 5. W009 は `fav lint` に含めない

W008 と同様に `--ambient` 専用。
`lint_program` は変更しない。

### 6. `AWS.s3_list_objects_raw` の存在確認

`s3_list_objects_raw` が VM primitive として存在するか `vm.rs` で確認してから実装すること。
未実装の場合は stub として `Result.err("not implemented")` で返す。
