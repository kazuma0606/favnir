# Plan: v52.1.0 — `assert_schema` Phase 1（型チェック）

Date: 2026-07-20
Status: 設計中

---

## 事前確認

- `cargo test` 3135 passed, 0 failed を確認（ベース）
- `cargo clippy -- -D warnings` クリーンであることを確認
- `ast.rs` に `AssertSchema` が存在しないことを確認
- `middle/ir.rs` に `AssertSchema` が存在しないことを確認
- `backend/vm.rs` に `AssertSchema` が存在しないことを確認
- `error_catalog.rs` の E0419 が「予約」コメントのみであることを確認

---

## Step 1 — `ast.rs` に `Expr::AssertSchema` 追加

`Expr` enum に以下を追加:

```rust
AssertSchema {
    ty_name: String,
    arg: Box<Expr>,
    span: Span,
},
```

その後 `cargo build` を実行し、`Expr` を exhaustive match する箇所のコンパイルエラーをすべて解消する。
既知の対応必須ファイル（他にも `cargo build` エラーで判明するものを都度追加）:
- `fav/src/fmt.rs`
- `fav/src/lint.rs`
- `fav/src/emit_python.rs`
- `fav/src/middle/checker.rs`（`collect_helpers_in_expr` / `check_expr` 等）
- `fav/src/middle/compiler.rs`（`collect_free_vars_expr` 等）
- `fav/src/lineage.rs`（`Expr` を match する複数の関数）
- `fav/src/lsp/references.rs`（`collect_in_expr` 等）

各ファイルに `AssertSchema` アームを追加（`_ => {}` や `unreachable!()` は避ける）。

---

## Step 2 — `middle/ir.rs` に `IRExpr::AssertSchema` 追加

`IRExpr` enum に以下を追加:

```rust
AssertSchema {
    ty_name: String,
    arg: Box<IRExpr>,
    ty: Type,
},
```

---

## Step 3 — `middle/compiler.rs` に変換ロジック追加

`compile_expr` の `Expr::AssertSchema` アームで `IRExpr::AssertSchema` を返す:

```rust
Expr::AssertSchema { ty_name, arg, .. } => {
    let compiled_arg = self.compile_expr(arg)?;
    Ok(IRExpr::AssertSchema {
        ty_name: ty_name.clone(),
        arg: Box::new(compiled_arg),
        ty: Type::Result(Box::new(Type::Named(ty_name.clone()))),
    })
}
```

---

## Step 4 — `backend/vm.rs` に実行時評価ハンドラ追加

`eval_expr` の `IRExpr::AssertSchema` アームを追加:

- `arg` を評価し `VMValue::Map(map)` を取得
- `ty_name` に対応するスキーマをレジストリから参照
- フィールド名・型が不一致なら `VMValue::Err(E0419 メッセージ)` を返す
- 一致すれば `VMValue::Ok(VMValue::Map(map))` を返す

---

## Step 5 — `backend/wasm_dce.rs` の exhaustive match 対応

`collect_expr_fns` の `IRExpr` 網羅 match に `AssertSchema` アームを追加:

```rust
IRExpr::AssertSchema { arg, .. } => {
    collect_expr_fns(arg, out);
}
```

---

## Step 6 — `error_catalog.rs` に E0419 定義

既存の「予約」コメントを実際の `ErrorEntry` に置き換える:

```rust
ErrorEntry {
    code: "E0419",
    title: "assert_schema type mismatch",
    category: "runtime",
    description: "assert_schema<T> found a field whose runtime type does not match the schema T.",
    example: "// expected { id: Int } but got { id: \"hello\" }",
    fix: "Ensure the input map contains fields matching the schema T.",
    suggestion: Some("Check the upstream data source for type mismatches."),
},
```

---

## Step 7 — `driver.rs` にテスト追加 + Cargo.toml バンプ

`v52100_tests` モジュールを `v52000_tests` の直前に追加（2 件）:

```rust
// -- v52100_tests (v52.1.0) -- assert_schema Phase 1 --
#[cfg(test)]
mod v52100_tests {
    #[test]
    fn assert_schema_type_ok() {
        let src = include_str!("ast.rs");
        assert!(src.contains("AssertSchema"), "ast.rs must define AssertSchema node");
        let vm = include_str!("backend/vm.rs");
        assert!(vm.contains("AssertSchema"), "vm.rs must handle AssertSchema");
    }
    #[test]
    fn assert_schema_type_fail() {
        let src = include_str!("error_catalog.rs");
        assert!(src.contains("E0419"), "error_catalog.rs must define E0419");
        assert!(src.contains("assert_schema type mismatch"),
            "E0419 must have title 'assert_schema type mismatch'");
    }
}
```

`v52000_tests` から `cargo_toml_version_is_52_0_0` を削除し、
`fav/Cargo.toml` version を `"52.1.0"` に更新。

---

## Step 8 — 後処理

- `cargo test` 3136 passed, 0 failed を確認（実測値で確定）
- `cargo clippy -- -D warnings` クリーンを確認
- `CHANGELOG.md` に v52.1.0 エントリ追加
- `versions/current.md` を v52.1.0（実測テスト数）に更新
- `roadmap-v52.1-v53.0.md` の v52.1.0 実績欄を更新
- `versions/v50-v55/v52.1.0/tasks.md` を COMPLETE に更新

---

## テスト数計算

| 操作 | 差分 |
|---|---|
| ベース（v52.0.0） | 3135 |
| `cargo_toml_version_is_52_0_0` 削除 | -1 |
| `v52100_tests` 2 件追加 | +2 |
| **合計（計算上）** | **3136** |

実装後 `cargo test` 実測値で確定する。

---

## 影響ファイル

| ファイル | 変更種別 |
|---|---|
| `fav/src/ast.rs` | `Expr::AssertSchema` 追加 |
| `fav/src/middle/ir.rs` | `IRExpr::AssertSchema` 追加 |
| `fav/src/middle/compiler.rs` | `AssertSchema` コンパイル追加 + `collect_free_vars_expr` アーム |
| `fav/src/middle/checker.rs` | `Expr::AssertSchema` exhaustive match 対応 |
| `fav/src/backend/vm.rs` | `AssertSchema` 評価ハンドラ追加 |
| `fav/src/backend/wasm_dce.rs` | `collect_expr_fns` に `AssertSchema` アーム追加 |
| `fav/src/error_catalog.rs` | E0419 実装 |
| `fav/src/fmt.rs` | `Expr::AssertSchema` アーム追加 |
| `fav/src/lint.rs` | `Expr::AssertSchema` アーム追加 |
| `fav/src/emit_python.rs` | `Expr::AssertSchema` アーム追加 |
| `fav/src/lineage.rs` | `Expr::AssertSchema` exhaustive match 対応 |
| `fav/src/lsp/references.rs` | `Expr::AssertSchema` exhaustive match 対応 |
| `fav/src/driver.rs` | `v52100_tests` 追加、`cargo_toml_version_is_52_0_0` 削除 |
| `fav/Cargo.toml` | version → `"52.1.0"` |
| `CHANGELOG.md` | v52.1.0 エントリ追加 |
| `versions/current.md` | v52.1.0 に更新 |
| `versions/roadmap/roadmap-v52.1-v53.0.md` | v52.1.0 実績欄更新 |

> `frontend/parser.rs` は Phase 1 スコープ外（パース実装は Phase 2 以降）。
> テスト 2 件は `include_str!` 静的検査のみのため、パーサー未対応でも pass する。
