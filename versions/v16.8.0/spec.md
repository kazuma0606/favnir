# v16.8.0 Spec — `tap` / `inspect` パイプライン演算子

Date: 2026-06-14

---

## 概要

seq パイプラインに `tap` と `inspect` ステップを追加する。

- `|> tap(observer_fn)` — 値を変換せずオブザーバー関数を副作用として呼び出す
- `|> inspect` — デバッグ用の組み込み tap（`io.println` で値を出力）
- `--no-tap` フラグ — 本番ビルド時に tap/inspect を Nop として除去しゼロコスト化

---

## 構文

```favnir
stage LoadCsv { ... }
stage TransformRows { ... }
stage SaveOutput { ... }

seq pipeline {
  LoadCsv
  |> tap(|rows| io.println(f"Loaded: {List.length(rows)} rows"))
  |> TransformRows
  |> inspect
  |> SaveOutput
}
```

---

## セマンティクス

### tap

```
tap<T>(observer: fn(T) -> Unit) -> fn(T) -> T
```

- 入力値 `v` を受け取る
- `observer(v)` を呼び出す（結果は無視）
- `v` をそのまま次のステージに渡す

展開イメージ（コンパイル時 IRExpr::Block に変換）:

```
{ let __tap = v; observer(__tap); __tap }
```

### inspect

`inspect` は `|row| io.println(row.to_string())` 相当の組み込み tap。
`vmvalue_repr` で文字列化して標準出力に出力する。
実装: `FlwStep::Inspect` → `vm.call_builtin("inspect_debug", [input])` 相当の `IRExpr::Block`。

### --no-tap フラグ

```bash
fav run --no-tap pipeline.fav
```

`CompileCtx.no_tap == true` の場合:
- `FlwStep::Tap(_)` → 入力をそのまま返す（identity）
- `FlwStep::Inspect` → 入力をそのまま返す（identity）

---

## 実装設計

### AST（ast.rs）

```rust
pub enum FlwStep {
    Stage(String),
    Par(Vec<String>),
    Tap(Box<Expr>),   // 追加
    Inspect,          // 追加
}
```

### Parser（parser.rs）

`parse_flw_step` で識別子が `tap` の場合:

```
tap ( <expr> )  →  FlwStep::Tap(expr)
inspect         →  FlwStep::Inspect
```

`tap` / `inspect` はソフトキーワード（新 `TokenKind` は追加しない）。
識別子 `Ident("tap")` を検出してパース。

### Compiler（compiler.rs）

`build_step_call` および `build_step_call_ctx` に追加:

```rust
FlwStep::Tap(observer_expr) => {
    if ctx.no_tap {
        input  // identity
    } else {
        let slot = ctx.fresh_local();
        IRExpr::Block(
            vec![
                IRStmt::Bind(slot, input),
                IRStmt::Expr(IRExpr::Apply(
                    compile_expr(observer_expr, ctx),
                    vec![IRExpr::Local(slot, ty.clone())],
                    Type::Unit,
                )),
            ],
            Box::new(IRExpr::Local(slot, ty.clone())),
            ty,
        )
    }
}
FlwStep::Inspect => {
    if ctx.no_tap {
        input
    } else {
        let slot = ctx.fresh_local();
        IRExpr::Block(
            vec![
                IRStmt::Bind(slot, input),
                IRStmt::Expr(IRExpr::CallBuiltin(
                    "inspect_debug".to_string(),
                    vec![IRExpr::Local(slot, ty.clone())],
                    Type::Unit,
                )),
            ],
            Box::new(IRExpr::Local(slot, ty.clone())),
            ty,
        )
    }
}
```

`flw_step_name` にも追加:

```rust
FlwStep::Tap(_) => "tap".to_string(),
FlwStep::Inspect => "inspect".to_string(),
```

### VM（vm.rs）

`inspect_debug` 組み込みプリミティブ追加:

```rust
"inspect_debug" => {
    let val = args[0].clone();
    println!("[inspect] {}", vmvalue_repr(&val));
    Ok(Value::Unit)
}
```

### CompileCtx

```rust
pub struct CompileCtx {
    // 既存フィールド...
    pub no_tap: bool,  // 追加
}
```

`no_tap` は `main.rs` の `--no-tap` フラグ解析で `cmd_run` に渡し、`compile_program_ctx` で `CompileCtx` に設定。

### driver.rs / main.rs

- `cmd_run` に `no_tap: bool` パラメータ追加
- `main.rs` に `--no-tap` フラグ解析追加

---

## exhaustive match 更新箇所

`FlwStep` を match している箇所すべてに `Tap` / `Inspect` 追加:

- `compiler.rs`: `build_step_call`, `build_step_call_ctx`, `flw_step_name`, `stage_names`, `display_str` (if any)
- `lineage.rs`: `FlwStep` match
- `driver.rs`: `FlwStep` match（`cmd_explain` / lineage 収集）
- `fmt.rs`: `FlwStep` match（あれば）
- `checker.rs`: `FlwStep` match（あれば）

---

## テスト（v168000_tests）

| テスト名 | 内容 |
|---|---|
| `version_is_16_8_0` | `Cargo.toml` に `"16.8.0"` が含まれる |
| `tap_passes_value_through` | `tap` ステップ後に同じ値が次のステージに渡る |
| `tap_calls_observer` | オブザーバー関数が呼ばれる（副作用確認） |
| `inspect_prints_debug` | `inspect` がクラッシュせず値を通す |
| `no_tap_flag_skips_observer` | `--no-tap` 時にオブザーバーが呼ばれない |

---

## ドキュメント

`site/content/docs/language/pipeline.mdx` に tap/inspect セクション追加。
