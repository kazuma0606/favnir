# v13.5.0 Implementation Plan

Date: 2026-06-10

---

## Phase A — `AppCtx` を型チェッカーに登録

**ファイル**: `fav/src/middle/checker.rs`

### A-1: `AppCtx` を InterfaceRegistry の impl テーブルに登録

`register_builtin_capabilities` の末尾（MigrateCtx 登録後）に追加:

```rust
// v13.5.0: AppCtx satisfies all context interfaces
// AppCtx → CommonCtx
self.impls.insert(
    ("AppCtx".into(), "CommonCtx".into()),
    InterfaceImplEntry { methods: HashMap::new(), is_auto: true },
);
// AppCtx → LoadCtx
self.impls.insert(
    ("AppCtx".into(), "LoadCtx".into()),
    InterfaceImplEntry { methods: HashMap::new(), is_auto: true },
);
// AppCtx → WriteCtx
self.impls.insert(
    ("AppCtx".into(), "WriteCtx".into()),
    InterfaceImplEntry { methods: HashMap::new(), is_auto: true },
);
// AppCtx → MigrateCtx
self.impls.insert(
    ("AppCtx".into(), "MigrateCtx".into()),
    InterfaceImplEntry { methods: HashMap::new(), is_auto: true },
);
```

### A-2: `MockDb` / `MockStorage` の impl 登録

```rust
// MockDb → DbRead
self.impls.insert(("MockDb".into(), "DbRead".into()),
    InterfaceImplEntry { methods: HashMap::new(), is_auto: true });
// MockDb → DbWrite
self.impls.insert(("MockDb".into(), "DbWrite".into()),
    InterfaceImplEntry { methods: HashMap::new(), is_auto: true });
// MockStorage → StorageWrite
self.impls.insert(("MockStorage".into(), "StorageWrite".into()),
    InterfaceImplEntry { methods: HashMap::new(), is_auto: true });
```

### A-3: `Named` 型 → `Interface` 型の互換チェック

`infer_apply` または `check_arg_type` の型互換チェック箇所に追加:

```rust
// Named(ty_name) 型の引数を Interface(iface_name, []) パラメータに渡す場合、
// impls テーブルに (ty_name, iface_name) エントリがあれば OK
if let (Type::Named(ty_name, _), Type::Interface(iface_name, args)) = (arg_ty, param_ty) {
    if args.is_empty() {
        if self.interface_registry.impls.contains_key(&(ty_name.clone(), iface_name.clone())) {
            return; // compatible
        }
    }
}
```

### A-4: コンパイル確認

`cargo build` でエラーなし。

---

## Phase B — VM Primitives（`Ctx.build_raw` / `Ctx.mock_raw`）

**ファイル**: `fav/src/vm.rs`

### B-1: `Ctx.build_raw` primitive 追加

引数: `db_url: String, aws_region: String, s3_bucket: String`
戻り値: `Result<String, String>`（AppCtx を JSON 表現で返す）

```rust
("Ctx", "build_raw") => {
    let db_url     = args[0].as_string()?;
    let aws_region = args[1].as_string()?;
    let s3_bucket  = args[2].as_string()?;
    // Validate DB connection (lightweight check)
    if db_url.is_empty() {
        return Ok(Value::err_str("missing env: DATABASE_URL"));
    }
    let ctx_json = serde_json::json!({
        "db_url": db_url,
        "aws_region": aws_region,
        "s3_bucket": s3_bucket,
        "type": "AppCtx"
    }).to_string();
    Ok(Value::ok_str(ctx_json))
}
```

### B-2: `Ctx.mock_raw` primitive 追加

引数: `seed_rows: List<String>`（MockDb のシードデータ）
戻り値: `String`（MockCtx の JSON 表現）

```rust
("Ctx", "mock_raw") => {
    let rows = args[0].as_list()?;
    let mock_json = serde_json::json!({
        "type": "MockAppCtx",
        "seed_rows": rows,
    }).to_string();
    Ok(Value::ok_str(mock_json))
}
```

### B-3: `Ctx` namespace を `vm.rs` のディスパッチに追加

既存の `("Postgres", ...)` / `("IO", ...)` 等のパターンに倣い、`("Ctx", "build_raw")` / `("Ctx", "mock_raw")` ケースを追加。

---

## Phase C — Rune ファイル作成

### C-1: `fav/runes/ctx/ctx.fav`

```
import MockDb from "ctx/mock_db"
import MockEnv from "env/mock_env"

public fn build(env: Env) -> Result<AppCtx, String> {
    chain db_url     <- env.require("DATABASE_URL")
    chain aws_region <- env.require("AWS_REGION")
    let bucket = Option.get_or(env.get("S3_BUCKET"), "default-bucket")
    bind _ <- Ctx.build_raw(db_url, aws_region, bucket)
    Result.ok(AppCtx { ... })
}

public fn mock(db: MockDb, storage: MockStorage, io: Io) -> AppCtx {
    ...
}
```

**注意**: `AppCtx` はレコード型として .fav ソース側では定義せず、
型チェッカーに組み込みとして登録することで実現する（Phase A）。
Rune ファイルは `Ctx.build_raw` を呼び出すラッパーとして機能する。

### C-2: `fav/runes/ctx/mock_db.fav`

既存の v13.2.0 mock_db.fav を `DbWrite` にも対応させる:

```
type MockDb(List<String>)

public fn empty() -> MockDb { MockDb(List.empty()) }
public fn seed(rows: List<String>) -> MockDb { MockDb(rows) }

impl DbRead for MockDb {
    query  = |db, sql, params| Result.ok(Json.encode_raw(MockDb.rows(db)))
    query1 = |db, sql, params| {
        bind rows <- Result.ok(MockDb.rows(db))
        match List.first(rows) {
            Some(r) => Result.ok(r)
            None    => Result.err("no rows")
        }
    }
}

impl DbWrite for MockDb {
    execute = |db, sql, params| Result.ok(0)
}
```

### C-3: `fav/runes/ctx/mock_storage.fav`

```
type MockStorage(List<String>)

public fn empty() -> MockStorage { MockStorage(List.empty()) }

impl StorageWrite for MockStorage {
    put    = |_, bucket, key, body| Result.ok(Unit)
    delete = |_, bucket, key|      Result.ok(Unit)
}
```

### C-4: `fav check` で Rune ファイルを確認

```bash
./target/debug/fav check runes/ctx/ctx.fav
./target/debug/fav check runes/ctx/mock_db.fav
./target/debug/fav check runes/ctx/mock_storage.fav
```

---

## Phase D — `fav.toml` `[context]` セクション

**ファイル**: `fav/src/toml.rs`

### D-1: `ContextConfig` 構造体追加

```rust
#[derive(Deserialize, Default)]
pub struct ContextConfig {
    pub db_url:  Option<String>,
    pub storage: Option<String>,
    pub http:    Option<String>,
}
```

### D-2: `FavToml` に `context` フィールドを追加

```rust
pub struct FavToml {
    // 既存フィールド...
    pub context: Option<ContextConfig>,
}
```

### D-3: 環境変数展開

既存の `expand_env_vars` を `ContextConfig` のフィールドにも適用。

---

## Phase E — テスト

**ファイル**: `fav/src/driver.rs` 末尾に `v135000_tests` モジュールを追加

テストケース一覧:
- `version_is_13_5_0`
- `app_ctx_impl_entries_registered` — impls テーブルに `(AppCtx, LoadCtx)` 等が存在
- `mock_db_implements_db_read` — `fn f(ctx: DbRead)` に MockDb を渡してエラーなし
- `mock_db_implements_db_write` — `fn f(ctx: DbWrite)` に MockDb を渡してエラーなし
- `app_ctx_satisfies_load_ctx` — `fn load(ctx: LoadCtx)` に AppCtx を渡してエラーなし
- `app_ctx_satisfies_write_ctx` — `fn write(ctx: WriteCtx)` に AppCtx を渡してエラーなし
- `ctx_build_missing_db_url_returns_err` — `Ctx.build_raw("")` → Err
- `ctx_mock_db_query_type_check` — MockDb + DbRead.query が型チェックを通る
- `context_toml_section_parsed` — `[context]` セクションが ContextConfig として解析される

---

## Phase F — バージョンバンプ + コミット

1. `fav/Cargo.toml` → `version = "13.5.0"`
2. `v134000_tests::version_is_13_4_0` をコメントアウト
3. `cargo test -- --test-threads=1` 全件パス確認
4. `git add` + `git commit -m "feat: v13.5.0 — AppCtx + Ctx.build/Ctx.mock + MockDb/MockStorage"`
5. `git push origin feat/v13-capability-context`

---

## 技術的注意点

### Named 型 → Interface 型の互換チェック箇所

現在の型チェッカーは `fn f(ctx: LoadCtx)` に `LoadCtx` 型の値のみ受け入れる。
`AppCtx` 型の値を渡すには、以下のいずれかの方法が必要:

1. **型引数での解決（推奨）**: `fn f(ctx: T) where T: LoadCtx` のような記法（v13.7.0 以降）
2. **impl テーブルによる互換チェック（今回の方針）**: 引数型チェック時に impl テーブルを参照

今回は (2) を採用し、`resolve_call_arg_type` または `infer_apply` の引数型チェック箇所に
「Named 型が Interface 型パラメータを満たすかを impl テーブルで確認」するロジックを追加する。

具体的には `check_arg_satisfies_interface` 関数（存在しない場合は新規追加）を実装:

```rust
fn arg_satisfies_interface(&self, arg_ty: &Type, param_ty: &Type) -> bool {
    match (arg_ty, param_ty) {
        (Type::Named(ty, _), Type::Interface(iface, args)) if args.is_empty() => {
            self.interface_registry.impls.contains_key(&(ty.clone(), iface.clone()))
        }
        _ => false,
    }
}
```

### AppCtx の型表現

型チェッカー内部では `Type::Named("AppCtx", vec![])` として扱う。
.fav ソースファイルでは `AppCtx` は型名として参照可能（組み込み登録）。
