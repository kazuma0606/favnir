# Favnir v0.5.0 仕様書

更新日: 2026-04-29

## 概要

v0.5.0 は「ローカルな文脈付き合成とパターン表現力の強化」をテーマとするバージョン。

- **`chain` 束縛**: `Result<T,E>` / `Option<T>` を返す関数内でのモナディックなバインド。失敗を自動伝播する。
- **`pipe match` sugar**: `|> match { ... }` 構文。パイプライン内でパターンマッチを直接書ける。
- **パターンガード**: `where` 節。マッチアームに `Bool` ガード条件を付けられる。
- **`inspect`**: パイプライン途中で値を観測するユーティリティ。`!Trace` effect を導入。
- **`collect / yield`**: 逐次的なリスト構築。`collect { yield x; yield y; ... }` → `[x, y, ...]`。

---

## スコープ

### v0.5.0 で追加するもの

- `chain x <- expr` 束縛文（`bind` と対。モナディックなバインド）
- `chain` の型検査（`Result<T,E>` / `Option<T>` コンテキストの確認）
- `chain` のランタイムセマンティクス（失敗時の早期リターン）
- `|> match { ... }` 構文（パーサでその場でデシュガー）
- `where guard_expr` パターンガード（`MatchArm` に追加）
- `!Trace` effect（新エフェクトバリアント）
- `Trace.print<T>(value: T) -> T !Trace`（値を観測して返す組み込み）
- `collect { ... }` 式（`Expr::Collect` として AST に追加）
- `yield expr` 文（`Stmt::Yield` として AST に追加）

### v0.5.0 では含まないもの

- `chain` の自動エラー型変換（`E1` → `E2` への `map_err` 的な変換）
- `async / await`（`chain` は同期的）
- モナド抽象（`Monad<M>` cap）— `chain` は `Result` / `Option` のみ対象
- `for` ループや無限列 — `collect / yield` は逐次的・有限のみ
- `yield` からのクロージャ脱出（`yield` が使えるのは `collect` の直接ブロック内のみ）
- エラー型の型推論 — 関数の戻り型注釈から明示的に読む
- パターンの or 合成 (`A | B =>`) — 別途検討

---

## `chain` 束縛

### 概念

`chain x <- expr` は `bind x <- expr` の「失敗伝播版」。

- `expr` が `ok(v)` または `some(v)` に評価されたとき、`x` に `v` を束縛して継続する。
- `expr` が `err(e)` に評価されたとき、現在の関数から直ちに `err(e)` を返す（早期リターン）。
- `expr` が `none` に評価されたとき、現在の関数から直ちに `none` を返す。

### 構文

```
chain_stmt = "chain" IDENT "<-" expr
```

`bind` と同じく、`chain` 文にセミコロンは不要。
`chain` はブロック内の文として使う。関数・trf・クロージャの先頭から使用できる。

```fav
fn parse_and_double(input: String) -> Result<Int, String> {
    chain parsed <- parse_int(input)   // Result<Int, String>: err なら即リターン
    chain valid  <- validate(parsed)   // Result<Int, String>: err なら即リターン
    ok(valid * 2)
}
```

> **Note**: `Db.execute` は現在 `Int`（変更行数）を返すため、chain の対象にならない。
> chain は `Result<T, E>` または `Option<T>` を返す式にのみ使える（それ以外は E025）。

### 型規則

#### コンテキスト型

`chain` を使える関数の戻り型は `Result<T, E>` または `Option<T>` でなければならない。
その戻り型を「chain コンテキスト型」と呼ぶ。

| 関数の戻り型 | chain コンテキスト |
|---|---|
| `Result<T, E>` | Result モード |
| `Option<T>`    | Option モード |
| それ以外       | `chain` 使用不可（E024）|

#### `chain x <- expr` の型規則

```
関数の戻り型 = Result<_, E>
expr : Result<T, E>
─────────────────────────────
x : T  （chain 以降の続きで使える）
```

```
関数の戻り型 = Option<_>
expr : Option<T>
─────────────────────────────
x : T  （chain 以降の続きで使える）
```

- `expr` の型が chain コンテキストと合わない場合は E025。
- `expr` の エラー型 `E` が関数の戻り型エラー型と合わない場合は E025。

#### 例: Option 版

```fav
fn find_user(id: Int) -> Option<String> {
    chain row  <- Db.query_one("SELECT name FROM users WHERE id = ?", id)  // Option<Map<String,String>>
    chain name <- Map.get(row, "name")   // Option<String>
    some(name)
}
```

### 評価セマンティクス

評価器は `chain` 文を次のように扱う:

1. `expr` を評価する。
2. 結果が `ok(v)` または `some(v)` なら `x = v` を環境に束縛して継続。
3. 結果が `err(e)` なら `ChainEscape(err(e))` を発行し、現在の関数境界まで伝播させる。
4. 結果が `none` なら `ChainEscape(none)` を発行する。

#### `ChainEscape` の伝播

関数呼び出しの評価（`eval_call`）で `ChainEscape` を受け取った場合、その値を関数の返り値として返す。
`ChainEscape` はブロック・if 分岐・match アームを通過して伝播するが、**関数境界で止まる**。

内部表現（評価器の実装）:

```rust
// eval.rs 内でブロック評価の戻り型を変更
enum EvalResult {
    Value(Value),
    ChainEscape(Value),   // 早期リターン
}
```

クロージャも関数境界なので、クロージャ内の `chain` はそのクロージャから早期リターンする（外の関数には影響しない）。

### `chain` と effect

`chain` 自体は新しい effect を持たない。`chain` の右辺で呼ぶ関数の effect が、そのまま enclosing fn/trf に要求される。通常の関数呼び出しと同じ扱い。

---

## `pipe match` sugar

### 概念

パイプライン `|>` の右辺に `match { ... }` を直接書ける syntax sugar。

### 構文

```
pipe_match = expr "|>" "match" "{" match_arm+ "}"
```

### デシュガー（パース時に変換）

```fav
expr |> match {
    pattern1 => body1
    pattern2 => body2
}
```

→ パーサが直ちに次の形に変換する:

```fav
match expr {
    pattern1 => body1
    pattern2 => body2
}
```

パイプラインの中間でも使える:

```fav
fn process(input: String) -> Int {
    input
    |> parse_int
    |> match {
        ok(n)  => n * 2
        err(_) => 0
    }
}
```

### 実装

パーサの `parse_pipe_expr` で、`|>` の後に `match` トークンが来た場合:

1. 左辺の式を `subject` として保持する。
2. `match` トークンを消費。
3. `parse_match_arms()` でアームをパース。
4. `Expr::Match(subject, arms)` を生成して返す。

新しい AST ノードは不要。

---

## パターンガード

### 概念

`match` のアームにオプショナルな `Bool` 条件 (`where` 節) を付けられる。

- パターンがマッチしても、ガード式が `false` ならそのアームをスキップして次のアームを試みる。
- ガードは型 `Bool` でなければならない（E027）。

### 構文

```
match_arm = pattern ("where" expr)? "=>" (expr | block)
```

```fav
fn classify(n: Int) -> String {
    match n {
        x where x < 0  => "negative"
        x where x == 0 => "zero"
        x               => "positive"
    }
}
```

ADT パターンとの組み合わせ:

```fav
fn process(result: Result<Int, String>) -> String {
    match result {
        ok(n) where n > 100 => "large"
        ok(n) where n > 0   => "small positive"
        ok(_)               => "non-positive"
        err(e)              => e
    }
}
```

### AST の変更

```rust
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,   // 追加: where 節
    pub body: Expr,
    pub span: Span,
}
```

### Checker の変更

- ガード式が存在する場合、`check_expr(guard)` を呼んで型が `Bool` であることを確認する（E027）。

### 評価セマンティクス

1. パターンマッチを試みる。
2. マッチしたら、ガード式を評価する。
3. ガードが `true` なら body を評価して返す。
4. ガードが `false` なら次のアームを試みる。

---

## `inspect` と `!Trace` effect

### 概念

パイプラインや処理中に値を「覗き見る」ための仕組み。値を変えずにデバッグ出力を行う。

`!Io` とは別の `!Trace` effect を導入する。`!Trace` は診断用出力専用。

- 本番では `!Trace` を無音化する（将来対応）。
- `!Io` はユーザー向け出力、`!Trace` はデバッグ用出力として意味を区別する。

### `!Trace` effect

新しい `Effect::Trace` バリアント:

```rust
pub enum Effect {
    // 既存 ...
    Trace,   // 追加
}
```

effect 表記: `!Trace`

### 組み込み関数: `Trace.print`

```fav
// 組み込み (Rust で実装)
// Trace.print(value: T) -> T !Trace
// 値の Debug 表現を標準エラー出力に出力し、value をそのまま返す。
```

型: `T -> T !Trace`（generics によって多相）

パイプライン内での使用:

```fav
fn process(n: Int) -> Int !Trace {
    n |> Double |> Trace.print |> Inc
}
```

複数の関数を組み合わせる場合:

```fav
fn debug_step(label: String, value: Int) -> Int !Trace {
    Trace.print(value)
}
```

### `Trace.log`: ラベル付き出力

```fav
// 組み込み
// Trace.log(label: String, value: T) -> T !Trace
// "{label}: {debug_repr}" を出力して value を返す。
```

```fav
fn calc(n: Int) -> Int !Trace {
    bind doubled <- Trace.log("input", n) |> Double
    Trace.log("doubled", doubled)
}
```

### `!Trace` と `fav explain`

`fav explain` の出力テーブルでは、`!Trace` 付きの関数・trf の EFFECTS 列に `!Trace` が表示される。

```
NAME            TYPE          EFFECTS    VIS
process         Int -> Int    !Trace     private
labeled_trace   String -> Int !Trace     private
```

複数 effect を持つ場合は通常通りスペース区切りで並ぶ:

```
NAME            TYPE          EFFECTS         VIS
fetch_and_trace String -> Int !Network !Trace private
```

### 評価器の実装

`Trace.print(value)`:
1. `Debug.show` 相当の表現を標準エラー出力 (`eprintln!`) に出力する。
2. `value` をそのまま返す。

`Trace.log(label, value)`:
1. `"label: {repr}"` 形式で標準エラー出力に出力する。
2. `value` をそのまま返す。

---

## `collect / yield`

### 概念

`collect { ... }` ブロックで逐次的にリストを構築する。`yield expr` で要素を追加する。

シンプルな用途:

```fav
bind xs <- collect {
    yield 1;
    yield 2;
    yield 3;
}
// xs : List<Int> = [1, 2, 3]
```

条件を使った用途:

```fav
bind evens <- collect {
    if 2 > 1 { yield 2 };
    if 1 > 2 { yield 1 };  // このアームには入らない
    yield 4;
}
// evens : List<Int> = [2, 4]
```

### スコープ制限

`yield` は `collect` ブロックの**直接内部**からのみ使える（クロージャ内からは使えない）。

```fav
// NG: yield がクロージャの中
bind xs <- collect {
    bind _  <- List.map([1, 2, 3], |x| { yield x })  // E026
    ()
}

// OK: yield が collect 直下
bind xs <- collect {
    yield 1;
    yield 2;
}
```

### 型規則

- `collect { ... }` の型は `List<T>` 。
- 全ての `yield expr` の `expr` 型が一致しなければならない（一致しない場合は E001）。
- `yield` が一つもない場合の型は `List<Unknown>`（空リスト）。

### 構文

```
collect_expr = "collect" block
yield_stmt   = "yield" expr ";"
```

`collect` はブロックの末尾式としても、`bind x <- collect { ... }` としても使える。

### 評価セマンティクス

1. 評価器がスレッドローカルの accumulator スタックを持つ（`COLLECT_STACK`）。
2. `collect { block }` に入るとき: 空の `Vec<Value>` をスタックにプッシュ。
3. `yield expr` を評価: `expr` を評価してスタックトップに追加、`Value::Unit` を返す。
4. `collect` ブロックを抜けるとき: スタックからポップして `Value::List(...)` を返す。

```rust
// eval.rs
thread_local! {
    static COLLECT_STACK: RefCell<Vec<Vec<Value>>> = RefCell::new(Vec::new());
}
```

`yield` が `collect` 外で使われた場合、スタックが空 → E026 を checker が事前に検出する。

---

## AST の変更

### 新キーワード

| トークン | キーワード |
|---|---|
| `TokenKind::Chain`   | `chain`   |
| `TokenKind::Yield`   | `yield`   |
| `TokenKind::Collect` | `collect` |
| `TokenKind::Where`   | `where`   |

### `Stmt` の拡張

現在の `Stmt` に新バリアントを追加する:

```rust
pub enum Stmt {
    // 既存
    Bind  { name: String, expr: Expr, span: Span },
    Expr  { expr: Expr, span: Span },
    // 追加
    Chain { name: String, expr: Expr, span: Span },
    Yield { expr: Expr, span: Span },
}
```

### `Expr` の拡張

```rust
pub enum Expr {
    // 既存 ...
    Collect(Box<Block>, Span),   // collect { ... }
}
```

### `MatchArm` の拡張

```rust
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Box<Expr>>,  // 追加: where 節
    pub body: Expr,
    pub span: Span,
}
```

### `Effect` の拡張

```rust
pub enum Effect {
    // 既存 ...
    Trace,   // 追加: !Trace
}
```

---

## Checker の変更

### `chain` 束縛の検査

`check_fn_def` および `check_trf_def` で「chain コンテキスト型」を記録する:

```rust
pub struct Checker {
    // 既存 ...
    chain_context: Option<Type>,  // 追加: Result<_,_> or Option<_> or None
}
```

- `check_fn_def` 冒頭で戻り型を解決し、`Result<_,_>` または `Option<_>` なら `chain_context` にセット。
- `check_fn_def` 終了時に `chain_context = None` にリセット。

`check_stmt` (または相当する処理) で `Stmt::Chain` を処理:

```
chain_context が None → E024 (chain コンテキストなし)
expr の型が Result<T, E> で chain_context が Result<_, E'> → E と E' を unify → 失敗なら E025
expr の型が Option<T>    で chain_context が Option<_>    → OK
x の型 = T として環境に登録
```

### パターンガードの検査

`check_match_arm` でガードが存在する場合:

```rust
if let Some(guard_expr) = &arm.guard {
    let guard_ty = self.check_expr(guard_expr);
    if !guard_ty.is_compatible(&Type::Bool) {
        self.type_error("E027", "pattern guard must be of type Bool", span);
    }
}
```

### `collect / yield` の検査

- `in_collect: bool` フィールドを Checker に追加。
- `Expr::Collect(block)` を処理するとき `in_collect = true` にセットしてブロックをチェック。
- `Stmt::Yield { expr }` を処理するとき `in_collect` が `false` なら E026。
- クロージャに入るとき `in_collect = false` にリセット（クロージャ内での `yield` はスコープ外）。
- `collect` の型: `yield` された expr の型を全て unify し、`Type::List(unified_type)` を返す。

### `!Trace` effect の検査

`Trace.print(...)` / `Trace.log(...)` の呼び出し時:

- 既存の `require_db_effect` / `require_network_effect` と同様のパターンで `require_trace_effect` を追加。
- `!Trace` なしで `Trace.*` を呼ぶと E010 を報告する（新コード）。

---

## 評価器の変更

### `ChainEscape` の導入

`eval_block` / `eval_fn_call` の内部で `EvalResult` を使う:

```rust
enum EvalResult {
    Value(Value),
    Escape(Value),   // chain の早期リターン値
}
```

- `eval_block` が `EvalResult` を返すよう変更する。
- `Stmt::Chain` を処理: 右辺を評価し、`err(e)` / `none` なら `EvalResult::Escape(...)` を返す。
- 親ブロックは `Escape` を受け取ったら即座に上に返す。
- 関数呼び出し（`eval_call`）で `Escape` を受け取ったら、それを関数の返り値として `Value` に変換して返す。

### `Trace.print` / `Trace.log` の実装

```rust
"Trace" => match field.as_str() {
    "print" => Value::Builtin("trace_print", ""),
    "log"   => Value::Builtin("trace_log", ""),
    _ => Value::Unknown,
}
```

`eval_builtin("trace_print", args)`:

```rust
// args = [value]
let repr = args[0].repr();
eprintln!("{}", repr);
args[0].clone()
```

`eval_builtin("trace_log", args)`:

```rust
// args = [label: String, value]
let label = args[0].as_str();
let repr  = args[1].repr();
eprintln!("{}: {}", label, repr);
args[1].clone()
```

### `COLLECT_STACK` の実装

```rust
thread_local! {
    static COLLECT_STACK: RefCell<Vec<Vec<Value>>> = RefCell::new(Vec::new());
}

fn collect_push_frame()              { COLLECT_STACK.with(|s| s.borrow_mut().push(vec![])); }
fn collect_yield(v: Value)           { COLLECT_STACK.with(|s| s.borrow_mut().last_mut().unwrap().push(v)); }
fn collect_pop_frame() -> Vec<Value> { COLLECT_STACK.with(|s| s.borrow_mut().pop().unwrap_or_default()) }
```

`Expr::Collect(block)` の評価:

```rust
collect_push_frame();
eval_block(block, env);   // yield 文が COLLECT_STACK に追加する
let items = collect_pop_frame();
Value::List(items)
```

`Stmt::Yield { expr }`:

```rust
let v = eval_expr(expr, env);
collect_yield(v);
EvalResult::Value(Value::Unit)
```

---

## エラーコード

| コード | 内容 |
|---|---|
| E010 | `!Trace` エフェクトなしで `Trace.*` を使用 |
| E024 | `chain` を `Result` / `Option` を返さない関数内で使用した |
| E025 | `chain` の式の型が関数の chain コンテキスト型と合わない |
| E026 | `yield` を `collect` ブロック外で使用した |
| E027 | パターンガード (`where`) の型が `Bool` でない |

---

## 新しい構文のまとめ

### `chain` 束縛

```
chain_stmt = "chain" IDENT "<-" expr
```
（セミコロン不要。`bind` と同じ規則）

### パターンガード

```
match_arm = pattern ("where" expr)? "=>" expr
```

### `pipe match`

```
pipe_match = expr "|>" "match" "{" match_arm+ "}"
// デシュガー後: match expr { match_arm+ }
```

### `collect / yield`

```
collect_expr = "collect" block
yield_stmt   = "yield" expr ";"
```

---

## 例

### `chain` — Result の伝播

```fav
fn parse_int(s: String) -> Result<Int, String> {
    // 組み込みのパース（エラーは err(msg) で返す）
    String.parse_int(s)
}

fn double_parsed(s: String) -> Result<Int, String> {
    chain n <- parse_int(s)
    ok(n * 2)
}

public fn main() -> Unit {
    bind r1 <- double_parsed("21")
    bind r2 <- double_parsed("abc")
    IO.println(Debug.show(r1));   // ok(42)
    IO.println(Debug.show(r2));   // err("parse error")
    ()
}
```

### `chain` — Option の伝播

```fav
fn find_user(id: Int) -> Option<String> {
    chain row  <- Db.query_one("SELECT name FROM users WHERE id = ?", id)
    chain name <- Map.get(row, "name")
    some(name)
}
```

### `pipe match` sugar

```fav
fn classify_score(score: Int) -> String {
    score
    |> match {
        n where n >= 90 => "A"
        n where n >= 70 => "B"
        n where n >= 50 => "C"
        _               => "F"
    }
}
```

### パターンガード

```fav
fn safe_div(a: Int, b: Int) -> Option<Int> {
    match b {
        x where x == 0 => none
        x               => some(a / x)
    }
}
```

### `inspect` (Trace)

```fav
fn process(n: Int) -> Int !Trace {
    n |> Double |> Trace.print |> Inc
}

fn labeled_trace(s: String) -> Int !Trace {
    bind n <- String.parse_int(s)
        |> match {
            ok(v)  => v
            err(_) => 0
        }
    Trace.log("parsed", n)
}
```

### `collect / yield`

```fav
fn squares(n: Int) -> List<Int> {
    collect {
        yield n * n;
        yield (n + 1) * (n + 1);
        yield (n + 2) * (n + 2);
    }
}

public fn main() -> Unit {
    bind xs <- squares(3)
    IO.println(Debug.show(xs));   // [9, 16, 25]
    ()
}
```

### 組み合わせ例: chain + pipe match + guard

```fav
fn process_input(raw: String) -> Result<String, String> {
    chain n <- parse_int(raw)
    bind  s <- n
        |> match {
            x where x < 0  => err("negative")
            x where x > 99 => err("too large")
            x               => ok(Int.show.show(x))
        }
    chain result <- s
    ok("Result: " ++ result)
}
```

---

## 完了条件

- `chain x <- expr` が `Result<T,E>` / `Option<T>` を返す関数内で動作する
- `chain` で `err(e)` / `none` が来た場合、関数から早期リターンされる
- クロージャ内の `chain` はクロージャから早期リターンする（外の関数には波及しない）
- `expr |> match { ... }` が `match expr { ... }` と等価に動作する
- `match` のアームに `where guard` が書けて、ガードが `false` なら次のアームを試みる
- `Trace.print(value)` が値を返しつつ標準エラーに出力する
- `!Trace` なしで `Trace.*` を使うと E010 が出る
- `collect { yield x; yield y; }` が `[x, y]` を返す
- `yield` を `collect` 外で使うと E026 が出る
- E024〜E027 が適切に報告される
- 既存 181 テストが全パス（デグレなし）
- 新規テスト（各機能ごと）が全パス
