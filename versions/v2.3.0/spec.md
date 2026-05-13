# Favnir v2.3.0 仕様書

作成日: 2026-05-13

---

## テーマ

型チェッカーが知っていることを書き手に還元し、定型コードを削減する。

v2.3.0 では 2 つの糖衣構文を追加する。

1. **分割 bind（destructuring bind）** — レコード型の各フィールドを一括で束縛する
2. **戻り型推論** — 単式ボディの関数から戻り型の明示を省略できる

---

## 1. 分割 bind（destructuring bind）

### 構文

```favnir
bind { name, age } <- fetch_user(id)
```

これは次と等価：

```favnir
bind name <- fetch_user(id).name
bind age  <- fetch_user(id).age
```

ただし右辺の評価は 1 回のみ（中間変数へ格納して共有する）。

### エイリアス構文

```favnir
bind { age: user_age } <- fetch_user(id)
// age フィールドを user_age として束縛
```

### 残余無視

```favnir
bind { name, _ } <- fetch_user(id)
// name だけ束縛、_ は無視
```

### 型チェック

- 右辺の型がレコード型でなければ **E072** を報告する
- 指定したフィールド名が右辺の型に存在しなければ **E073** を報告する
- エイリアス付きフィールドは元の型が束縛先変数に引き継がれる

### エラーコード

| コード | 意味 |
|---|---|
| E072 | 分割 bind の右辺がレコード型ではない |
| E073 | 指定したフィールドがレコード型に存在しない |

### 使用例

```favnir
type Point = { x: Int  y: Int }

bind pt <- Point { x: 3  y: 4 }
bind { x, y } <- pt

IO.println_int(x)  // 3
IO.println_int(y)  // 4
```

```favnir
type User = { name: String  age: Int  role: String }

bind user <- fetch_user(42)
bind { name, age: user_age } <- user
// name: String, user_age: Int として束縛される
```

---

## 2. 戻り型推論

### 構文

```favnir
fn double(n: Int) = n * 2
fn greet(name: String) = $"Hello {name}!"
```

`->` と戻り型の明示を省略し、`=` に続けて単一の式を書く。
チェッカーが本体式の型を推論して戻り型として採用する。

### 混在

```favnir
fn id(x: Int) -> Int = x       // 明示あり（従来構文）
fn double(n: Int) = n * 2      // 推論（新構文）
```

どちらも有効。同一ファイル内での混在を許可する。

### 制約

- **再帰関数は推論不可**。本体評価に戻り型が必要なため、明示アノテーションが必須：
  ```favnir
  fn factorial(n: Int) -> Int = if n <= 1 { 1 } else { n * factorial(n - 1) }
  ```
- **エフェクトは別途明示**が必要（推論対象外）：
  ```favnir
  fn print_hello() -> Unit !Io = IO.println("hello")
  ```
  エフェクトを省略した場合は `Unit` かつエフェクトなしとして扱う（既存ルール）。

### 型エラー

- 本体式から型が決定できない場合は **E074** を報告する

### エラーコード

| コード | 意味 |
|---|---|
| E074 | 戻り型推論が不可能な式（Unknown 型になった場合など） |

### 使用例

```favnir
fn double(n: Int) = n * 2
fn triple(n: Int) = n * 3
fn square(n: Int) = n * n

fn greet(name: String) = $"Hello {name}!"
fn is_adult(age: Int) = age >= 18
```

---

## 3. 実装箇所サマリ

### 分割 bind

| フェーズ | ファイル | 状態 |
|---|---|---|
| Parser | `frontend/parser.rs` — `bind { ... }` を `Stmt::Bind(Pattern::Record, expr)` として解析 | 実装済み |
| Checker | `middle/checker.rs` — `check_pattern_bindings(Pattern::Record, ...)` で型チェック | 実装済み |
| Compiler | `middle/compiler.rs` — `Stmt::Bind` + `Pattern::Record` の脱糖が未実装 | **要実装** |

コンパイラの現状：`Stmt::Bind` で `Pattern::Record` が来た場合、`ctx.define_pattern_slot()` で
1 つの匿名スロットを確保するだけで個別フィールドのローカルを定義しない。

必要な変換（中間変数 `$tmp` を経由）：

```
bind { name, age } <- expr
↓
bind $tmp <- expr
bind name <- $tmp.name
bind age  <- $tmp.age
```

### 戻り型推論

| フェーズ | ファイル | 状態 |
|---|---|---|
| Parser | `frontend/parser.rs` — `->` なしで `=` を受け付ける | **要実装** |
| AST | `ast.rs` — `FnDef.return_ty: TypeExpr` を `Option<TypeExpr>` に変更 | **要実装** |
| Checker | `middle/checker.rs` — `return_ty` が `None` の場合に本体式の型を推論 | **要実装** |

---

## 4. 互換性

- v2.2.0 以前の全コードはそのまま有効
- 既存の `fn name(params) -> RetTy { body }` 構文は変更なし
- 分割 bind はパーサー・チェッカーが既に対応済みのため、コンパイラ修正のみで動作する
- 戻り型推論は AST 変更を伴うが、`return_ty` が `Some` の場合の動作は変わらない
