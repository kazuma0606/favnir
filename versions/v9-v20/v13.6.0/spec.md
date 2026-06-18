# v13.6.0 Spec — 既存 Rune の ctx ベース版並行提供 + E2E デモ更新

Date: 2026-06-10
Branch: feat/v13-capability-context

---

## テーマ

旧エフェクト API（`!IO !Postgres !AWS`）と新 capability API（ctx 引数）を並行運用し、
E2E デモの移行によって移行パスを実証する。

v13.5.0 で `AppCtx` と `Ctx.build/Ctx.mock` の型定義・VM primitive が揃った。
本バージョンでは、実際に動く E2E デモを ctx ベースで書き直すことで、
「旧コードから新コードへの移行がどのように見えるか」を確立する。

---

## 1. 移行対象

### 1-1. `infra/e2e-demo/fav2py/src/pipeline.fav`

**旧**:
```
stage LoadAndInsert: String -> Int !IO !Postgres = |path| {
    bind _ <- Postgres.execute_raw("CREATE TABLE ...", "[]")
    bind _ <- IO.println("[INFO] done")
    0
}

fn main() -> Unit !IO !Postgres !AWS {
    bind args <- IO.argv()
    Pipeline(get_csv_path(args))
}
```

**新**:
```
stage LoadAndInsert: AppCtx -> String -> Int = |ctx, path| {
    bind _ <- AppCtx.db_execute(ctx, "CREATE TABLE ...", "[]")
    bind _ <- AppCtx.io_println(ctx, "[INFO] done")
    0
}

fn main() -> Unit !IO !AWS {
    bind args <- IO.argv()
    chain ctx  <- Ctx.build_raw(
        AppCtx.env_get(IO.getenv_raw("DATABASE_URL"), ""),
        AppCtx.env_get(IO.getenv_raw("AWS_REGION"), "ap-northeast-1"),
        AppCtx.env_get(IO.getenv_raw("S3_BUCKET"), "favnir-e2e-demo")
    )
    Pipeline(ctx, get_csv_path(args))
}
```

### 1-2. `infra/e2e-demo/airgap/src/analyze.fav`

同様に `!IO !AWS` → `AppCtx` 引数に移行。各ステージが `ctx` を受け取る形に変更。

---

## 2. VM Runtime 対応：AppCtx ラッパー Primitive

ctx ベースの pipeline が実際に動くには、
`ctx.db.execute(...)` の実行時ディスパッチが必要。

現時点では型チェッカーは `AppCtx` を知っているが、
VM は `AppCtx` の内容（db_url など）にアクセスする手段を持っていない。

**方針**: `AppCtx.*` VM primitive を追加し、AppCtx JSON から設定を取り出して
既存の `Postgres.*` / `AWS.*` / `IO.*` primitive に委譲する。

| VM primitive | 動作 |
|---|---|
| `AppCtx.db_execute(ctx, sql, params)` | AppCtx JSON から db_url を取り出し `Postgres.execute_raw` に委譲 |
| `AppCtx.db_query(ctx, sql, params)` | 同様に `Postgres.query_raw` に委譲 |
| `AppCtx.storage_put(ctx, bucket, key, body)` | AppCtx JSON から aws_region / bucket を取り出し `AWS.s3_put_object_raw` に委譲 |
| `AppCtx.io_println(ctx, msg)` | `IO.println_raw` に委譲（ctx 引数は無視） |
| `AppCtx.env_get(val, default)` | `Option.get_or` 相当（None → default、Some(v) → v）|

AppCtx JSON 形式（`Ctx.build_raw` が返す）:
```json
{
    "type": "AppCtx",
    "db_url":     "postgres://...",
    "aws_region": "ap-northeast-1",
    "s3_bucket":  "favnir-e2e-demo"
}
```

---

## 3. Rune ラッパー（`runes/ctx/appctx.fav`）

型チェッカー向けのシグネチャを提供する Rune。
VM primitive の `AppCtx.*` を Favnir 関数としてラップする。

```
// runes/ctx/appctx.fav

public fn db_execute(ctx: AppCtx, sql: String, params: String) -> Result<Int, String> {
    AppCtx.db_execute(ctx, sql, params)
}

public fn db_query(ctx: AppCtx, sql: String, params: String) -> Result<String, String> {
    AppCtx.db_query(ctx, sql, params)
}

public fn storage_put(ctx: AppCtx, bucket: String, key: String, body: String) -> Result<Unit, String> {
    AppCtx.storage_put(ctx, bucket, key, body)
}

public fn io_println(ctx: AppCtx, msg: String) -> Unit {
    AppCtx.io_println(ctx, msg)
}
```

---

## 4. pipeline.fav の書き換え詳細

### fav2py pipeline（新）

```
// 新: ctx を受け取る形
fn load_csv_rows_json(ctx: AppCtx, path: String) -> String {
    match IO.read_file_raw(path) {
        Err(_)   => "[]"
        Ok(text) => match Csv.parse_raw(text, ",", true) {
            Err(_)   => "[]"
            Ok(rows) => Schema.to_json_array(rows, "TxnRow")
        }
    }
}

stage LoadAndInsert: AppCtx -> String -> Int = |ctx, path| {
    let rows_json = load_csv_rows_json(ctx, path)
    bind _ <- AppCtx.db_execute(ctx, "CREATE TABLE IF NOT EXISTS txn ...", "[]")
    bind _ <- AppCtx.db_execute(ctx, "DELETE FROM txn", "[]")
    bind _ <- AppCtx.db_execute(ctx, String.concat("INSERT INTO txn ... '", String.concat(rows_json, "' ...")), "[]")
    bind _ <- AppCtx.io_println(ctx, "[INFO] LoadAndInsert complete")
    0
}

stage Aggregate: AppCtx -> Int -> String = |ctx, n| {
    match AppCtx.db_query(ctx, "SELECT region, ...", "[]") {
        Ok(json) => json
        Err(e)   => String.concat("[ERROR] Aggregate: ", e)
    }
}

stage SaveResult: AppCtx -> String -> Unit = |ctx, result_json| {
    bind uid <- Gen.uuid_raw()
    bind _ <- AppCtx.io_println(ctx, String.concat("[INFO] Saving to S3 key: ", uid))
    match AppCtx.storage_put(ctx, "favnir-e2e-demo", String.concat("proof/fav2py/", String.concat(uid, ".json")), result_json) {
        Ok(_)  => AppCtx.io_println(ctx, "[INFO] Saved to S3 successfully")
        Err(e) => AppCtx.io_println(ctx, "[ERROR] S3 failed")
    }
}

seq Pipeline = LoadAndInsert |> Aggregate |> SaveResult

fn main() -> Unit !IO {
    bind args <- IO.argv()
    let db_url     = Option.get_or(IO.getenv_raw("DATABASE_URL"), "")
    let aws_region = Option.get_or(IO.getenv_raw("AWS_REGION"),   "ap-northeast-1")
    let s3_bucket  = Option.get_or(IO.getenv_raw("S3_BUCKET"),    "favnir-e2e-demo")
    chain ctx <- Ctx.build_raw(db_url, aws_region, s3_bucket)
    Pipeline(ctx, get_csv_path(args))
}
```

`--legacy` フラグ不要。`!IO` のみ（`Ctx.build_raw` の外側の環境変数読み取りに使用）。

### airgap analyze（新）

同様のパターンで `LoadAll` / `Validate` / `WriteOutput` ステージに `ctx: AppCtx` を追加。
`IO.println` → `AppCtx.io_println(ctx, ...)` に置き換え。
`AWS.s3_put_object_raw` → `AppCtx.storage_put(ctx, ...)` に置き換え。

---

## 5. run.sh 更新

`infra/e2e-demo/fav2py/scripts/run.sh`:
- `fav run --legacy pipeline.fav` → `fav run pipeline.fav`（`--legacy` 削除）
- 環境変数は引き続き `DATABASE_URL` / `AWS_REGION` / `S3_BUCKET` から取得（変更なし）

---

## 6. W009 チェック

```bash
./target/debug/fav check --strict infra/e2e-demo/fav2py/src/pipeline.fav
./target/debug/fav check --strict infra/e2e-demo/airgap/src/analyze.fav
```

`--strict` オプション: W009（direct Rune call deprecated）があればエラー終了。
新 pipeline は `Postgres.*` / `AWS.*` の直接呼び出しを一切含まないため W009 = 0。

**注意**: `--strict` フラグが未実装の場合は、W009 カウント = 0 を確認するだけでもよい。

---

## 7. テスト

| テスト名 | 内容 |
|---|---|
| `version_is_13_6_0` | Cargo.toml バージョン確認 |
| `e2e_fav2py_ctx_based_compiles` | 新 fav2py pipeline.fav が `--legacy` なしでコンパイルできる |
| `e2e_airgap_ctx_based_compiles` | 新 airgap analyze.fav が `--legacy` なしでコンパイルできる |
| `app_ctx_db_execute_primitive_exists` | `AppCtx.db_execute` が既知 VM primitive として認識される |
| `app_ctx_storage_put_primitive_exists` | `AppCtx.storage_put` が既知 VM primitive として認識される |
| `w009_count_fav2py_zero` | 新 fav2py pipeline に W009 警告が出ない |
| `w009_count_airgap_zero` | 新 airgap analyze に W009 警告が出ない |
