# v18.7.0 Spec — 型レベル定数（Const Generics）

Date: 2026-06-16

## 概要

バッチサイズ・配列長などの整数定数を型パラメータとして扱えるようにする。
「バッチサイズが 0 のまま本番デプロイ」のような設定ミスをコンパイル時に検出する。

v18.7.0 では **`Int` 型定数のみ** 対応（Float / String は v19.x 以降）。

---

## 設計

### 構文

```favnir
// const 型パラメータの宣言
fn process_batch<const N: Int>(items: List<Row>) -> List<Output> {
  List.map(List.chunk(items, N), process_chunk)
}

// const 制約（コンパイル時チェック）
fn safe_chunk<const N: Int where { N > 0 }>(items: List<Row>) -> List<List<Row>> {
  List.chunk(items, N)
}

// type 定義での const パラメータ
type FixedBatch<T, const N: Int> = { items: List<T>, size: Int }

// 呼び出し時の const 引数（整数リテラルを型位置に渡す）
bind result <- process_batch::<100>(my_rows)
bind bad    <- safe_chunk::<0>(my_rows)   // E0335: N > 0 が成立しない
```

### E0335 — Const 制約違反

呼び出し時に const 引数が `where` 制約を満たさない場合、コンパイル時にエラーを出す。

```
E0335: const constraint violation: `N > 0` is not satisfied (N = 0)
  hint: `safe_chunk` requires `N > 0`; provide a positive integer
```

---

## AST 変更

### `GenericParam` への const フィールド追加

```rust
// 現状（v18.6.0）
pub struct GenericParam {
    pub name: String,
    pub bounds: Vec<TypeConstraint>,
    pub variance: Variance,
}

// v18.7.0 追加後
pub struct GenericParam {
    pub name: String,
    pub bounds: Vec<TypeConstraint>,
    pub variance: Variance,
    pub is_const: bool,                      // v18.7.0: `const N: Int`
    pub const_ty: Option<TypeExpr>,          // v18.7.0: const の型（Int のみ）
    pub const_constraint: Option<Box<Expr>>, // v18.7.0: `where { N > 0 }` の式
}
```

`unbounded()` コンストラクタへのデフォルト追加:
```rust
pub fn unbounded(name: impl Into<String>) -> Self {
    Self { name: name.into(), bounds: vec![], variance: Variance::Invariant,
           is_const: false, const_ty: None, const_constraint: None }
}
```

### `TypeExpr::ConstInt` 追加

型位置に整数リテラルを渡すための新バリアント:

```rust
// ast.rs
pub enum TypeExpr {
    // ...既存...
    /// `100` — integer constant in type argument position (v18.7.0)
    ConstInt(i64, Span),
}
```

`f::<100>(...)` の `100` を `TypeExpr::ConstInt(100, span)` としてパースする。

---

## Parser 変更

### `parse_type_params` の拡張 — `const N: Int` 対応

```
'<' → loop {
    if peek() == Ident("const"):
        advance()           // consume `const`
        name = expect_ident
        expect(':')
        ty = parse_base_type  // Int のみ
        constraint = if peek() == With: parse_where_constraint
        push GenericParam { name, is_const: true, const_ty: Some(ty), const_constraint: ... }
    else:
        variance = parse_variance_prefix
        name = expect_ident
        bounds = parse_type_bounds
        push GenericParam { name, bounds, variance, is_const: false, ... }
    ...
}
```

`const` はソフトキーワード（`Ident("const")`）として検出する（新トークン不要）。

### `parse_type_arg_list` の拡張 — 整数リテラル対応

```
'<' → loop {
    if peek() == Int(n):
        advance()
        push TypeExpr::ConstInt(n, span)
    else:
        push parse_type_expr()
    ...
}
```

### const 制約 `where { N > 0 }` のパース

`parse_type_params` 内で、`const N: Int` の後に `where` ソフトキーワードが続く場合:

```
if peek_ident("where"):
    advance()
    expect('{')
    constraint = parse_expr()
    expect('}')
    const_constraint = Some(constraint)
```

---

## Checker 変更

### `check_const_constraint` — E0335

`Expr::TypeApply` チェック時（`f::<100>(...)`）に const 引数を評価:

1. 呼び出す関数の `const` 型パラメータを `fn_bounds_registry` から取得
2. 型引数 `TypeExpr::ConstInt(n, _)` を取り出して `N = n` の代入を作る
3. `const_constraint` がある場合、式中の `N` を `n` に置き換えて静的評価
4. 評価が `false` → **E0335**

### 静的評価（`eval_const_expr`）

`where { N > 0 }` の評価関数:

```
eval_const_expr(expr, subst: HashMap<name, i64>) -> Option<bool>
```

対応する式:
- `N > 0` / `N >= 1` / `N != 0` / `N < 100` 等の比較演算
- `N > 0 && N < 100` 等の論理 AND
- リテラル `0`, `100` 等の整数定数

評価できない式（変数が関与している等）は `None`（チェックスキップ）。

### `fn_bounds_registry` への const パラメータ登録

`register_item_signatures` の `FnDef` 処理で、`is_const: true` の `GenericParam` を持つ fn を `fn_bounds_registry` に登録（既存の bounds 登録と統合）。

---

## Compiler 変更

### const 型パラメータのインライン化

`TypeApply` で const 引数 `ConstInt(n)` が渡された場合:
- IR 生成時に `N` を `n` に置き換えた状態でコンパイル（モノモーフィゼーション不要、定数参照で十分）
- `N` が式の中に現れる箇所（`List.chunk(items, N)` など）は `Int(n)` 定数に置換

`compile_expr` の `Expr::TypeApply` ハンドラに `const` 引数の置換ロジックを追加。

---

## テスト（v187000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_18_7_0` | Cargo.toml に "18.7.0" が含まれる |
| `const_generic_parses` | `fn f<const N: Int>(items: List<Int>) -> Int` が `GenericParam { is_const: true }` としてパースされる |
| `const_generic_constraint_parses` | `fn f<const N: Int where { N > 0 }>()` が `const_constraint: Some(...)` としてパースされる |
| `const_generic_violation` | `safe_chunk::<0>()` で E0335 が出る |
| `const_generic_valid` | `safe_chunk::<100>()` でエラーが出ない |

---

## 完了条件（PASS=5）

1. `fn f<const N: Int>(...)` が `GenericParam { is_const: true }` としてパースされる
2. `fn f<const N: Int where { N > 0 }>(...)` の制約が `const_constraint` に保存される
3. `f::<100>(...)` 形式で const 引数を渡せる（整数リテラルが型位置に来る）
4. `f::<0>(...)` で `N > 0` 制約が E0335 になる
5. `f::<100>(...)` で制約を満たす場合はエラーなし

---

## スコープ外（v19.x 以降）

- `Float` / `String` 型の const パラメータ
- ユーザー定義型の const パラメータ
- 実行時の const 引数（変数を型引数に渡す）
- モノモーフィゼーション（各 N の値で別々のコードを生成）
- const 引数の型推論（`f(my_rows)` から N を推論）
