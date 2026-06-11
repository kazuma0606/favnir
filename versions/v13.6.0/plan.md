# v13.6.0 Implementation Plan

Date: 2026-06-10

---

## Phase A — `AppCtx.*` VM Primitives 追加

**ファイル**: `fav/src/backend/vm.rs`

### A-1: `AppCtx` namespace を `is_known_builtin_namespace` に追加

既に v13.5.0 で `"Ctx"` を追加済み。`"AppCtx"` も追加:

```rust
"Ctx" | "AppCtx" => true,
```

### A-2: `AppCtx.db_execute` primitive

AppCtx JSON を受け取り、db_url を取り出して `Postgres.execute_raw` に委譲。

```rust
"AppCtx.db_execute" => {
    // args: ctx_json: String, sql: String, params: String
    let ctx_json = args[0].as_str()?;
    let sql      = args[1].as_str()?;
    let params   = args[2].as_str()?;
    // parse db_url from ctx JSON
    // delegate to existing postgres_execute_raw logic
    call_builtin_postgres_execute(db_url, sql, params)
}
```

実装方針: `Postgres.execute_raw` と同じロジックを `db_url` を引数に取る内部ヘルパー
`execute_postgres(url, sql, params)` に切り出し、両方から呼ぶ。

### A-3: `AppCtx.db_query` primitive

同様に `Postgres.query_raw` に委譲。

### A-4: `AppCtx.storage_put` primitive

AppCtx JSON から `aws_region` / `s3_bucket` を取り出し `AWS.s3_put_object_raw` に委譲。

```rust
"AppCtx.storage_put" => {
    // args: ctx_json: String, bucket: String, key: String, body: String
    // extract aws_region from ctx JSON
    // delegate to existing AWS S3 logic
}
```

### A-5: `AppCtx.io_println` primitive

`IO.println_raw` に委譲（ctx は無視）:

```rust
"AppCtx.io_println" => {
    // args: ctx_json: String, msg: String
    let msg = args[1].as_str()?;
    println!("{}", msg);
    Ok(VMValue::Unit)
}
```

### A-6: `IO.getenv_raw` primitive

環境変数を取得する primitive（main での ctx 構築に使用）:

```rust
"IO.getenv_raw" => {
    // args: key: String
    // returns: Option<String>
    let key = args[0].as_str()?;
    match std::env::var(key) {
        Ok(val) => Ok(some_vm(VMValue::Str(val))),
        Err(_)  => Ok(none_vm()),
    }
}
```

**注意**: `IO.getenv_raw` が既に実装済みか確認すること。存在すれば省略。

### A-7: コンパイル確認

`cargo build` でエラーなし。

---

## Phase B — `runes/ctx/appctx.fav` 作成

**ファイル**: `fav/runes/ctx/appctx.fav`（新規）

```
// AppCtx capability wrapper Rune (v13.6.0)
// Provides ctx-based wrappers for db/storage/io operations.

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

型チェッカー向けシグネチャ: `AppCtx` → 各 capability 操作。

---

## Phase C — fav2py pipeline.fav を ctx ベースに書き換え

**ファイル**: `infra/e2e-demo/fav2py/src/pipeline.fav`

### C-1: 型定義・ヘルパー関数

変更不要（`TxnRow` / `SummaryRow` / `get_csv_path` はそのまま）。

`load_csv_rows_json` のシグネチャから `!IO` を削除（ctx 引数に移行）:
```
// 旧
fn load_csv_rows_json(path: String) -> String !IO
// 新
fn load_csv_rows_json(path: String) -> String
```
`IO.read_file_raw` は引き続き使用（ctx.io.read_file は未実装のため）。

### C-2: 各ステージのシグネチャ変更

```
// LoadAndInsert
// 旧: stage LoadAndInsert: String -> Int !IO !Postgres
// 新: stage LoadAndInsert: AppCtx -> String -> Int

// Aggregate
// 旧: stage Aggregate: Int -> String !Postgres
// 新: stage Aggregate: AppCtx -> Int -> String

// SaveResult
// 旧: stage SaveResult: String -> Unit !IO !AWS
// 新: stage SaveResult: AppCtx -> String -> Unit
```

### C-3: ステージ本体の呼び出し書き換え

- `Postgres.execute_raw(sql, params)` → `AppCtx.db_execute(ctx, sql, params)`
- `Postgres.query_raw(sql, params)` → `AppCtx.db_query(ctx, sql, params)`
- `AWS.s3_put_object_raw(bucket, key, body)` → `AppCtx.storage_put(ctx, bucket, key, body)`
- `IO.println(msg)` → `AppCtx.io_println(ctx, msg)`

### C-4: `main` 関数書き換え

```
// 旧
fn main() -> Unit !IO !Postgres !AWS {
    bind args <- IO.argv()
    Pipeline(get_csv_path(args))
}

// 新
fn main() -> Unit !IO {
    bind args <- IO.argv()
    let db_url     = Option.get_or(IO.getenv_raw("DATABASE_URL"), "")
    let aws_region = Option.get_or(IO.getenv_raw("AWS_REGION"),   "ap-northeast-1")
    let s3_bucket  = Option.get_or(IO.getenv_raw("S3_BUCKET"),    "favnir-e2e-demo")
    chain ctx <- Ctx.build_raw(db_url, aws_region, s3_bucket)
    Pipeline(ctx, get_csv_path(args))
}
```

### C-5: `seq Pipeline` 変更

```
// 旧: seq Pipeline = LoadAndInsert |> Aggregate |> SaveResult
// 新: seq Pipeline = LoadAndInsert |> Aggregate |> SaveResult
// (形式は同じ。ステージ型が AppCtx -> ... に変わっただけ)
```

---

## Phase D — airgap analyze.fav を ctx ベースに書き換え

**ファイル**: `infra/e2e-demo/airgap/src/analyze.fav`

### D-1: `read_txn_csv` シグネチャ変更

```
// 旧: fn read_txn_csv(path: String) -> List<TxnRow> !IO
// 新: fn read_txn_csv(path: String) -> List<TxnRow>
// IO.read_file_raw は引き続き使用
```

**注意**: airgap デモは `--legacy` で `Schema.adapt` を使用。
ctx 移行後も `Schema.adapt` が動作するか確認すること。

### D-2: 各ステージのシグネチャ変更

```
// LoadAll:    List<String> -> List<TxnRow> !IO    → AppCtx -> List<String> -> List<TxnRow>
// Validate:   List<TxnRow> -> List<TxnRow> !IO    → AppCtx -> List<TxnRow> -> List<TxnRow>
// WriteOutput: List<TxnRow> -> Unit !IO !AWS       → AppCtx -> List<TxnRow> -> Unit
```

### D-3: ステージ本体の呼び出し書き換え

- `IO.println(msg)` → `AppCtx.io_println(ctx, msg)`
- `AWS.s3_put_object_raw(bucket, key, body)` → `AppCtx.storage_put(ctx, bucket, key, body)`

### D-4: `main` 関数書き換え

```
// 新
fn main() -> Unit !IO {
    bind paths <- IO.argv()
    let db_url     = Option.get_or(IO.getenv_raw("DATABASE_URL"), "")
    let aws_region = Option.get_or(IO.getenv_raw("AWS_REGION"),   "ap-northeast-1")
    let s3_bucket  = Option.get_or(IO.getenv_raw("S3_BUCKET"),    "favnir-e2e-demo")
    chain ctx <- Ctx.build_raw(db_url, aws_region, s3_bucket)
    AnalyzePipeline(ctx, paths)
}
```

---

## Phase E — run.sh 更新

**ファイル**: `infra/e2e-demo/fav2py/scripts/run.sh`

`fav run --legacy pipeline.fav` → `fav run pipeline.fav`（`--legacy` 削除）。

airgap の run.sh は `--legacy` を使用しているか確認し、同様に削除。

---

## Phase F — テスト追加

**ファイル**: `fav/src/driver.rs` 末尾に `v136000_tests` モジュールを追加

テストケース:

```rust
// F-1: version bump
fn version_is_13_6_0()

// F-2: fav2py pipeline コンパイルチェック（--legacy なし）
fn e2e_fav2py_ctx_based_compiles()
// check_src で新 pipeline.fav の内容を型チェック → エラーなし

// F-3: airgap analyze コンパイルチェック（--legacy なし）
fn e2e_airgap_ctx_based_compiles()

// F-4: AppCtx.db_execute は E0007 を出さない
fn app_ctx_db_execute_no_e0007()

// F-5: AppCtx.storage_put は E0007 を出さない
fn app_ctx_storage_put_no_e0007()

// F-6: 新 pipeline に W009 が出ない
fn w009_count_fav2py_zero()

// F-7: 新 airgap に W009 が出ない
fn w009_count_airgap_zero()
```

---

## Phase G — バージョンバンプ + コミット

1. `fav/Cargo.toml` → `version = "13.6.0"`
2. `v135000_tests::version_is_13_5_0` をコメントアウト
3. `cargo test` 全件パス確認（目標: 1469 tests）
4. self-check: `./target/debug/fav check self/compiler.fav` / `self/checker.fav`
5. `git add` + `git commit -m "feat: v13.6.0 — ctx-based E2E demos + AppCtx VM primitives"`
6. `git push origin feat/v13-capability-context`

---

## 技術的注意点

### Postgres / AWS VM primitive の委譲

`AppCtx.db_execute` は `Postgres.execute_raw` と同じ TCP 接続ロジックを使う。
`Postgres.execute_raw` の実装を内部ヘルパー関数に切り出して、
`db_url` を引数として渡せるようにすることが必要。

現在の `Postgres.execute_raw` は `fav.toml` の `[postgres]` セクションから接続情報を取得している。
AppCtx primitive では、渡された `db_url` を直接使うよう別パスを実装する。

### Schema.adapt の --legacy 依存

airgap デモは `Schema.adapt` を使用しているが、これは `--legacy` モードでのみ動作するという
既知の問題（v11.0.0 時点の注記）がある。
ctx 移行後も `--legacy` なしで動作させるには、compiler.fav の `type_metas` シリアライズを
修正するか、airgap デモから `Schema.adapt` を除外する必要がある。

**対応方針**:
- まず `Schema.adapt` を除いた形で ctx 移行を確認する
- `Schema.adapt` の問題は技術的負債として残し、v13.7.0 以降で対処

### IO.getenv_raw の存在確認

`IO.getenv_raw` が vm.rs に既に実装済みか grep で確認すること。
未実装の場合は Phase A で追加する。
