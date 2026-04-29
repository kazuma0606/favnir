# Favnir v0.2.0 仕様書

更新日: 2026-04-27

## 概要

v0.2.0 は effect システムを実運用レベルに引き上げるバージョン。
`Db` (SQLite)・`Emit<E>` (インメモリイベント)・`Network` (HTTP) の 3 effect を追加し、
型チェックと実行の両方で動くようにする。

ユーザーは SQL を文字列として埋め込んで DB を操作できる（SQLx スタイル）。
ORM・マイグレーション管理は対象外。

---

## スコープ

### v0.2.0 で追加するもの

- `Db` effect — SQLite への SQL 埋め込みアクセス (`Db.query`, `Db.query_one`, `Db.execute`)
- `Emit<Event>` effect — インメモリイベントバス (`emit expr` 構文)
- `Network` effect — HTTP GET / POST の最小対応 (`Http.get`, `Http.post`)
- effect を複数同時に注釈できる多重 effect 構文 (`!Db !Emit<UserCreated>`)
- `Emit<A> + Emit<B> = Emit<A | B>` の合成を型チェックに組み込む
- レコード構築式 (`TypeName { field: expr, ... }`)
- `fav run --db <url>` CLI フラグ (SQLite 接続文字列)
- `fav explain <file>` コマンド (各 `trf` / `flw` の effect を表示)
- エラーコードの拡張 (E007〜)

### v0.2.0 でも含まないもの

- ユーザー定義ジェネリクス (`Db.query<T>` の型付きマッピングは v0.4.0)
- `chain` 束縛
- `rune` / `namespace` / `use`
- `cap`
- ORM / マイグレーション管理
- `fav fmt` / `fav test`
- bytecode / VM
- WASM

---

## 文法変更点

### effect 注釈の拡張

v0.1.0 では effect は 1 つのみ (`!Io`)。
v0.2.0 からは複数の effect を空白区切りで並べられる。

```
effect_ann  = ("!" effect_term)+
effect_term = "Pure"
            | "Io"
            | "Db"
            | "Network"
            | "Emit" "<" IDENT ">"
```

例:

```fav
trf SaveAndNotify: UserInput -> Int !Db !Emit<UserCreated> = |input| { ... }

public fn main() -> Unit !Io !Db { ... }
```

同じ `Emit<T>` が複数の `trf` に現れた場合は `flw` レベルで合成される。

```
Emit<A> + Emit<B> = Emit<A | B>
```

### レコード構築式（新規）

v0.1.0 にはレコード値を直接構築する構文がなかった。v0.2.0 で追加する。

```
record_expr = IDENT_UPPER "{" field_init ("," field_init)* ","? "}"
field_init  = IDENT ":" expr
            | IDENT           // shorthand: { name } = { name: name }
```

`IDENT_UPPER` は大文字で始まる識別子。型名と一致する必要がある（実行時チェック）。

例:

```fav
bind user <- User { name: "Alice", email: "alice@example.com" }
emit UserCreated { user_id: id, name: user.name }
```

### `emit` 式（新規）

`emit` はブロック内で使える式。`Unit` を返す。

```
emit_expr = "emit" expr
```

ブロック内で中間文として使う場合は `;` が必要:

```fav
{
    bind id <- CreateUser(input)
    emit UserCreated { user_id: id, name: input.name };
    id
}
```

最後の式として使う場合は `;` 不要:

```fav
{
    bind id <- CreateUser(input)
    emit UserCreated { user_id: id, name: input.name }
}
```

### ブロック文法の更新

```
stmt = bind_stmt
     | emit_expr ";"
     | expr ";"
```

末尾式として `emit_expr` も使える:

```
block = "{" stmt* (emit_expr | expr) "}"
```

---

## effect システム

### effect の種類 (v0.2.0)

| effect | 意味 | 実行対応 |
|---|---|---|
| `Pure` | 副作用なし | v0.1.0 |
| `Io` | 標準入出力 | v0.1.0 |
| `Db` | データベースアクセス | v0.2.0 |
| `Network` | HTTP 通信 | v0.2.0 |
| `Emit<E>` | イベント発行 | v0.2.0 |

### 合成規則

一般則: **異種の effect は集合和として扱う。同種の effect は種類ごとのルールで畳み込む。**

```
Pure + X                = X             // Pure は identity
Io   + Io               = Io            // 同種は冪等
Db   + Db               = Db
Network + Network       = Network
Emit<A> + Emit<B>       = Emit<A | B>   // Emit のみ型引数が合成される
Emit<A | B> + Emit<B>   = Emit<A | B>   // 重複を除く
X + Y (異種)            = {X, Y}        // Db + Network = {Db, Network}
```

`flw` の effect は各ステップの effect の和になる:

```fav
trf A: Int -> Int !Db       = ...
trf B: Int -> Int !Emit<E>  = ...
flw AB = A |> B
// AB : Trf<Int, Int, Db + Emit<E>>
```

### 型チェックでの扱い

- effect のない `trf` は `Pure` として扱う
- `flw` の effect は各ステップの effect の和として推論する
- `Db` 系の関数呼び出し (`Db.query` など) は `!Db` を持つ `trf` / `fn` の中でのみ使える
  - v0.2.0 では警告にとどめ、エラーにはしない（後方互換を考慮）
- `emit expr` は `Emit<T>` effect として記録される
  - `T` は `expr` の型から推論する

---

## Db 組み込み関数

### 設計方針

- SQL は文字列として埋め込む (SQLx スタイル)
- パラメータは `?` プレースホルダで位置指定
- ORM・スキーマ定義・マイグレーションは提供しない
- 返り値の型は v0.2.0 では `Map<String, String>` (全カラム値を文字列として取得)
  - v0.4.0 でジェネリクスが追加された時点で `Db.query<T>` の型付きマッピングを追加する

### 組み込み関数

```
// 複数行を返す SELECT
Db.query(sql: String, args...) -> List<Map<String, String>>

// 0 または 1 行を返す SELECT
Db.query_one(sql: String, args...) -> Map<String, String>?

// INSERT / UPDATE / DELETE / CREATE TABLE 等
// INSERT の場合: 最後に挿入した行の rowid を返す
// UPDATE / DELETE の場合: 変更された行数を返す
Db.execute(sql: String, args...) -> Int
```

`args` は可変長。各引数の値は SQLite のパラメータとしてバインドされる。
型変換は次のルールで行う:

| Favnir 型 | SQLite 型 |
|---|---|
| `Int` | INTEGER |
| `Float` | REAL |
| `String` | TEXT |
| `Bool` | INTEGER (0/1) |
| `Unit` | NULL |

取得した列の値はすべて `String` に変換して `Map<String, String>` に格納する。

### 接続設定

```
fav run --db <connection-string> <file.fav>
```

| 接続文字列 | 意味 |
|---|---|
| `sqlite://./app.db` | ファイル DB |
| `sqlite://:memory:` | インメモリ DB |

`--db` を省略した場合はインメモリ DB を使う。

---

## Emit 組み込み

`emit expr` でイベントをインメモリイベントログに追加する。

```
Emit.log() -> List<String>
```

v0.2.0 ではイベントは文字列表現でログに記録される。
購読・フィルタは v0.5.0 以降。

---

## Network 組み込み関数

HTTP クライアントの最小セット。同期実行。

```
Http.get(url: String) -> String!   // !Network
Http.post(url: String, body: String) -> String!  // !Network
```

`String!` は `Result<String, Error>` の sugar。
ネットワークエラー・タイムアウトは `err(...)` として返す。

---

## CLI 変更点

### `fav run` の拡張

```
fav run [--db <url>] <file.fav>
```

### `fav explain` (新規)

```
fav explain <file.fav>
```

各トップレベル定義の型と effect を表示する:

```
fn main                : Unit                  !Io !Db
trf CreateUser         : UserInput -> Int       !Db
trf CreateAndNotify    : UserInput -> Int       !Db !Emit<UserCreated>
flw Onboard            : UserInput -> Int       !Db !Emit<UserCreated>
```

---

## 型システムへの影響

### Option / Result の組み込みバリアント

`Option<T>` と `Result<T, E>` は組み込みの ADT として扱う。
パターンマッチで使う `some` / `none` / `ok` / `err` は **予約バリアント名** であり、
ユーザーが定義する型の variant 名として使うことはできない。

| 型 | バリアント | payload |
|---|---|---|
| `Option<T>` = `T?` | `some(value)` | `T` |
| `Option<T>` = `T?` | `none` | なし |
| `Result<T,E>` = `T!` | `ok(value)` | `T` |
| `Result<T,E>` = `T!` | `err(reason)` | `E` (= `Error` = `String`) |

これらは sugar ではなく Favnir ランタイムが直接認識する特殊バリアント。
`match` のパターンで小文字スタート (`some`, `none`, `ok`, `err`) を使うと
組み込みバリアントとして扱われる。

```fav
match opt {
    some(v) => v      // Option の some バリアント
    none    => 0      // Option の none バリアント (payload なし)
}

match res {
    ok(v)     => v
    err(msg)  => 0
}
```

### `Debug.show` (新規)

任意の値を人間が読める文字列に変換するデバッグ用関数。
`IO.println` は `String` 専用なので、非 String 値を表示したい場合はこれを通す。

```
Debug.show(value: T) -> String
```

例:

```fav
bind row <- Db.query_one("SELECT id, name FROM users WHERE id = ?", 1)
match row {
    some(r) => IO.println(Debug.show(r))
    none    => IO.println("not found")
}
```

### `Map<String, String>` の追加操作

DB からの行アクセスで使う。

```
Map.get(m: Map<K, V>, key: K) -> V?
Map.set(m: Map<K, V>, key: K, value: V) -> Map<K, V>
Map.keys(m: Map<K, V>) -> List<K>
Map.values(m: Map<K, V>) -> List<V>
```

### エラーコードの追加

| コード | 種類 |
|---|---|
| E007 | `Db` 系関数を `!Db` のない `trf` / `fn` 内で使った |
| E008 | `Http` 系関数を `!Network` のない `trf` / `fn` 内で使った |
| E009 | `emit` を `!Emit<T>` のない `trf` / `fn` 内で使った |
| E010 | レコード構築式: フィールド名不一致 |
| E011 | `Db` 接続エラー (実行時) |

---

## 実行モデルの変更

### Db 実行

- `rusqlite` クレートで SQLite に接続
- 接続はプロセス起動時に 1 つ確立し、インタープリタ全体で共有
- トランザクションは v0.2.0 では自動コミット (各 `Db.execute` が即座にコミット)

### Emit 実行

- インメモリのベクタにイベントを追加
- `Emit.log()` でその内容を文字列リストとして取得できる
- プロセス終了時に破棄される

### Network 実行

- `ureq` クレートで同期 HTTP を実行
- タイムアウト: デフォルト 30 秒

---

## CRUD 例

```fav
// users.fav - User CRUD example with SQLite

type UserInput = {
    name:  String
    email: String
}

type UserCreated = {
    user_id: Int
    name:    String
}

// テーブル初期化
trf InitDb: Unit -> Unit !Db = |_| {
    Db.execute("
        CREATE TABLE IF NOT EXISTS users (
            id    INTEGER PRIMARY KEY AUTOINCREMENT,
            name  TEXT NOT NULL,
            email TEXT NOT NULL UNIQUE
        )
    ")
}

// Create
trf CreateUser: UserInput -> Int !Db !Emit<UserCreated> = |input| {
    bind id <- Db.execute(
        "INSERT INTO users (name, email) VALUES (?, ?)",
        input.name,
        input.email
    )
    emit UserCreated { user_id: id, name: input.name };
    id
}

// Read one
trf FindUser: Int -> Map<String, String>? !Db = |id| {
    Db.query_one("SELECT id, name, email FROM users WHERE id = ?", id)
}

// Read all
trf ListUsers: Unit -> List<Map<String, String>> !Db = |_| {
    Db.query("SELECT id, name, email FROM users")
}

// Update
trf UpdateEmail: UserInput -> Int !Db = |input| {
    Db.execute(
        "UPDATE users SET email = ? WHERE name = ?",
        input.email,
        input.name
    )
}

// Delete
trf DeleteUser: Int -> Int !Db = |id| {
    Db.execute("DELETE FROM users WHERE id = ?", id)
}

public fn main() -> Unit !Io !Db {
    bind _ <- InitDb(())
    bind id <- CreateUser(UserInput { name: "Alice", email: "alice@example.com" })
    IO.println("created");
    bind row <- FindUser(id)
    match row {
        some(r) => IO.println(Debug.show(r))   // Map<String, String> -> String
        none    => IO.println("not found")
    }
}
```

---

## 完了条件

- `fav run --db sqlite://:memory: examples/users.fav` が動く
- CRUD (Create / Read / ListAll / Update / Delete) が SQLite で動作する
- `trf CreateUser: UserInput -> Int !Db !Emit<UserCreated>` が型チェックを通る
- `emit UserCreated { ... }` が動き、`Emit<UserCreated>` が型に現れる
- `Http.get(url)` が `String!` を返して動く
- `fav explain` で各 `trf` / `flw` の effect が表示される
