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
fav run [--db <path>] [file]   — 型チェック後にプログラムを実行
fav check [file]               — 型チェックのみ（実行しない）
fav explain [file]             — 全トップレベル項目の型・エフェクトを表示
fav help                       — ヘルプを表示
```

`file` を省略するとカレントディレクトリから `fav.toml` を探して**プロジェクトモード**で動作します。

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
