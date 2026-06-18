# v18.7.0 — 型レベル定数（Const Generics）タスク

## ステータス: COMPLETE ✅ (2026-06-16) — 1684 tests pass, 5/5 v187000_tests PASS

---

## タスク一覧

### T1: `fav/src/ast.rs` — 型追加・構造体拡張

- [x] `GenericParam` に以下のフィールドを末尾に追加:
  ```rust
  pub is_const: bool,                      // v18.7.0: `const N: Int`
  pub const_ty: Option<TypeExpr>,          // v18.7.0: const の型（Int のみ）
  pub const_constraint: Option<Box<Expr>>, // v18.7.0: `where { N > 0 }`
  ```
- [x] `GenericParam::unbounded()` に `is_const: false, const_ty: None, const_constraint: None` を追加
- [x] `TypeExpr::ConstInt(i64, Span)` バリアントを追加（型位置の整数リテラル）
- [x] `TypeExpr::span()` に `ConstInt(_, s) => s` を追加
- [x] `cargo build` でコンパイルエラーが生じることを確認（T2 で修正）

### T2: 波及ファイル修正（exhaustive match / 構造体リテラル）

**`GenericParam { ... }` struct リテラル修正**（`is_const: false, const_ty: None, const_constraint: None` 追加）:

- [x] `fav/src/frontend/parser.rs` — `parse_type_params` 内の `GenericParam { name, bounds, variance }` （2箇所）
- [x] `fav/src/frontend/parser.rs` — `parse_variance_type_params` 内の `GenericParam { name, bounds, variance }` （1箇所）
- [x] その他 `GenericParam { ... }` struct リテラルが存在するファイル（Grep で確認）

**`TypeExpr::ConstInt` exhaustive match 追加**（各ファイル）:

- [x] `fav/src/ast.rs` — `TypeExpr::span()` に `ConstInt(_, s) => s` ← T1 で追加済み
- [x] `fav/src/fmt.rs` — `type_expr()`: `ConstInt(n, _) => format!("{}", n)`
- [x] `fav/src/fmt.rs` — `fmt_type_expr_simple()`: `ConstInt(n, _) => format!("{}", n)`
- [x] `fav/src/emit_python.rs` — `ConstInt(_, _) => "int".to_string()`（または既存の `=> "Any"` arm に merge）
- [x] `fav/src/middle/ast_lower_checker.rs` — `lower_te()`: `ConstInt(n, _) => v1("TeInt", ...)` など適切な形式
- [x] `fav/src/middle/ast_lower_checker.rs` — `te_to_string()`: `ConstInt(n, _) => format!("{}", n)`
- [x] `fav/src/middle/compiler.rs` — `lower_type_expr_with_subst()`: `ConstInt(_, _) => Type::Int`
- [x] `fav/src/middle/compiler.rs` — `substitute_self_in_type_expr()`: `ConstInt(n, s) => TypeExpr::ConstInt(*n, s.clone())`（そのまま返す）
- [x] `fav/src/middle/compiler.rs` — 3番目の `lower_type_expr()`: `ConstInt(_, _) => Type::Int`
- [x] `fav/src/middle/checker.rs` — `resolve_type_expr_with_subst()`: `ConstInt(_, _) => Type::Int`
- [x] `fav/src/middle/checker.rs` — `resolve_type_expr_with_self()`: `ConstInt(_, _) => Type::Int`
- [x] `fav/src/middle/checker.rs` — `validate_type_expr_arity()`: `ConstInt(_, _) => {}`
- [x] `fav/src/middle/checker.rs` — `type_expr_contains()`: `ConstInt(_, _) => false`
- [x] `fav/src/driver.rs` — `format_type_expr()`: `ConstInt(n, _) => format!("{}", n)`
- [x] `fav/src/driver.rs` — `favnir_type_display()`: `ConstInt(n, _) => format!("{}", n)`
- [x] `fav/src/driver.rs` — `graphql_type_from_type_expr_nonnull()`: `ConstInt(_, _) => "Int".to_string()`
- [x] `fav/src/driver.rs` — `proto_type_from_type_expr_nonwrapper()`: `ConstInt(_, _) => "int32".to_string()`
- [x] `fav/src/driver.rs` — `favnir_type_to_sql_from_expr()`: `ConstInt(_, _) => "INTEGER".to_string()`
- [x] `cargo build` でコンパイルエラーが 0 になることを確認

### T3: `fav/src/frontend/parser.rs` — パース実装

**3-A: `parse_type_params` への `const N: Int` 対応**

- [x] `parse_type_params` のループ冒頭に `const` ソフトキーワード検出を追加:
  - `peek_ident_text("const")` または `peek() == &TokenKind::Ident && token_text() == "const"` で検出
  - `advance()` → `expect_ident()` → `expect(':')` → `parse_base_type()` で型取得
  - `where` ソフトキーワードが続く場合: `advance()` → `expect('{')` → `parse_expr()` → `expect('}')`
  - `GenericParam { name, is_const: true, const_ty: Some(ty), const_constraint, bounds: vec![], variance: Variance::Invariant, ... }` を push
- [x] `const` がない場合は既存の variance + bounds パースへフォールスルー
- [x] `const` は `Ident("const")` として lexer からくる（新 TokenKind 不要）

**3-B: `parse_type_arg_list` + `parse_type_arg` 追加**

- [x] `parse_type_arg(&mut self) -> Result<TypeExpr, ParseError>` メソッドを追加:
  ```rust
  if let TokenKind::Int(n) = self.peek().clone() {
      let sp = self.peek_span().clone();
      self.advance();
      Ok(TypeExpr::ConstInt(n, sp))
  } else {
      self.parse_type_expr()
  }
  ```
- [x] `parse_type_arg_list` を `parse_type_expr()` → `parse_type_arg()` に変更（2箇所）

**注意**: `parse_variance_type_params`（interface 用）は const 対応不要（interface では const param は使わない）。

### T4: `fav/src/middle/checker.rs` — E0335 実装

**4-A: `eval_const_expr` / `eval_const_int` フリー関数の追加**

checker.rs の末尾（`type_expr_contains` の近く）に追加:

```rust
/// Statically evaluate a const constraint: `N > 0`, `N != 0`, `N >= 1` etc.
/// `var_name` は const パラメータ名（"N" 等）、`var_val` は実際に渡された整数値。
/// Returns Some(true/false) if evaluable, None if static evaluation is impossible.
fn eval_const_expr(expr: &Expr, var_name: &str, var_val: i64) -> Option<bool> {
    match expr {
        Expr::BinOp(op, lhs, rhs, _) => {
            match op {
                BinOp::And => {
                    let l = eval_const_expr(lhs, var_name, var_val)?;
                    let r = eval_const_expr(rhs, var_name, var_val)?;
                    Some(l && r)
                }
                BinOp::Or => {
                    let l = eval_const_expr(lhs, var_name, var_val)?;
                    let r = eval_const_expr(rhs, var_name, var_val)?;
                    Some(l || r)
                }
                _ => {
                    let l = eval_const_int(lhs, var_name, var_val)?;
                    let r = eval_const_int(rhs, var_name, var_val)?;
                    Some(match op {
                        BinOp::Gt    => l > r,
                        BinOp::GtEq  => l >= r,
                        BinOp::Lt    => l < r,
                        BinOp::LtEq  => l <= r,
                        BinOp::Eq    => l == r,
                        BinOp::NotEq => l != r,
                        _ => return None,
                    })
                }
            }
        }
        _ => None,
    }
}

fn eval_const_int(expr: &Expr, var_name: &str, var_val: i64) -> Option<i64> {
    match expr {
        Expr::Lit(Lit::Int(n), _) => Some(*n),
        Expr::Ident(name, _) if name == var_name => Some(var_val),
        _ => None,
    }
}
```

**4-B: `Expr::TypeApply` チェック時に E0335 を追加**

`check_expr` の `Expr::TypeApply(func, type_args, span)` ハンドラを確認・拡張:

- [x] `Expr::Ident(fn_name, _)` か `Expr::FieldAccess(_, method_name, _)` で関数名を取得
- [x] `fn_bounds_registry.get(fn_name)` で const パラメータを含む `GenericParam` リストを取得
- [x] zip でパラメータと型引数を対応させ、`param.is_const == true` の場合:
  - `TypeExpr::ConstInt(n, _)` なら `eval_const_expr` で制約評価
  - 評価結果が `Some(false)` → E0335 emit
- [x] const 制約の文字列表現（エラーメッセージ用）: `format_const_constraint` ヘルパー or 仮文字列で OK

**注意**: `Expr::TypeApply` チェックの既存実装箇所を Grep で特定してから挿入位置を決める。

### T5: `fav/src/driver.rs` — `v187000_tests` 追加

- [x] `v186000_tests::version_is_18_6_0` に `#[ignore]` を追加
- [x] `v187000_tests` モジュールを追加（5件）:

  ```rust
  #[test]
  fn version_is_18_7_0() {
      // Cargo.toml に "18.7.0" が含まれる
  }

  #[test]
  fn const_generic_parses() {
      // `fn process<const N: Int>(items: List<Int>) -> Int` をパースして
      // GenericParam { is_const: true, name: "N", const_ty: Some(Named("Int",...)) }
      // が含まれることを確認
  }

  #[test]
  fn const_generic_constraint_parses() {
      // `fn safe_chunk<const N: Int where { N > 0 }>(items: List<Int>) -> List<List<Int>>`
      // をパースして const_constraint: Some(...) が存在することを確認
  }

  #[test]
  fn const_generic_violation() {
      // safe_chunk::<0>(xs) で E0335 が出ることを確認
      // fn safe_chunk<const N: Int where { N > 0 }>(items: List<Int>) -> Int { N }
      // fn main() -> Int { safe_chunk::<0>(List.empty()) }
      // → errors に E0335 が含まれる
  }

  #[test]
  fn const_generic_valid() {
      // safe_chunk::<100>(xs) でエラーが出ないことを確認
      // fn safe_chunk<const N: Int where { N > 0 }>(items: List<Int>) -> Int { N }
      // fn main() -> Int { safe_chunk::<100>(List.empty()) }
      // → errors に E0335 が含まれない
  }
  ```

### T6: バージョン更新

- [x] `fav/Cargo.toml` のバージョンを `18.6.0` → `18.7.0` に更新
- [x] `fav/src/driver.rs` の `version_is_18_6_0` に `#[ignore]` を追加

### T7: `site/content/docs/language/const-generics.mdx` 作成

- [x] `const N: Int` 構文説明
- [x] `where { N > 0 }` 制約の説明
- [x] `f::<100>(...)` 呼び出し構文
- [x] E0335 エラーの説明と例
- [x] データパイプラインでのバッチサイズ制御の実践例
- [x] スコープ外事項（Float/String/推論）の説明

---

## テスト（v187000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_18_7_0` | Cargo.toml に "18.7.0" が含まれる |
| `const_generic_parses` | `const N: Int` が `GenericParam { is_const: true }` としてパースされる |
| `const_generic_constraint_parses` | `where { N > 0 }` が `const_constraint: Some(...)` に保存される |
| `const_generic_violation` | `f::<0>()` で E0335 が出る |
| `const_generic_valid` | `f::<100>()` でエラーが出ない |

---

## 完了条件チェックリスト

- [x] `GenericParam.is_const` / `const_ty` / `const_constraint` フィールドが存在する
- [x] `TypeExpr::ConstInt` が `ast.rs` に存在する
- [x] `fn f<const N: Int>(...)` が `is_const: true` としてパースされる
- [x] `where { N > 0 }` 制約が `const_constraint` に保存される
- [x] `f::<100>(...)` の `100` が `TypeExpr::ConstInt(100, _)` としてパースされる
- [x] `f::<0>()` で `N > 0` 制約が E0335 になる
- [x] 制約を満たす場合はエラーなし
- [x] `cargo test v187000` — 5/5 PASS
- [x] `cargo test` — リグレッションなし
- [x] `site/content/docs/language/const-generics.mdx` が存在する

---

## 優先度

```
T1（ast.rs 型追加）                   ← 最初
T2（波及ファイル修正）                 ← T1 完了後すぐ（cargo build が通るまで）
T3（parser.rs const パース）          ← T2 完了後
T4（checker.rs E0335）                ← T3 完了後
T5（v187000_tests）                   ← T4 完了後
T6（バージョン更新）                   ← T5 完了後
T7（ドキュメント）                     ← T6 と並列可
```

---

## 重要な技術ノート

### `GenericParam` struct 拡張の波及

`GenericParam` は struct なので、既存の `GenericParam { name, bounds, variance }` リテラルに
`is_const: false, const_ty: None, const_constraint: None` を追加する必要がある。
`GenericParam::unbounded()` コンストラクタ経由の箇所は自動的に正しいので修正不要。

### `TypeExpr::ConstInt` の波及が最大の作業

`TypeExpr` に新バリアントを追加すると 7〜10 ファイルで exhaustive match エラーが発生する。
T2 で `cargo build` のエラーリストを全件確認してから一括修正すること。

### `const` のレキシング

`const` は `Ident("const")` としてレキシングされる（別の TokenKind にはならない）。
`peek_ident_text("const")` 相当のメソッドを使って判定する。
既存の `peek_ident_value()` / `peek_ident_text()` の実装を確認してから使用すること。

### `eval_const_expr` の `BinOp` 型

`Expr::BinOp(op, lhs, rhs, span)` の `op` は `BinOp` enum（`Add/Sub/Mul/Div/Eq/NotEq/Lt/Gt/LtEq/GtEq/And/Or/NullCoalesce`）。
文字列ではないので `match op { BinOp::Gt => ..., ... }` で分岐する。

### `Expr::TypeApply` の既存チェック箇所

`check_expr` 内の `Expr::TypeApply` ハンドラはすでに実装済み。
Grep で `TypeApply` を検索して既存ロジックを確認してから E0335 ロジックを挿入する。
