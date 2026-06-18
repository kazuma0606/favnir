# v16.8.0 Plan — `tap` / `inspect` パイプライン演算子

Date: 2026-06-14

---

## Phase A — Cargo バージョン更新

`fav/Cargo.toml` の `version` を `"16.8.0"` に変更。
`cargo build` → コンパイルエラーなし確認。

---

## Phase B — AST: `FlwStep::Tap` / `FlwStep::Inspect` 追加（ast.rs）

`fav/src/ast.rs` の `FlwStep` enum に 2 variant 追加:

```rust
Tap(Box<Expr>),
Inspect,
```

`cargo build` → exhaustive match エラーが出ることを確認（後続 Phase で対処）。

---

## Phase C — Parser: `tap(expr)` / `inspect` パース（parser.rs）

`parse_flw_step` でソフトキーワード検出:

- `Ident("tap")` → `(` `<expr>` `)` を消費 → `FlwStep::Tap(expr)`
- `Ident("inspect")` → `FlwStep::Inspect`

新 `TokenKind` は追加しない（ソフトキーワード）。
`cargo build` → コンパイルエラーなし確認。

---

## Phase D — VM: `inspect_debug` プリミティブ追加（vm.rs）

`vm_call_builtin` に `"inspect_debug"` 追加:

```rust
"inspect_debug" => {
    let val = args[0].clone();
    println!("[inspect] {}", vmvalue_repr(&val));
    Ok(Value::Unit)
}
```

compiler.rs のグローバル builtin 名前テーブル（2 箇所）に `"inspect_debug"` 追加。
`cargo build` → コンパイルエラーなし確認。

---

## Phase E — CompileCtx: `no_tap` フィールド追加（compiler.rs）

`CompileCtx` に `pub no_tap: bool` フィールド追加。
`CompileCtx::new()` / `CompileCtx::default()` 等の初期化箇所に `no_tap: false` 追加。
`cargo build` → コンパイルエラーなし確認。

---

## Phase F — Compiler: `FlwStep::Tap` / `FlwStep::Inspect` コンパイル（compiler.rs）

exhaustive match を更新する箇所:

1. `flw_step_name` — `Tap(_) => "tap"`, `Inspect => "inspect"`
2. `stage_names` / `display_str` — skip（Tap/Inspect はステージ名に含めない）
3. `build_step_call` — `Tap(observer)` → `IRExpr::Block`, `Inspect` → `IRExpr::Block` with `inspect_debug`
4. `build_step_call_ctx` — 同上（ctx-aware 版）

`no_tap == true` の場合は identity（入力をそのまま返す）。
`cargo build` → コンパイルエラーなし確認。

---

## Phase G — lineage.rs / driver.rs / fmt.rs exhaustive match

`FlwStep` を match している残りの箇所に `Tap(..) | Inspect => { /* skip or handle */ }` 追加:

- `lineage.rs` の `collect_lineage_steps`
- `driver.rs` の explain/lineage 収集
- `fmt.rs`（FlwStep フォーマット）
- `checker.rs`（あれば）

`cargo build` → コンパイルエラーなし確認。

---

## Phase H — driver.rs / main.rs: `--no-tap` フラグ

- `cmd_run` に `no_tap: bool` パラメータ追加
- `cmd_run` から `compile_program_ctx` に `no_tap` を渡す
- `main.rs` に `--no-tap` フラグ解析追加（`fav run --no-tap`）

`cargo build` → コンパイルエラーなし確認。

---

## Phase I — テスト追加（v168000_tests）

`fav/src/driver.rs` に `v168000_tests` モジュール追加:

1. `version_is_16_8_0`
2. `tap_passes_value_through`
3. `tap_calls_observer`
4. `inspect_prints_debug`
5. `no_tap_flag_skips_observer`

`cargo test v168000` → 5/5 PASS 確認。

---

## Phase J — ドキュメント + コミット

- `site/content/docs/language/pipeline.mdx` に tap/inspect セクション追加
- `cargo test v168000` → 5/5 PASS 最終確認
- `cargo test` → 全件 PASS（リグレッションなし）
- コミット

---

## 依存関係

```
A → B → C → D → E → F → G → H → I → J
```

各 Phase は前の Phase の `cargo build` 成功を前提とする。
