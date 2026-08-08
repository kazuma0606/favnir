# v62.8.0 Spec — AOT エラーコード E0427（AOT 未サポート機能検出）

Version: 62.8.0
Status: 未着手
Base tests: 3397
Target tests: 3399

---

## 概要

AOT コンパイルパスで未サポート機能（`IRExpr::Emit` によるエフェクト発行等）を検出して
エラーコード **E0427** を発行するバリデーターを追加する。
`error_catalog.rs` に E0427 を登録し、`fav explain E0427` で詳細が参照できるようにする。

```
E0427: unsupported feature in AOT mode
 --> pipeline.fav:5:3
  |
5 |   emit "order_placed"
  |   ^^^^^^^^^^^^^^^^^^^ emit は AOT コンパイルではサポートされていません
  |
  help: `fav build` の代わりに `fav run` を使用するか、emit 式を除去してください。
```

---

## 前提確認（T0 で実施）

- `cranelift_aot.rs` に `validate_aot_compat` が **存在しない** ことを確認
- `error_catalog.rs` に `E0427` が **存在しない** ことを確認
- `driver.rs` に `cmd_build_aot_validate` が **存在しない** ことを確認
- `driver.rs` に `v62700_tests` が存在することを確認（挿入位置確認）
- `cargo test -j 8 -- --test-threads=8` でベース 3397 tests passed, 0 failed を確認

---

## 実装スコープ

### 1. `cranelift_aot.rs` — `validate_aot_compat` + `contains_aot_unsupported` 追加

**`contains_aot_unsupported(expr: &IRExpr) -> bool`**（private fn）:

AOT でサポートされていない IR 式を再帰的に検出する。

```rust
fn contains_aot_unsupported(expr: &IRExpr) -> bool {
    match expr {
        IRExpr::Emit(_, _) => true,  // エフェクト発行は AOT 未サポート
        IRExpr::BinOp(_, lhs, rhs, _) => {
            contains_aot_unsupported(lhs) || contains_aot_unsupported(rhs)
        }
        IRExpr::If(cond, then_e, else_e, _) => {
            contains_aot_unsupported(cond)
                || contains_aot_unsupported(then_e)
                || contains_aot_unsupported(else_e)
        }
        IRExpr::Block(stmts, final_expr, _) => {
            stmts.iter().any(|s| match s {
                IRStmt::Bind(_, e)
                | IRStmt::LegacyBind(_, e)
                | IRStmt::Chain(_, e)
                | IRStmt::Yield(e)
                | IRStmt::Return(e)
                | IRStmt::Expr(e) => contains_aot_unsupported(e),
                IRStmt::SeqChain { expr, .. } => contains_aot_unsupported(expr),
                IRStmt::TrackLine(_) => false,
                IRStmt::RefinementAssert { expr, .. } => contains_aot_unsupported(expr),
            }) || contains_aot_unsupported(final_expr)
        }
        _ => false,
    }
}
```

**`validate_aot_compat(ir: &IRProgram) -> Vec<String>`**（pub fn）:

IR 内の全関数を走査し、AOT 未サポート機能を含む関数ごとに E0427 エラーメッセージを返す。

```rust
pub fn validate_aot_compat(ir: &IRProgram) -> Vec<String> {
    let mut errors = Vec::new();
    for fn_def in &ir.fns {
        if contains_aot_unsupported(&fn_def.body) {
            errors.push(format!(
                "E0427: unsupported feature in AOT mode in function `{}`",
                fn_def.name
            ));
        }
    }
    errors
}
```

挿入位置: `CraneliftBackend` の `impl` ブロック **外側**（impl の閉じ括弧の直後）に standalone fn として配置。
`analyze_for_inlining` は `impl CraneliftBackend` の内部にあるが、`validate_aot_compat` は
`crate::backend::cranelift_aot::validate_aot_compat(&ir)` として呼べるよう impl 外に配置する。

### 2. `error_catalog.rs` — E0427 エントリ追加

`E0426` エントリの直後（`E05xx` セクションコメントの直前）に追加：

```rust
// ── E0427: AOT 未サポート機能 (v62.8.0) ─────────────────────────────────────
ErrorEntry {
    code: "E0427",
    title: "unsupported feature in AOT mode",
    category: "build",
    description: "The pipeline contains a feature that is not supported in AOT compilation mode. \
                  Detected features: effect emission (emit expression).",
    example: "fn f() -> Unit { emit \"order_placed\" }  // E0427: emit is not supported in AOT mode",
    fix: "Use `fav run` instead of `fav build`, or remove the unsupported feature from the pipeline.",
    long_description: Some(
        "AOT (Ahead-of-Time) compilation generates a native binary from the pipeline IR.\n\
         Some Favnir features rely on VM-level dynamic dispatch and cannot be lowered to \
         native code in the current AOT backend.\n\
         \n\
         Unsupported features in AOT mode (v62.8.0):\n\
         - `emit expr` — effect emission requires VM runtime dispatch.\n\
         \n\
         To use effect-emitting stages, run the pipeline with `fav run` which uses \
         the full VM runtime. If AOT output is required, refactor the pipeline to \
         remove emit expressions and use pure data transformations only."
    ),
    suggestion: Some("Run `fav explain E0427` for details on which features are AOT-incompatible."),
},
```

### 3. `driver.rs` — `cmd_build_aot_validate` 追加

`cmd_build_aot_stats` の直後に追加：

```rust
/// v62.8.0: AOT 互換性チェック — E0427 を返す機能を持つ関数を報告する。
pub fn cmd_build_aot_validate(src: &str) -> String {
    let program = match crate::frontend::parser::Parser::parse_str(src, "<validate>") {
        Ok(p) => p,
        Err(e) => return format!("parse error: {e}"),
    };
    let ir = compile_program(&program);
    let errors = crate::backend::cranelift_aot::validate_aot_compat(&ir);
    if errors.is_empty() {
        "AOT compatibility check passed.".to_string()
    } else {
        errors.join("\n")
    }
}
```

### 4. `driver.rs` — `v62800_tests` 追加

`v62700_tests` の直前（ファイル先頭方向）に挿入。

**`aot_e0427_eval_detected`**:
- ソース: `"fn f() -> Unit { emit \"hello\" }\nfn main() -> Bool { true }"`
  （`emit` 式が `IRExpr::Emit` にコンパイルされることを利用）
- `cmd_build_aot_validate(src)` を呼ぶ
- 結果が `"E0427"` を含むことを確認
- 結果が `"f"` を含むことを確認（関数名が出力される）

**`error_catalog_has_e0427`**:
- `crate::error_catalog::ERROR_CATALOG` を走査
- `"E0427"` コードを持つエントリが存在することを確認
- `entry.category == "build"` を確認

---

## 完了条件

- `cargo build` エラーなし
- `cargo test v62800` で 2 件 PASS
- `cargo test -j 8 -- --test-threads=8` で 3399 tests passed, 0 failed

---

## 非スコープ

- CLI フラグ `fav build --validate` の追加（v62.9.0 以降）
- E0427 を `fav build` の自動チェックに統合（現時点は `cmd_build_aot_validate` API のみ）
- `IRExpr::Par`（並列ステージ）/ `IRExpr::Collect` 等の追加 AOT 非対応機能の検出（v62.9.0 以降）
- `site/content/docs/` の MDX ドキュメント — v62.9.0 でまとめて作成
- 行番号・列番号付きのエラーロケーション出力（現時点は関数名のみ）

---

## 技術ノート

### `emit` 構文と `IRExpr::Emit` の対応

Favnir の `emit "expr"` 構文（`TokenKind::Emit`）は
`middle/compiler.rs` の `compile_expr` で `IRExpr::Emit(Box<IRExpr>, Type::Unit)` にコンパイルされる。
`!Emit<E>` 型注釈は v34.8A で E0374 として除去済みだが、`emit expr` 式自体は引き続きパース・コンパイル可能。

### ベーステスト数の変更について

ロードマップ記載のベースは 3396 だが、v62.7.0 の code-reviewer 対応により
`build_resolve_defaults_when_no_toml` テストが追加されたため実際のベースは **3397**。
完了条件のターゲットは 3397 + 2 = **3399**。
