# v13.5.0 Spec — AppCtx 具象型 + `Ctx.build` / `Ctx.mock` Rune 実装

Date: 2026-06-10
Branch: feat/v13-capability-context

---

## テーマ

本番・テスト双方で使えるコンテキスト組み立て標準を提供する。

v13.4.0 で `LoadCtx` / `WriteCtx` / `MigrateCtx` interface が揃った。
本バージョンでは、それら全 interface を満たす具象型 `AppCtx` と、
`Ctx.build(env)` / `Ctx.mock(...)` による組み立てパターンを実装する。

---

## 1. `AppCtx` 具象型

```
type AppCtx {
    db:      DbWrite    // DbRead も満たす（DbWrite ⊇ DbRead と見なす）
    storage: StorageWrite
    http:    HttpClient
    io:      Io
    env:     Env
}
```

`AppCtx` は全 capability interface を包含するレコード型。
- `fn load(ctx: LoadCtx)` に `AppCtx` を渡せる（`AppCtx` は `LoadCtx` を実装）
- `fn write(ctx: WriteCtx)` に `AppCtx` を渡せる（`AppCtx` は `WriteCtx` を実装）
- `fn migrate(ctx: MigrateCtx)` に `AppCtx` を渡せる

### impl 登録

| 型 | 実装する interface |
|---|---|
| `AppCtx` | `CommonCtx` |
| `AppCtx` | `LoadCtx` |
| `AppCtx` | `WriteCtx` |
| `AppCtx` | `MigrateCtx` |
| `MockDb` | `DbRead` |
| `MockDb` | `DbWrite` |
| `MockStorage` | `StorageWrite` |

---

## 2. `AppCtx` の型チェック統合

`fn load(ctx: LoadCtx)` に `AppCtx` 型の値を渡したとき、
型チェッカーが impl テーブルを参照して compatible と判断する。

現在の挙動:
- `fn f(ctx: SomeInterface)` に `Named("AppCtx")` を渡すと型エラー

追加する挙動:
- `Named` 型を `Interface` 型パラメータに渡す際、impl テーブルを照合
- `(AppCtx, LoadCtx)` エントリが存在すれば OK
- 存在しない場合は既存エラー

---

## 3. `Ctx` Rune（`runes/ctx/ctx.fav`）

### `Ctx.build(env: Env) -> Result<AppCtx, String>`

1. `env.require("DATABASE_URL")` で DB 接続文字列を取得
2. `env.require("AWS_REGION")` / `env.get("S3_BUCKET")` で Storage 設定を取得
3. PostgresDb / S3Storage / HttpClientImpl / IoImpl / EnvImpl を組み立てて AppCtx を返す
4. 環境変数未設定の場合は `Result.err("missing env: DATABASE_URL")` を返す

```
public fn build(env: Env) -> Result<AppCtx, String> {
    chain db_url     <- env.require("DATABASE_URL")
    chain aws_region <- env.require("AWS_REGION")
    let bucket = Option.get_or(env.get("S3_BUCKET"), "default-bucket")
    Result.ok(AppCtx {
        db:      PostgresDb(db_url),
        storage: S3Storage(bucket),
        http:    HttpClientImpl {},
        io:      IoImpl {},
        env:     env
    })
}
```

### `Ctx.mock(db, storage, io) -> AppCtx`

テスト専用コンストラクタ。本番の Env / Http も自動で Mock 化。

```
public fn mock(db: MockDb, storage: MockStorage, io: MockIo) -> AppCtx {
    AppCtx {
        db:      db,
        storage: storage,
        http:    MockHttp {},
        io:      io,
        env:     MockEnv {}
    }
}
```

---

## 4. `MockDb` / `MockStorage` 型

### `MockDb`（`runes/ctx/mock_db.fav`）

既に v13.2.0 で作成済み（spec 再確認）。
本バージョンでは `DbWrite` にも対応させる。

```
type MockDb(List<String>)

fn MockDb.empty() -> MockDb { MockDb(List.empty()) }
fn MockDb.seed(rows: List<String>) -> MockDb { MockDb(rows) }

impl DbRead for MockDb:
    query(sql, params) -> Result<String, String>: シードデータを JSON で返す
    query1(sql, params) -> Result<String, String>: 先頭行を返す

impl DbWrite for MockDb:
    execute(sql, params) -> Result<Int, String>: Result.ok(0)
```

### `MockStorage`（`runes/ctx/mock_storage.fav`）

```
type MockStorage(Map<String, String>)

fn MockStorage.empty() -> MockStorage { MockStorage(Map.empty()) }

impl StorageWrite for MockStorage:
    put(bucket, key, body) -> Result<Unit, String>: Result.ok(Unit)
    delete(bucket, key) -> Result<Unit, String>: Result.ok(Unit)
```

---

## 5. `fav.toml` `[context]` セクション

```toml
[context]
db_url     = "${DATABASE_URL}"
storage    = "s3"
http       = "ureq"
```

`fav run` 実行時に `[context]` が存在すれば `Ctx.build` に渡す設定として読み込む。
環境変数展開（`${VAR}`）は既存の `expand_env_vars` で処理。

`ContextConfig` 構造体を `fav/src/toml.rs` に追加:
```rust
pub struct ContextConfig {
    pub db_url:  Option<String>,
    pub storage: Option<String>,
    pub http:    Option<String>,
}
```

---

## 6. テスト

| テスト名 | 内容 |
|---|---|
| `version_is_13_5_0` | Cargo.toml バージョン確認 |
| `app_ctx_registered` | InterfaceRegistry に AppCtx impl エントリが存在 |
| `mock_db_implements_db_read` | `fn f(ctx: DbRead)` に MockDb を渡してもエラーなし |
| `mock_db_implements_db_write` | `fn f(ctx: DbWrite)` に MockDb を渡してもエラーなし |
| `app_ctx_satisfies_load_ctx` | `fn load(ctx: LoadCtx)` に AppCtx を渡してもエラーなし |
| `app_ctx_satisfies_write_ctx` | `fn write(ctx: WriteCtx)` に AppCtx を渡してもエラーなし |
| `ctx_build_missing_db_url_returns_err` | `Ctx.build` が DATABASE_URL 未設定で Err を返す |
| `ctx_mock_db_query_returns_seeded` | MockDb.seed + DbRead.query で種データが返る |
| `ctx_mock_io_capture_output` | MockIo（IoCapture）でキャプチャが動作 |
| `context_toml_config_parsed` | `fav.toml` の `[context]` が ContextConfig として解析される |
