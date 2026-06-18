# v18.3.0 仕様書 — Refinement Types（引数制約）

## 概要

関数引数に `where { ... }` 制約を付けることで、値レベルの不変条件を型に埋め込む。
コンパイル時に静的に評価できる場合は E0331 をコンパイルエラーとして発行し、
実行時にのみ判定できる場合は `RefinementAssert` opcode でランタイムアサーションを挿入する。

---

## 構文

```
param ::= ident ":" type_expr ["where" "{" expr "}"]
```

### 例

```fav
// ゼロ除算禁止
fn divide(a: Int, b: Int where { b != 0 }) -> Int {
  a / b
}

// 正の数のみ
fn sqrt(x: Float where { x >= 0.0 }) -> Float {
  Float.sqrt(x)
}

// 範囲制約
fn set_age(age: Int where { age >= 0 && age <= 150 }) -> Int {
  age
}

// 複数引数にそれぞれ制約
fn slice(arr: List<Int>, start: Int where { start >= 0 }, end: Int where { end > start }) -> List<Int> {
  List.slice(arr, start, end)
}
```

制約式 `{ expr }` の中では **当該引数名** が束縛されている（他の引数も参照可能）。

---

## コンパイル時チェック（E0331）

呼び出し時の引数が**整数/浮動小数点/真偽値リテラル**の場合、コンパイラは制約を静的に評価する。

```fav
fn main() -> Int {
  divide(10, 0)
  // E0331: refinement violated: argument `b` must satisfy `b != 0`, got 0
}
```

### 静的評価の対象

`eval_static_expr` が評価できる式（`StaticValue::Int / Float / Bool / String`）であれば
コンパイル時に制約評価を試みる。評価できない場合はランタイムチェックにフォールバック。

---

## ランタイムチェック（RefinementAssert opcode）

変数・式を渡す場合、コンパイラは制約のランタイムアサーションを注入する。

```fav
fn main() -> Int {
  bind divisor <- IO.read_int()
  divide(10, divisor)
  // ランタイム: divisor != 0 を検査。違反なら RefinementError をスロー
}
```

違反時は `Err("refinement violated: b != 0")` を返す（または panic）。

---

## AST 変更

### `Param` 構造体

```rust
pub struct Param {
    pub name: String,
    pub ty: TypeExpr,
    pub constraint: Option<Box<Expr>>,  // 追加
    pub span: Span,
}
```

---

## エラーコード

| コード | 説明 |
|---|---|
| E0331 | Refinement violated at call site（コンパイル時静的チェック失敗） |

---

## VM opcode

| opcode | 説明 |
|---|---|
| `RefinementAssert` | スタックトップの値に対してクロージャ制約を評価。違反なら error value をプッシュ |

---

## テスト一覧（v183000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_18_3_0` | Cargo.toml に "18.3.0" が含まれる |
| `refinement_literal_pass` | リテラル引数が制約を満たす場合は正常コンパイル |
| `refinement_literal_fail` | リテラル引数が制約違反 → E0331 |
| `refinement_runtime_check` | 変数引数に RefinementAssert が注入される（正常実行） |
| `refinement_range_constraint` | `age >= 0 && age <= 150` 範囲制約の複合 where 式 |

---

## 完了条件

- [ ] `Param.constraint: Option<Box<Expr>>` が `ast.rs` に存在する
- [ ] `fn f(x: Int where { x > 0 })` がパースされる
- [ ] リテラル違反で E0331 が発行される
- [ ] 変数引数で `RefinementAssert` opcode が注入される
- [ ] `cargo test v183000` — 5/5 PASS
- [ ] `cargo test` — リグレッションなし
- [ ] `site/content/docs/language/refinement-types.mdx` が存在する
