# v13.7.0 Spec — seq pipeline と ctx の統合

Date: 2026-06-10

---

## 概要

`seq Pipeline = A |> B |> C` のパイプライン定義で、
capability context（AppCtx）を各ステージに自動転送する構文を追加する。

現行の seq パイプラインは `param_count: 1`（データ引数のみ）でコンパイルされており、
ctx を持つステージを seq で扱う標準的な方法がない。
v13.7.0 でこの制限を解消する。

---

## 現状（v13.6.0）

```fav
// 現状: ctx-aware ステージを seq で扱えないため、fn + chain で手動記述
fn load_and_insert(ctx: AppCtx, path: String) -> Result<Int, String>
fn aggregate(ctx: AppCtx, n: Int) -> Result<String, String>
fn save_result(ctx: AppCtx, result: String) -> Result<Unit, String> !Gen

fn main() -> Result<Unit, String> !IO {
  chain ctx <- Ctx.build_raw(...)
  chain n      <- load_and_insert(ctx, get_csv_path(IO.argv()))
  chain result <- aggregate(ctx, n)
  save_result(ctx, result)
}
```

問題:
- ステージ数が増えると main が肥大化する
- `seq` の fail-fast（SeqStageCheck）が利用できない
- パイプライン構造が型レベルで見えない

---

## 新構文

### ctx-aware seq 定義

```fav
seq Pipeline(ctx) = load_and_insert |> aggregate |> save_result
```

- `(ctx)` は ctx を各ステージへ転送することを宣言するキーワード
- ステージ名の解決方法は現行と同じ（ファイルスコープのユーザー定義関数）

### 呼び出し

```fav
fn main() -> Result<Unit, String> !IO {
  chain ctx  <- Ctx.build_raw(
    Option.unwrap_or(IO.getenv_raw("DATABASE_URL"), ""),
    Option.unwrap_or(IO.getenv_raw("AWS_REGION"),   "ap-northeast-1"),
    Option.unwrap_or(IO.getenv_raw("S3_BUCKET"),    "favnir-e2e-demo")
  )
  Pipeline(ctx, get_csv_path(IO.argv()))
}
```

`Pipeline(ctx, data)` — 第1引数が ctx、第2引数がデータ入力。

---

## コンパイル後のセマンティクス

### ctx なし（既存、変化なし）

```
seq Pipeline = A |> B |> C
Pipeline(input)
```

コンパイル後:
```
fn Pipeline($input):
  $s0 = SeqChain: A($input)        // stage 1/3
  $s1 = SeqChain: B($s0)           // stage 2/3
  C($s1)                           // stage 3/3 (final)
```

### ctx あり（新規）

```
seq Pipeline(ctx) = A |> B |> C
Pipeline(ctx_val, input)
```

コンパイル後:
```
fn Pipeline($ctx, $input):
  $s0 = SeqChain: A($ctx, $input)   // stage 1/3
  $s1 = SeqChain: B($ctx, $s0)      // stage 2/3
  C($ctx, $s1)                      // stage 3/3 (final)
```

- `$ctx` は各ステージの第1引数として静的に転送される
- `SeqStageEnter` / `SeqStageCheck` の挙動は変更なし（ctx は単なる追加引数）
- ctx は値として全ステージに渡される（v13.6.0 時点での AppCtx は JSON 文字列）

### par ステップ（v9.13.0）

```
seq Pipeline(ctx) = par [A, B] |> Merge
```

`par [A, B]` は現行 `IO.par_execute_raw(["A", "B"], input)` にコンパイルされる。
v13.7.0 では ctx 転送を `par` ステップにも適用:
`IO.par_execute_raw(["A", "B"], input, ctx_json)` の形に拡張するか、
または `par` を含む ctx-aware FlwDef は v13.7.0 スコープ外とする（下記参照）。

---

## エラーコード

### E0022: capability missing in pipeline

```
E0022: ctx-aware pipeline requires 2 arguments (ctx, data)
  --> pipeline.fav:30:3
   |
30 | Pipeline(get_csv_path(IO.argv()))
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `Pipeline` is defined as `seq Pipeline(ctx) = ...`
   = help: call as `Pipeline(ctx, get_csv_path(IO.argv()))`
```

トリガー条件:
- `seq Name(ctx) = ...` で定義されたパイプラインを 1 引数で呼び出した場合
- 逆に `seq Name = ...`（ctx なし）を 2 引数で呼び出した場合（arity mismatch）

v13.7.0 では型レベルの capability 充足チェック（`LoadCtx` vs `AppCtx` の比較）は
Rust checker では行わない。これは v13.9.0 の型状態統合時に追加する予定。
呼び出し引数数（arity）の一致チェックのみ実装する。

---

## AST の変更

### `FlwDef` に `ctx_param` フィールドを追加

```rust
pub struct FlwDef {
    pub name: String,
    pub steps: Vec<FlwStep>,
    pub ctx_param: Option<String>,  // ← 追加: Some("ctx") if ctx-aware
    pub span: Span,
}
```

`ctx_param: None` → 既存の動作（後方互換）
`ctx_param: Some("ctx")` → ctx 転送モード

### `AbstractFlwDef` は変更なし

`AbstractFlwDef`（型パラメータ付き template FlwDef）は v13.7.0 のスコープ外。

---

## パーサーの変更

```
FlwDef = "seq" Ident ["(" Ident ")"] "=" FlwStep ("|>" FlwStep)*
```

- `["(" Ident ")"]` が追加部分
- ident はコンパイラ内部ではスロット名として扱う（`ctx` を推奨するが任意の識別子を許容）

---

## コンパイラの変更

### `compile_flw_def`

```rust
fn compile_flw_def(fd: &FlwDef, ctx: &mut CompileCtx) -> IRFnDef {
    if fd.ctx_param.is_some() {
        compile_flw_def_ctx(fd, ctx)   // 新規: ctx-aware パス
    } else {
        compile_flw_def_plain(fd, ctx) // 既存: データのみパス（リファクタ）
    }
}
```

### `compile_flw_def_ctx` (新規)

- `param_count: 2`（ctx スロット + input スロット）
- `build_step_call_ctx(step, ctx_slot, input, ctx)` → `Stage(ctx_local, input)` をエミット
- `par` ステップは v13.7.0 スコープ外としてエラー: "par steps not supported in ctx-aware pipeline"（実行時パニックまたは compile error）

---

## Rust チェッカーの変更

### E0022 チェック

`check_program` 内の call-site 解析で:
1. `Pipeline(ctx, data)` と呼ばれている FlwDef が `ctx_param: None` → E0022
2. `Pipeline(data)` と呼ばれている FlwDef が `ctx_param: Some` → E0022

実装: `check_flw_call_arity` を新規追加または既存のアリティチェックを拡張。

---

## E2E デモ更新（型チェックのみ）

### fav2py pipeline.fav

```fav
// 旧（v13.6.0）
fn main() -> Result<Unit, String> !IO {
  chain ctx    <- Ctx.build_raw(...)
  chain n      <- load_and_insert(ctx, get_csv_path(IO.argv()))
  chain result <- aggregate(ctx, n)
  save_result(ctx, result)
}

// 新（v13.7.0）
seq Pipeline(ctx) = load_and_insert |> aggregate |> save_result

fn main() -> Result<Unit, String> !IO {
  chain ctx  <- Ctx.build_raw(...)
  Pipeline(ctx, get_csv_path(IO.argv()))
}
```

### airgap analyze.fav

```fav
// 旧（v13.6.0）
fn analyze_pipeline(ctx: AppCtx, paths: List<String>) -> Unit {
  bind rows      <- load_all(ctx, paths)
  bind validated <- validate(ctx, rows)
  write_output(ctx, validated)
}

// 新（v13.7.0）
seq AnalyzePipeline(ctx) = load_all |> validate |> write_output
```

---

## スコープ外（v13.7.0 では実装しない）

- `par` ステップを含む ctx-aware FlwDef（v13.8.0 以降で検討）
- `AbstractFlwDef`（型パラメータ付き template）への ctx 対応
- capability type 充足チェック（`LoadCtx` vs `AppCtx` の比較）— v13.9.0 で実装
- checker.fav への ctx-seq 型推論追加 — 現行は Rust checker レベルのアリティチェックのみ
- E2E デモの実際の実行（PASS=5 確認）— v14.0.0 以降

---

## 後方互換性

- `ctx_param: None` の FlwDef → 既存の動作完全維持
- `seq` キーワードのパーサー変更は後方互換（`["(" Ident ")"]` はオプション）
- テスト: 既存の seq/FlwDef テストが全件パスすること
