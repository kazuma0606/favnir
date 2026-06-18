# v17.4.0 — `let` バインディング Spec

Date: 2026-06-15

## 概要

関数ボディ内で非 Result 値に名前を付ける自然な構文 `let x = expr` を追加する。
現状は `bind x <- Result.ok(expr)` という不自然なラップが必要だった。
`let` は Result でない値専用。Result 値に `let` を使うと E0326 エラーになる。

---

## 現状の問題と解決

```fav
// 現状: 非 Result 値を束縛するには Result.ok でラップが必要
fn process(row: Row) -> Result<Output, String> {
  bind trimmed <- Result.ok(String.trim(row.name))   // ← 無駄なラップ
  bind score   <- Result.ok(compute_score(row))       // ← 無駄なラップ
  bind result  <- validate({ ...row, name: trimmed, score: score })
  Result.ok(result)
}

// v17.4 以降: let で自然に束縛
fn process(row: Row) -> Result<Output, String> {
  let trimmed = String.trim(row.name)
  let score   = compute_score(row)
  bind result <- validate({ ...row, name: trimmed, score: score })
  Result.ok(result)
}
```

---

## 構文

```
let <name> = <expr>
```

- `<name>`: 識別子（変数名）
- `<expr>`: 任意の式。ただし型が `Result<_, _>` でないこと
- 関数ボディ内の `Stmt` として使用。トップレベルでは使用不可
- `bind` との混在可能（`let` の後に `bind` を使える）

### 使用例

```fav
// 基本: 数値・文字列
fn greet(name: String) -> Result<String, String> {
  let trimmed = String.trim(name)
  let msg     = f"Hello, {trimmed}!"
  Result.ok(msg)
}

// stdlib 呼び出し
fn transform(row: Row) -> Result<Row, String> {
  let upper = String.to_upper(row.name)
  let score = compute_score(row.value)
  Result.ok({ ...row, name: upper, score: score })
}

// レコードスプレッドと組み合わせ
fn update_row(row: Row, new_name: String) -> Result<Row, String> {
  let trimmed = String.trim(new_name)
  let updated = { ...row, name: trimmed }
  bind validated <- validate(updated)
  Result.ok(validated)
}

// 内包表記と組み合わせ（v17.3）
fn double_positives(ns: List<Int>) -> Result<List<Int>, String> {
  let doubled = [x * 2 | x <- ns, x > 0]
  Result.ok(doubled)
}
```

### `let` 使用ルール

```fav
// OK: 非 Result 値の束縛
fn f() -> Result<Int, String> {
  let x    = 42                       // Int — OK
  let name = String.trim("  hi  ")    // String — OK
  let list = List.singleton(1)        // List<Int> — OK
  bind r <- some_result_fn()          // Result → bind を使う
  Result.ok(x + r)
}

// エラー: Result 値に let を使った場合
fn g() -> Result<Int, String> {
  let r = some_result_fn()   // E0326: Result value must use `bind`, not `let`
  Result.ok(r)
}
```

---

## AST

### 追加 Node

```rust
// fav/src/ast.rs の Stmt enum に追加
Stmt::Let {
    name: String,
    expr: Box<Expr>,
    span: Span,
}
```

既存の `Stmt::Bind { name, expr, span }` とは独立した variant。
`Bind` は Result を unwrap するが、`Let` は単純な値束縛。

---

## 型チェック

`checker.rs` の `check_stmt` / `infer_stmt` に `Stmt::Let` を追加：

1. `expr` の型を推論する
2. 推論された型が `Type::Result(_, _)` であれば **E0326** を発出
3. `name` を推論型でスコープに追加（`bind` と同様）

E0326 のメッセージ例：
```
E0326: `let` cannot be used with Result values — use `bind r <- expr` instead
```

---

## コンパイル

`compiler.rs` の `compile_stmt` に `Stmt::Let` を追加：

```rust
Stmt::Let { name, expr, .. } => {
    compile_expr(expr, ctx);          // 値をスタックに積む
    let slot = ctx.define_local(name); // local 変数スロットを確保
    ctx.emit(Opcode::StoreLocal(slot));
}
```

`Stmt::Bind` の `SeqStageCheck` / `LegacyBindCheck` opcode は不要。
単純な `StoreLocal` のみ。

---

## Lexer

`let` キーワードを追加：

```rust
// lexer.rs
TokenKind::Let  // "let"
```

既存の `bind` に対して `let` は等値比較用（`=`）を使う区別がある。

---

## エラーコード

| コード | 意味 |
|---|---|
| E0326 | `let` に Result 型の式を割り当てた（`bind` を使うべき） |

---

## テスト（v174000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_17_4_0` | バージョン文字列が "17.4.0" であること |
| `let_binding_basic` | `let x = 42` で束縛し、その後 `x` を使って計算できる |
| `let_binding_string` | `let name = String.trim(s)` が動作する |
| `let_with_record_spread` | `let updated = { ...row, x: val }` が動作する |
| `let_result_type_error` | Result 値に `let` を使うと E0326 が出る |

---

## 完了条件（PASS=5）

1. `let x = non_result_expr` で変数を束縛できる
2. `let` の後に `bind` を続けて使える（混在）
3. `let name = String.trim(s)` のような stdlib 呼び出しが動作する
4. `let updated = { ...row, field: val }` でレコードスプレッドと組み合わせられる
5. `let r = result_fn()` で E0326 が出る

---

## 非対応（スコープ外）

- トップレベル `let`（`fn` 外での使用） — スコープ外
- パターン分解 `let { name, score } = row` — v17.x 以降の検討
- 型アノテーション `let x: Int = 42` — 省略可能（型推論で対応）
- `let` によるシャドウイング — 同一スコープでの再定義は E0018（既存ルール）
