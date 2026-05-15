# Favnir v0.1.0 仕様書

更新日: 2026-04-26

## 概要

v0.1.0 は Favnir の最小実装。
純粋な pipeline を型安全に書き、インタープリタで実行できることを目標にする。

---

## スコープ

### 含むもの

- 基本型・組み込みコレクション型 (`List<T>`, `Map<K,V>` など)
- ユーザー定義型 (`type` による record / sum、ジェネリクスなし)
- 束縛 (`bind <-`)、パターン分解
- 関数 (`fn`)、クロージャ
- 変換 (`trf`)
- フロー (`flw`)
- パイプライン演算子 (`|>`)
- パターンマッチ (`match`)
- 条件式 (`if`)
- effect の構文・型注釈 (全種類をパース)
- effect の実行対応: `Pure`, `Io` のみ
- tree-walking インタープリタ
- CLI: `fav run`, `fav check`
- 組み込み関数の最小セット

### 含まないもの

- ユーザー定義ジェネリクス (`type Box<T>` や `fn id<T>` は書けない)
- `chain` 束縛
- `rune` / `namespace` / `use`
- `cap` / generics 制約
- `event` / `emit`
- `async` / `await`
- bytecode / VM
- `fav build` / `fav exec` / `fav fmt` / `fav test`

### ジェネリクスの扱い

ユーザーが新たに generic 型・generic 関数を定義することはできない。

ただし、組み込み型 (`List<T>`, `Map<K,V>`, `Option<T>`, `Result<T,E>`) は
型注釈の中で具体型を当てはめる形で使用できる。

```fav
// OK: 組み込み型に具体型を当てはめる
bind rows: List<Row> <- parse(text)

// NG: ユーザー定義のジェネリクス
type Box<T> = { value: T }     // v0.1.0 では不可
fn identity<T>(x: T) -> T { x } // v0.1.0 では不可
```

---

## 文法

### 記法

```
A*      = A の 0 回以上の繰り返し
A+      = A の 1 回以上の繰り返し
A?      = A の 0 回または 1 回
(A | B) = A または B
```

### トップレベル

トップレベルには定義のみを置く。`bind` はブロック内でのみ使用する。

```
program = item*
item    = type_def
        | fn_def
        | trf_def
        | flw_def
```

### 型定義

ユーザー定義型に型引数は持てない。

```
type_def    = "type" IDENT "=" type_body
type_body   = record_body
            | sum_body

record_body = "{" field* "}"
sum_body    = ("|" variant)+

field       = IDENT ":" type_expr
variant     = IDENT                       // unit variant:  Guest
            | IDENT "(" type_expr ")"     // tuple variant: ok(User)
            | IDENT "{" field* "}"        // record variant: Authenticated { user: User }
```

例:

```fav
type User = {
    name: String
    email: String
}

type Session =
    | Guest
    | Authenticated { user: User }

type ParseResult =
    | ok(User)
    | err(String)
```

### 型式

組み込みの多相型は型引数に具体型を与えて使用できる。

```
type_expr = base_type type_args?
          | type_expr "?"
          | type_expr "!"
          | type_expr "->" type_expr

base_type = IDENT
type_args = "<" type_expr ("," type_expr)* ">"
```

特殊型:

| 表記 | 意味 |
|---|---|
| `T?` | `Option<T>` の sugar |
| `T!` | `Result<T, Error>` の sugar |
| `A -> B` | 関数型 |

### 可視性

トップレベル定義には可視性修飾子を付けられる。デフォルトは `private`。

```
visibility = "public" | "private"
```

v0.1.0 では `public` と `private` のみ対応する。`internal` は後回し。

### 関数定義

型引数は持てない。

```
fn_def = visibility? "fn" IDENT "(" params? ")" "->" type_expr effect_ann? block
params = param ("," param)*
param  = IDENT ":" type_expr
```

例:

```fav
fn normalize_email(value: String) -> String {
    value.trim().lower()
}
```

### ### 変換定義

型引数は持てない。v0.1.0 の effect は `Pure` と `Io` のみ。`Db`, `Network`, `Emit` は文法上も存在しない。

```
trf_def     = visibility? "trf" IDENT ":" type_expr "->" type_expr effect_ann? "=" "|" params "|" block
effect_ann  = "!" effect_name
effect_name = "Pure" | "Io"
```

例:

```fav
trf ParseCsv: String -> List<Row> = |text| {
    ...
}

trf PrintRows: List<Row> -> Unit !Io = |rows| {
    ...
}
```

### フロー定義

```
flw_def  = "flw" IDENT "=" trf_ref ("|>" trf_ref)*
trf_ref  = IDENT
```

例:

```fav
flw ImportUsers =
    ParseCsv
    |> ValidateUser
    |> SaveUsers
```

### 束縛

`bind` はブロック内でのみ使用できる。トップレベルには書けない。

```
bind_stmt = "bind" pattern "<-" expr
```

pattern binding:

```fav
bind user            <- parse(row)      // 単純束縛
bind { name, email } <- user            // record 分解
bind ok(value)       <- parse_user(row) // variant 分解
```

### 式

```
expr = pipeline_expr

pipeline_expr = apply_expr ("|>" apply_expr)*

apply_expr = primary_expr actual_args?
actual_args = "(" (expr ("," expr)*)? ")"

primary_expr = literal
             | IDENT
             | closure
             | block
             | match_expr
             | if_expr
             | "(" expr ")"

literal = INT | FLOAT | STRING | "true" | "false" | "()"

closure = "|" closure_params? "|" expr
closure_params = IDENT ("," IDENT)*
```

### ブロック

ブロック内の中間文には `;` を必要とする。末尾の式がブロック全体の値になる。

```
block = "{" stmt* expr "}"
stmt  = bind_stmt
      | expr ";"
```

例:

```fav
{
    bind x <- compute();
    IO.println("step");
    x + 1
}
```

### パターンマッチ

```
match_expr = "match" expr "{" arm+ "}"
arm        = pattern "=>" expr ","?

pattern = "_"
        | literal
        | IDENT
        | IDENT "(" pattern ")"
        | "{" field_pattern ("," field_pattern)* "}"

field_pattern = IDENT | IDENT ":" pattern
```

例:

```fav
match result {
    ok(user)    => user.name
    err(msg)    => "error"
}

match session {
    Guest                  => "guest"
    Authenticated { user } => user.name
}
```

### 条件式

```
if_expr = "if" expr block ("else" block)?
```

`if` は式として扱う。値を返す。`else` を省略した場合の型は `Unit`。

```fav
bind label <- if rows.is_empty() {
    "empty"
} else {
    "non-empty"
}
```

---

## エントリポイント

`fav run` 実行時のエントリポイントは `main` 関数とする。

```fav
public fn main() -> Unit !Io {
    IO.println("hello")
}
```

規約:

- 関数名は `main`
- `public` が必須 (外部から呼び出す関数であるため)
- 引数なし
- 戻り値型は `Unit`
- effect は `Pure` または `Io` (標準出力を使う場合は `!Io`)
- ファイルに `main` が存在しない場合はエラー (E006)

---

## 型システム

### 基本型

| 型 | 説明 |
|---|---|
| `Bool` | 真偽値 |
| `Int` | 64bit 整数 |
| `Float` | 64bit 浮動小数点 |
| `String` | UTF-8 文字列 |
| `Unit` | 値なし |
| `List<T>` | 不変リスト (組み込み) |
| `Map<K, V>` | 不変マップ (組み込み) |
| `Option<T>` / `T?` | optional (組み込み) |
| `Result<T, E>` / `T!` | fallible (組み込み) |

### 型推論

v0.1.0 では単相的な推論のみ。

- `bind` の右辺から型を推論する
- `fn` / `trf` の引数型は必須
- 戻り値型は省略可能 (推論できる場合)
- ユーザー定義のジェネリクスは不可

### `trf` の内部型

`trf` は内部的に次の 3-tuple で表現する。

```
Trf<Input, Output, Fx>
```

例:

- `ParseCsv : Trf<String, List<Row>, Pure>`
- `ValidateUser : Trf<List<Row>, List<User>, Pure>`
- `SaveUsers : Trf<List<User>, List<UserId>, Db>`

### `flw` の合成型

```
Trf<A, B, F1> |> Trf<B, C, F2> = Trf<A, C, F1 + F2>
```

- 左辺の Output と右辺の Input が一致しなければ型エラー
- effect は和で合成される

---

## effect システム

### v0.1.0 の effect 集合

v0.1.0 では `Pure` と `Io` のみ。`Db`, `Network`, `Emit` は v0.2.0 以降で追加する。

| effect | 意味 |
|---|---|
| `Pure` | 副作用なし |
| `Io` | 標準入出力 |

### 合成規則

```
Pure + X  = X
Io   + Io = Io
```

### 型検査での扱い

- `Pure` な `trf` のみで構成された `flw` は `Pure` として扱う
- `Io` を含む `trf` を接続した `flw` は `Io` を持つ

---

## 実行モデル

### インタープリタ

v0.1.0 は tree-walking インタープリタ。

実行の流れ:

```
.fav ファイル
  -> Lexer -> Token 列
  -> Parser -> AST
  -> 型チェック -> Typed AST
  -> インタープリタ -> 実行結果
```

### 値の種類

| 種類 | 表現 |
|---|---|
| Bool | `true` / `false` |
| Int | 64bit 整数 |
| Float | 64bit 浮動小数点 |
| String | UTF-8 文字列 |
| Unit | `()` |
| List | 不変リスト |
| Map | 不変マップ |
| Closure | 環境 + AST |
| Trf | 名前 + 環境 + AST |
| ADT variant | タグ + payload |
| Record | フィールド名 → 値 のマップ |

### メモリ管理

v0.1.0 では参照カウント (`Rc`) を使う。GC は持たない。

### 実行環境

```
Env = {
    bindings: Map<Name, Value>
    parent: Env?
}
```

lexical scope で入れ子になる。

---

## 組み込み関数

v0.1.0 で提供する最小セット。シグネチャの型変数 `T`, `U` は組み込みの多相を示す (ユーザーが真似できる構文ではない)。

### IO

```
IO.print(value: String) -> Unit    // !Io
IO.println(value: String) -> Unit  // !Io
```

### List

```
List.map(items: List<T>, f: T -> U) -> List<U>
List.filter(items: List<T>, f: T -> Bool) -> List<T>
List.fold(items: List<T>, init: U, f: U -> T -> U) -> U
List.length(items: List<T>) -> Int
List.is_empty(items: List<T>) -> Bool
List.first(items: List<T>) -> T?
List.last(items: List<T>) -> T?
```

### String

```
String.trim(s: String) -> String
String.lower(s: String) -> String
String.upper(s: String) -> String
String.split(s: String, sep: String) -> List<String>
String.length(s: String) -> Int
String.is_empty(s: String) -> Bool
```

### Option

```
Option.some(value: T) -> T?
Option.none() -> T?
Option.map(opt: T?, f: T -> U) -> U?
Option.unwrap_or(opt: T?, default: T) -> T
```

### Result

```
Result.ok(value: T) -> T!
Result.err(message: String) -> T!
Result.map(r: T!, f: T -> U) -> U!
Result.unwrap_or(r: T!, default: T) -> T
```

---

## CLI

### `fav run`

```
fav run <file.fav>
```

- `.fav` ファイルをインタープリタで実行する
- `fn main() -> Unit !Io ` をエントリポイントとして呼び出す
- `main` が存在しない場合はエラー

### `fav check`

```
fav check <file.fav>
```

- 型チェック・effect チェックのみ行い、実行しない
- エラーがなければ `ok` を表示する

---

## エラー表示

エラーメッセージには次を含める。

- ファイル名
- 行番号・列番号
- エラーコード
- 簡潔な説明

例:

```
error[E001]: type mismatch
  --> main.fav:12:5
   |
12 |     rows |> SaveUsers
   |             ^^^^^^^^^ expected List<User>, got List<Row>
```

### エラーコード

| コード | 種類 |
|---|---|
| E001 | 型不一致 |
| E002 | 未定義の識別子 |
| E003 | `trf` 接続型不一致 |
| E004 | パターンの網羅性不足 |
| E005 | effect 実行非対応 |
| E006 | `main` 関数が見つからない |

---

## 非対応・後回し

以下は v0.1.0 では対応しない。

- ユーザー定義ジェネリクス
- `chain` 束縛
- `rune` / `namespace` / `use`
- `cap` (capability)
- `Db`, `Network`, `Emit` effect (文法・型検査・実行のすべて)
- `event` / `emit`
- `async` / `await`
- `internal` 可視性
- `fav build` / `fav exec` / `fav fmt` / `fav test`
- WASM backend
- self-hosting
