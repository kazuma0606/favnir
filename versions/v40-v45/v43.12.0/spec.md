# v43.12.0 Spec — W031〜W033 lint（冗長型注釈の警告）

## 概要

型推論が充実した v43.1〜v43.11 の成果を受けて、**推論可能であるにもかかわらず明示されている型注釈**を警告する lint ルールを追加する。

| コード | 名称 | 対象 |
|---|---|---|
| W031 | `redundant_return_type` | 推論可能な戻り値型の明示的注釈 |
| W032 | `redundant_generic_arg` | 推論可能なジェネリック型引数の明示 |
| W033 | `redundant_lambda_param_type` | 推論可能なラムダ引数型の明示（lint.rs スタブコメントのみ） |

---

## W031 — 推論可能な戻り値型の明示的注釈

### 条件（すべてを満たすとき警告）

1. `FnDef.return_ty.is_some()` — 戻り値型が明示されている
2. `FnDef.body.stmts.is_empty()` — ボディが単一式（`stmts` が空）
3. ボディ末尾式が**リテラル**（`Expr::Lit`）または**識別子**（`Expr::Ident`）

### 例

```favnir
fn answer() -> Int { 42 }     // W031: return type `Int` is inferrable
fn greeting() -> String { "hello" }  // W031
```

### 非警告

```favnir
fn add(a: Int, b: Int) -> Int { a + b }  // 複合式 → 警告なし
```

### メッセージ形式

```
W031: return type annotation is redundant; type can be inferred
```

### AST 確認事項

- `Block.expr` は `Box<Expr>`（非 Option）— `*fd.body.expr` で参照
- `Expr::Var` は存在しない — 変数参照は `Expr::Ident(String, Span)` を使用

---

## W032 — 推論可能なジェネリック型引数の明示

### 条件

関数呼び出し式が `Expr::TypeApply(inner, type_args, span)` である場合に警告。

ジェネリック型引数は呼び出し側引数から推論可能なため、明示は冗長。

### 例

```favnir
bind v <- identity::<Int>(42)   // W032: generic type arg `Int` is inferrable
```

### メッセージ形式

```
W032: explicit generic type argument is redundant; type can be inferred from argument
```

### 走査対象

FnDef の `body.stmts`（各 Stmt 内の式）および `body.expr`（末尾式）の両方を再帰的に走査する。

---

## W033 — 推論可能なラムダ引数型の明示（lint.rs スタブのみ）

### 背景

AST の `Expr::Closure(Vec<String>, Box<Expr>, Span)` はパラメータ名のみを保持し、型注釈を持たない。したがって AST レベルでの型注釈有無の検出が不可能。

将来実装のためには、parser および AST の拡張（型注釈付きクロージャ構文 `|x: Int|`）が必要になる。

### v43.12.0 スコープ

- `lint.rs` に `// W033: 将来版（AST 拡張後に実装）` スタブコメントを追加するのみ
- error_catalog.rs への追加なし

---

## 実装方針

- 既存の W コードは error_catalog.rs ではなく **lint.rs にインラインで定義**されている（W001〜W030 の既存パターン）。W031〜W032 も同様に lint.rs にインライン文字列リテラルとして定義する
- `lint.rs` の `lint_program()` に `check_w031` / `check_w032` を追加呼び出し
- W033 は `lint_program()` 内にスタブコメントのみ
- `driver.rs` に `v431200_tests` モジュール（3 件）追加
- `Cargo.toml` version: `43.11.0` → `43.12.0`

---

## 完了条件

- `cargo test -j 8 -- --test-threads=8` で **2935 passed; 0 failed**
- `v431200_tests` 3 件 pass
  1. `cargo_toml_version_is_43_12_0`
  2. `w031_warns_on_redundant_return_annotation`
  3. `w032_warns_on_explicit_generic_type_arg`
