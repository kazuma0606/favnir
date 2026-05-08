# fav — Favnir interpreter

**Favnir** は型付きパイプラインとエフェクトを中心に設計された関数型データ言語です。
`fav` はそのリファレンス実装（ツリーウォーキングインタープリタ）です。

## インストール

```bash
cargo build --release
# バイナリは target/release/fav に生成されます
```

## 使い方

```
fav run [--db <path>] [file]          — 型チェック後にプログラムを実行
fav build [-o <out>] [--target wasm] [file] — .fvc / .wasm アーティファクトをビルド
fav exec [--db <path>] [--info] <fvc> — .fvc を VM で実行（または情報表示）
fav check [--no-warn] [file]          — 型チェックのみ（--no-warn で W010 等を抑制）
fav explain [file]                    — 全トップレベル項目の型・エフェクトを表示
fav fmt [--check] [file]              — コードフォーマット
fav lint [file]                       — 静的解析（L002/L003/L004）
fav test [file]                       — テスト実行（*.test.fav）
fav lsp                               — LSP サーバー起動（stdin/stdout JSON-RPC）
fav install                           — fav.toml の依存を解決して fav.lock を生成
fav publish                           — ローカルレジストリへ公開
fav help                              — ヘルプを表示
```

`file` を省略するとカレントディレクトリから `fav.toml` を探して**プロジェクトモード**で動作します。

## v1.5.0

v1.5.0 では次を追加しています。

- `fav explain diff <from> <to>`
- `fav graph --focus fn --entry <name> --depth <n>`
- top-level `effect Name`
- lint `L005` / `L006` / `L007`

例:

```bash
fav explain diff examples/diff_demo/old.fav examples/diff_demo/new.fav
fav explain diff examples/diff_demo/old.fav examples/diff_demo/new.fav --format json
fav graph examples/hello.fav --focus fn --entry main --depth 2
fav check examples/custom_effects.fav
fav lint examples/custom_effects.fav
```

### 実行例

```bash
fav run examples/hello.fav
fav run examples/pipeline.fav
fav run examples/adt_match.fav
fav check examples/hello.fav

# SQLite を使う場合 (v0.2.0)
fav run --db myapp.db  examples/users.fav   # ファイル DB
fav run --db :memory:  examples/users.fav   # インメモリ DB (デフォルト)

# シグネチャ一覧 (v0.2.0)
fav explain examples/users.fav

# プロジェクトモード (v0.3.0) — fav.toml のあるディレクトリで実行
cd examples/multi_file
fav check                      # src/ 配下の全 .fav をチェック
fav run --db :memory:          # src/main.fav をエントリポイントとして実行
fav explain                    # 全ファイルの定義を VIS 列付きで一覧
```

### プロジェクト構成 (v0.3.0)

複数ファイルで構成するプロジェクトは `fav.toml` で管理します。

```toml
[rune]
name    = "my_project"
version = "0.1.0"
src     = "src"          # ソースルートディレクトリ（省略時: "."）
```

ファイルは `src/` 配下に置き、ディレクトリ構造がモジュールパスになります。
`src/data/users.fav` → モジュールパス `data.users`

#### namespace 宣言

```favnir
namespace data.users
```

ファイル先頭に書き、モジュールパスを宣言します（省略可）。

#### use 宣言

```favnir
use data.users.create
use data.users.find
```

別ファイルのシンボルを import します。最後のセグメントがシンボル名、それ以前がモジュールパスです。

#### 可視性

| 修飾子     | スコープ |
|-----------|---------|
| (なし)    | そのファイル内のみ (`private`) |
| `internal`| 同一 rune (プロジェクト) 内のみ |
| `public`  | 外部からも import 可 |

```favnir
fn private_helper() -> Unit { () }           // private（デフォルト）
internal fn setup_table() -> Int !Db { ... } // internal
public fn create(name: String) -> Int { ... } // public
```

## ジェネリクスと Cap (v0.4.0)

### ジェネリック関数 / ジェネリック型

型パラメータは `<T>` 形式で宣言します。

```favnir
fn identity<T>(x: T) -> T { x }

type Pair<A, B> = { first: A  second: B }

fn make_pair<A, B>(a: A, b: B) -> Pair<A, B> {
    Pair { first: a  second: b }
}
```

### Cap（ケイパビリティ）定義

`cap` キーワードでインタフェース（トレイトに相当）を定義します。

```favnir
cap Eq<T> = {
    equals: T -> T -> Bool
}
```

### impl（実装）

`impl` で既存の型に cap を実装します。

```favnir
impl Eq<Int> {
    fn equals(a: Int, b: Int) -> Bool { a == b }
}
```

ユーザー定義型への実装：

```favnir
type User = { name: String  age: Int }

impl Eq<User> {
    fn equals(a: User, b: User) -> Bool {
        a.name == b.name
    }
}
```

### Cap インスタンスの利用

`TypeName.cap_name.method(...)` で cap メソッドを呼び出します。

```favnir
bind same <- Int.eq.equals(1, 1)       // true
bind cmp  <- Int.ord.compare(3, 7)     // 負の数
bind s    <- Int.show.show(42)         // "42"
bind ok   <- User.eq.equals(alice, alice2)
```

### 組み込み Cap

| Cap    | 対応型 | メソッド |
|--------|--------|---------|
| `Eq`   | Int, Float, String, Bool | `equals: T -> T -> Bool` |
| `Ord`  | Int, Float, String       | `compare: T -> T -> Int` |
| `Show` | Int, Float, String, Bool | `show: T -> String`      |

## Interface System (v1.1.0)

v1.1.0 adds `interface` as the primary abstraction mechanism. Legacy `cap` still works for compatibility, but `fav check` now emits `W010` for it.

### Interface declaration

```favnir
interface Show {
    show: Self -> String
}

interface Eq {
    eq: Self -> Self -> Bool
}

interface Ord : Eq {
    compare: Self -> Self -> Int
}
```

### Manual implementation

```favnir
type Point = { x: Int  y: Int }

impl Show for Point {
    show = |p| "Point"
}
```

Runtime call shape stays namespace-oriented:

```favnir
Point.show.show(p)
Int.ord.compare(1, 2)
```

### Auto-synthesis and `with`

```favnir
type UserRow with Show, Eq = { name: String  age: Int }
```

This is equivalent in intent to a record declaration plus:

```favnir
impl Show, Eq for UserRow
```

### Built-in interfaces

v1.1.0 registers these built-ins in the checker:

- `Show`, `Eq`, `Ord`
- `Gen`
- `Semigroup`, `Monoid`, `Group`, `Ring`, `Field`

Examples:

- `examples/interface_basic.fav`
- `examples/interface_auto.fav`
- `examples/algebraic.fav`

---

## invariant + std.states (v1.2.0)

v1.2.0 adds invariant-bearing record states and a virtual standard module of reusable validated state types.

### Invariant-bearing types

```favnir
type Email = {
    value: String
    invariant String.contains(value, "@")
    invariant String.length(value) > 3
}
```

Every invariant-bearing record type gets a synthetic constructor:

```favnir
Email.new("user@example.com")  // ok(Email { ... })
Email.new("bad")               // err("InvariantViolation: Email")
```

## abstract trf / abstract flw (v1.3.0)

v1.3.0 adds reusable abstract pipeline templates.

### abstract trf

```favnir
abstract trf FetchUser: Int -> String !Db
```

An `abstract trf` is a contract only. It participates in checking and template binding, but direct runtime calls are rejected with `E051`.

### abstract flw

```favnir
abstract flw DataPipeline<Row> {
    parse: String -> Row
    save: Row -> String
}
```

Slots define the required input/output contract for each step in the template.

### Binding a concrete flow

```favnir
flw UserImport = DataPipeline<UserRow> {
    parse <- ParseUser
    save <- SaveUser
}
```

If every slot is bound, the flow becomes executable. If some slots are missing:

- `fav check` emits `W011`
- `fav run` / `fav build` fail with `E050`

### Examples

- `examples/abstract_flw_basic.fav`
- `examples/abstract_flw_inject.fav`

## Explain JSON, Bundle, Graph (v1.4.0)

Favnir v1.4.0 adds artifact-oriented inspection and flow graph output.

### `fav explain --format json`

```bash
fav explain examples/abstract_flw_basic.fav --format json
fav explain examples/abstract_flw_basic.fav --format json --focus trfs
```

JSON output includes:

- entry/source metadata
- `fns`, `trfs`, `flws`, `types`
- reachability flags
- `effects_used`
- `emits`
- `runes_used`

### `fav bundle`

```bash
fav bundle examples/bundle_demo.fav -o dist/app.fvc --manifest --explain
```

Behavior:

- bundles only entry-reachable code
- writes a manifest with `included` / `excluded`
- can emit a sibling explain JSON file
- embeds explain JSON into the `.fvc` artifact

You can read embedded explain metadata back with:

```bash
fav explain dist/app.fvc --format json
```

### `fav graph`

```bash
fav graph examples/abstract_flw_basic.fav
fav graph examples/abstract_flw_basic.fav --format mermaid
```

Current graph output focuses on:

- `flw`
- `abstract flw`
- bound flow templates

### Generic abstract-trf slot shorthand

Slots in an abstract flow can now reference generic abstract transforms directly:

```favnir
abstract trf Parse<Row>: String -> Row

abstract flw Pipeline<Row> {
    parse: Parse<Row>
    render: Row -> String
}
```

See:

- `examples/bundle_demo.fav`
- `examples/dynamic_inject.fav`

Typed bind is also invariant-aware:

```favnir
bind age: PosInt <- 25
```

### std.states

`use std.states.*` exposes common validated states:

- `PosInt`
- `NonNegInt`
- `Probability`
- `PortNumber`
- `NonEmptyString`
- `Email`
- `Url`
- `Slug`

Examples:

- `examples/invariant_basic.fav`
- `examples/std_states.fav`

### explain and schema

- `fav explain` now shows an `INVARIANTS` column for record types
- `fav explain --schema` lowers supported invariants to SQL `CHECK` output

---

## chain / collect / pipe match (v0.5.0)

### chain — Result / Option の早期脱出

`chain` は `Result<T,E>` または `Option<T>` を返す関数の中で使えるモナディックバインドです。
`err(e)` または `none` が返ってきた場合、その値を関数の戻り値として即座に脱出します。

```favnir
fn parse(s: String) -> Int! { Result.ok(42) }

fn validate(n: Int) -> Int! {
    if n > 0 { Result.ok(n) } else { Result.err("negative") }
}

fn process(s: String) -> Int! {
    chain n <- parse(s)     // parse が err なら即 err で脱出
    chain v <- validate(n)  // validate が err なら即 err で脱出
    Result.ok(v)
}
```

`chain` の後にセミコロンは不要です（`bind` と同じ）。

### collect / yield — リスト構築

`collect { }` ブロック内で `yield expr;` を使うと要素を追加できます。
ブロックの戻り値は `List<T>` になります。

```favnir
fn nums() -> List<Int> {
    collect {
        yield 1;
        yield 2;
        yield 3;
        ()
    }
}
```

### pipe match — `|> match`

パイプラインの末尾に `match` を続けると、左辺値を scrutinee として直接マッチできます。

```favnir
fn classify(n: Int) -> String {
    n |> match {
        x where x > 0 => "positive"
        x where x < 0 => "negative"
        _              => "zero"
    }
}
```

### where — パターンガード

`match` アームのパターン後に `where 条件式` を書くと、条件が `false` の場合はそのアームをスキップします。

```favnir
match value {
    n where n > 100 => "large"
    n where n > 0   => "small"
    _               => "nonpositive"
}
```

### !Trace — デバッグトレース

`!Trace` エフェクトで `Trace.print` / `Trace.log` が使えます（stderr に出力）。

```favnir
fn debug_fn(x: Int) -> Int !Trace {
    Trace.log("x", x);
    x * 2
}
```

---

## bytecode コンパイルと VM 実行 (v0.6.0)

### fav build — artifact のビルド

```bash
fav build examples/hello.fav            # hello.fvc を生成
fav build examples/hello.fav -o out.fvc # 出力先を指定
```

`.fav` ソースを型チェック後にバイトコードコンパイルし、`.fvc` アーティファクトを生成します。
型チェックエラーがある場合は非 0 で終了し、`.fvc` は生成しません。

### fav exec — artifact の実行

```bash
fav exec hello.fvc
fav exec --db myapp.db hello.fvc    # (v0.7.0 で Db.* 対応予定)
```

`.fvc` アーティファクトをロードして `main` 関数を VM で実行します。

### fav exec --info — artifact のメタデータ表示

```bash
fav exec --info hello.fvc
```

`.fvc` のヘッダ情報・グローバルテーブル・関数テーブル（命令列・定数プール）を表示します。
出力例:

```
artifact: hello.fvc
  format:     .fvc v0.6
  functions:  1
  globals:    13
  bytecode:   18 bytes (total)
  constants:  1 (total)
  str table:  22 entries, 0 bytes total, longest 0

globals:
  [0]  fn      main

functions:
  [0] main  params=0 locals=0  line=2
    return_ty: Named("Unit", [])
    effects:   [Io]
    constants: [Str("Hello, Favnir!")]
    code (18 bytes):
      0000  12 00 00  LOAD_GLOBAL    0
      ...
```

### 対応エラーコード (v0.6.0)

| コード | 内容 |
|--------|------|
| E032   | アーティファクトファイルが見つからない |
| E033   | マジックバイト不一致（`.fvc` でない） |
| E034   | バージョン非互換（異なる VM バージョン） |
| E035   | `main` 関数がアーティファクトに存在しない |

### ビルドと実行のサイクル

```bash
# ソース編集 → ビルド → 実行
fav build examples/chain.fav -o /tmp/chain.fvc
fav exec /tmp/chain.fvc

# 型チェックのみ（ビルドせず確認）
fav check examples/chain.fav
```

---

## 言語概要

### エントリポイント

```favnir
public fn main() -> Unit !Io {
    IO.println("Hello, Favnir!")
}
```

### 型定義

```favnir
// レコード型
type User = { name: String  email: String }

// 代数的データ型 (ADT)
type Shape = | Circle(Float) | Rect { w: Float  h: Float }
```

### レコード構築式 (v0.2.0)

```favnir
bind u <- User { name: "Alice", email: "alice@example.com" };
```

### 関数定義 (`fn`)

```favnir
fn add(a: Int, b: Int) -> Int {
    a + b
}
```

### トランスフォーム定義 (`trf`)

```favnir
trf Double: Int -> Int = |n| { n + n }
trf Inc:    Int -> Int = |n| { n + 1 }
```

### フロー定義 (`flw`)

```favnir
flw DoubleInc = Double |> Inc
```

### パイプライン式 (`|>`)

```favnir
fn process(n: Int) -> Int {
    n |> Double |> Inc
}
```

### バインド束縛 (`bind <-`)

```favnir
fn f() -> Int {
    bind x <- 10;
    bind y <- 20;
    x + y
}
```

### パターンマッチ

```favnir
match value {
    ok(v)  => v
    err(_) => 0
}
```

### エフェクト注釈

複数エフェクトはスペース区切りで並べます。

```favnir
fn create_user(name: String, email: String) -> Int !Db !Emit<UserCreated> {
    bind id <- Db.execute("INSERT INTO users (name, email) VALUES (?, ?)", name, email);
    emit User { id: id, name: name, email: email };
    id
}
```

| エフェクト | 意味 |
|-----------|------|
| `!Pure`   | 副作用なし（デフォルト）|
| `!Io`     | 標準入出力 |
| `!Db`     | SQLite データベースアクセス |
| `!Network`| HTTP ネットワーク通信 |
| `!Emit<T>`| イベント発行 |

### emit 式 (v0.2.0)

```favnir
emit UserCreated { id: 1, name: "Alice" }
```

`emit` は任意の値をイベントログに追記し、`Unit` を返します。
ログは `Emit.log()` で取得できます（`List<String>` 形式）。

## 組み込み関数

| 名前空間  | 関数 |
|-----------|------|
| `IO`      | `print`, `println` |
| `List`    | `map`, `filter`, `fold`, `length`, `is_empty`, `first`, `last` |
| `String`  | `trim`, `lower`, `upper`, `split`, `length`, `is_empty` |
| `Option`  | `some`, `none`, `map`, `unwrap_or` |
| `Result`  | `ok`, `err`, `map`, `unwrap_or` |
| `Db`      | `execute(sql, args...)`, `query(sql, args...)`, `query_one(sql, args...)` |
| `Http`    | `get(url)`, `post(url, body)` |
| `Map`     | `get(map, key)`, `set(map, key, value)`, `keys(map)`, `values(map)` |
| `Debug`   | `show(value)` |
| `Emit`    | `log()` |

### Db 組み込み詳細

```favnir
// 変更行数を返す
bind n <- Db.execute("INSERT INTO users (name) VALUES (?)", name)

// List<Map<String, String>> を返す
bind rows <- Db.query("SELECT * FROM users WHERE active = ?", true)

// Map<String, String>? を返す
bind row <- Db.query_one("SELECT * FROM users WHERE id = ?", id)
```

SQL パラメータには `?` を使い、Favnir の値（`Int`, `Float`, `String`, `Bool`, `Unit`）を対応する SQLite 型にバインドします。

## エラーコード

| コード | 内容 |
|--------|------|
| E001   | 型不一致 |
| E002   | 未定義の識別子 |
| E003   | パイプライン / flw の接続型エラー |
| E004   | エフェクト違反 |
| E005   | 引数の個数不一致 |
| E006   | パターンマッチエラー |
| E007   | `!Db` エフェクトなしで `Db.*` を使用 |
| E008   | `!Network` エフェクトなしで `Http.*` を使用 |
| E009   | `!Emit<T>` エフェクトなしで `emit` を使用 |
| E012   | 循環 import の検出 |
| E013   | モジュールまたはシンボルが見つからない |
| E014   | `private` シンボルを別ファイルから import しようとした |
| E015   | `private` シンボルを別ファイルから参照しようとした |
| E017   | 未解決の型変数（型推論で具体化できなかった） |
| E018   | 型単一化の失敗（型不一致） |
| E019   | 無限型（occurs check 失敗） |
| E020   | 未定義の cap |
| E021   | その型への cap 実装 (impl) が存在しない |
| E022   | impl のメソッドが cap 定義と合わない |
| E023   | 型パラメータの個数が合わない |
| E024   | `chain` を Result / Option 以外の戻り型の関数で使用 |
| E025   | `chain` 式の型がコンテキストと合わない |
| E026   | `yield` を `collect` ブロック外で使用 |
| E027   | パターンガード (`where`) の型が `Bool` でない |
| W001   | `namespace` 宣言がファイルパスと一致しない |

## サンプル

| ファイル | 内容 |
|----------|------|
| `examples/hello.fav`                   | 最小の Hello World |
| `examples/pipeline.fav`               | trf / flw / パイプライン |
| `examples/adt_match.fav`              | ADT とパターンマッチ |
| `examples/users.fav`                  | User CRUD + Emit (v0.2.0) |
| `examples/effect_errors.fav`          | エフェクト違反のデモ |
| `examples/multi_file/`                | マルチファイルプロジェクト (v0.3.0) |
| `examples/visibility_errors/`         | 可視性エラーのデモ (v0.3.0) |
| `examples/generics.fav`              | ジェネリック関数・型 (v0.4.0) |
| `examples/cap_sort.fav`              | Ord / Eq cap の使用 (v0.4.0) |
| `examples/cap_user.fav`              | ユーザー定義型への impl (v0.4.0) |
| `examples/chain.fav`                 | `chain` による Result / Option 伝播 (v0.5.0) |
| `examples/collect.fav`              | `collect / yield` によるリスト構築 (v0.5.0) |
| `examples/pipe_match.fav`           | `\|> match` + `where` ガードの組み合わせ (v0.5.0) |
| `examples/interface_basic.fav`      | `interface` / 手書き `impl` の基本例 (v1.1.0) |
| `examples/interface_auto.fav`       | `with` 糖衣構文と自動合成の例 (v1.1.0) |
| `examples/algebraic.fav`            | `Ring` interface を使った代数演算の例 (v1.1.0) |
| `examples/invariant_basic.fav`      | `invariant` と `TypeName.new(...)` の基本例 (v1.2.0) |
| `examples/std_states.fav`           | `std.states` の利用例 (v1.2.0) |

> **v0.6.0**: 上記の全サンプルは `fav build` → `fav exec` でも実行できます。
## v1.6.0

### FString

```favnir
bind name <- "Favnir"
bind age <- 42
$"Hello {name}! Age: {age}"
```

### Record Pattern

```favnir
match user {
    { name, age } where age >= 18 => name
    { name: n } => n
}
```

### Test / Watch

```text
fav test --filter "keyword"
fav watch --cmd check
```

## v1.8.0

### Task\<T\> parallel API

```favnir
// Task.all — collect all results
bind tasks <- collect {
    yield Task.run(|| compute(1));
    yield Task.run(|| compute(2));
}
bind results <- Task.all(tasks)

// Task.race — first result wins
bind winner <- Task.race(tasks)

// Task.timeout — Option<T> (always some in v1.8.0)
bind result <- Task.timeout(Task.run(|| 42), 1000)
```

### async fn main()

```favnir
public async fn main() -> Unit !Io {
    bind msg <- greet("world")
    IO.println(msg)
}
```

### chain + Task\<T\>

`chain x <-` now unwraps `Task<Option<T>>` and `Task<Result<T,E>>` transparently.

### Coverage: function-level report

```
fav test --coverage
fav test --coverage --coverage-report ./coverage_out
```

Per-function breakdown shows covered/total lines and `[full|partial|none]` status.
`--coverage-report <dir>` writes `<dir>/coverage.txt`.

### fav bench

```favnir
bench "fib(15)" { fib(15) }
bench "factorial(10)" { factorial(10) }
```

```
fav bench math.bench.fav --iters 500 --filter fib
```

Output: `µs/iter` per benchmark with configurable iteration count.

---

## v1.7.0

### Task\<T\> async base

```favnir
async fn fetch_greeting() -> String !Io {
    "Hello from async!"
}

public fn main() -> Unit !Io {
    bind msg <- fetch_greeting()   // msg: String (Task unwrapped)
    IO.println(msg)
}
```

### Type aliases

```favnir
type UserId   = Int
type UserName = String
type MaybeId  = Option<Int>
```

Aliases are fully compatible with their target types and resolved at
type-check time with no runtime overhead.

### Coverage

```text
fav test --coverage
```

Output:
```
coverage: src/main.fav
  lines covered: 12 / 15 (80.0%)
  uncovered:     lines 7, 9, 14
```

### Watch multi-dir

```text
fav watch --cmd check --dir src --dir tests --debounce 500
```
