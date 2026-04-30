# Favnir Core Spec

更新日: 2026-04-26

## 概要

Favnir は、型付き pipeline と effect を中核にした関数型データ言語。

コアは次の 6 つ。

- `type`
- `bind`
- `trf`
- `flw`
- `rune`
- effect

## 型

### 基本

- `Bool`
- `Int`
- `Float`
- `String`
- `Unit`
- `List<T>`
- `Map<K, V>`

### 特殊

- `T?` = optional
- `T!` = fallible

初期は:

- `T? = Option<T>`
- `T! = Result<T, Error>`

### ユーザー定義型

`type` を主役にする。

```fav
type User = {
    name: String
    email: String
}

type Session =
    | Guest
    | Authenticated { user: User }
```

record も sum も `type` で定義する。

## ADT / pattern

- variant は `Name { ... }` 形式で統一
- nullary variant は `Guest`
- `bind` と `match` は同じ pattern を共有する

```fav
bind Authenticated { user } <- session

match session {
    Guest => ...
    Authenticated { user } => ...
}
```

## 束縛

### `bind <-`

- 初回束縛専用
- 再代入なし
- pattern binding も `bind <-`

```fav
bind user <- parse(row)
bind { name, email } <- user
```

### `chain`

- 既存束縛に段階的に処理を積む
- 再代入ではなく fresh binding への sugar
- failure は伝播
- effect は蓄積

```fav
bind user <- row
chain user <- parse_user
chain user <- normalize_user
chain user <- save_user
```

## 関数と処理片

### `fn`

通常関数。

```fav
fn normalize_email(value: String) -> String {
    value.trim().lower()
}
```

### `trf`

effect を持てる名前付き処理片。

```fav
trf ParseCsv: String -> List<Row> = |text| {
    ...
}
```

### `flw`

`trf` の合成済み資産。

```fav
flw ImportUsers =
    ParseCsv
    |> ValidateUser
    |> SaveUsers
```

### `rune`

公開・配布・再利用単位。

## 関数型

- 関数は第一級値
- 関数型は `A -> B`
- `trf` は概念的に `A -> B !Fx`
- `Fn<T, U>` は使わない

```fav
fn map<T, U>(items: List<T>, f: T -> U) -> List<U> {
    ...
}
```

## 式指向

- `if` は式
- `match` は式
- block は式
- block の最後の式が値
- `bind` は式ではなく束縛構文

## effect

最小集合:

- `Pure`
- `Io`
- `Db`
- `Network`
- `Emit<Event>`
- `Trace`

合成は集合の和で扱う。

例:

- `Pure + Db = Db`
- `Db + Io = Db + Io`
- `Emit<A> + Emit<B> = Emit<A | B>`

`Emit` は型パラメータを union で合成する。
複数の event を発火する `flw` の effect は `Emit<A | B>` として一つに畳まれる。

## `cap` (capability)

`cap` は compile-time の明示的な能力記述。effect とは独立した概念。

- effect = runtime の副作用 (`Db`, `Io`, `Emit<E>` など)
- cap = 型に対して要求される操作 (`Ord`, `Eq`, `Show` など)

### 定義

```fav
cap Ord<T> {
    compare: T -> T -> Int
}
```

### 使用

```fav
fn sort<T>(items: List<T>, ord: Ord<T>) -> List<T> {
    ...
}
```

### 使用側

```fav
bind sorted <- sort(users, User.ord)
```

capability は値として渡す。暗黙解決はしない。

これにより:

- generic 制約を trait bound なしに表現できる
- 呼び出し側が何を渡しているかが明示される
- AI が安全に推論しやすい
- テストで差し替えが可能

`Ord`, `Eq`, `Hash`, `Show` のような標準 capability は将来的に標準ライブラリで提供する。

## 非同期

- `async fn`
- `async trf`
- `await`
- 内部型は `Future<T>`

`await?` は後回し。

## スコープ

- lexical scope を基本
- file/module scope
- `namespace` はトップレベル名整理
- `rune` は公開単位

## 可視性

- `private` がデフォルト
- `internal`
- `public`

`pub` ではなく `public` を使う。

## テスト

最小仕様:

- `test "..." { ... }`
- `assert(cond)`
- `fail(msg)`

実行は `fav test`。

初期から意識するオプション:

- `--jobs`
- `--max-memory`
- `--filter`
- `--fail-fast`
- `--trace`
- `--shard`

## IO

標準入出力は `IO` namespace にまとめる。

```fav
IO.print("hello")
IO.println("hello")
```

`IO` は `!Io` effect に属する。

## module / package

- file-based module
- `namespace`
- `use`
- `rune`
- root `fav.toml`
- workspace 対応

## CLI

初期コア:

- `fav run`
- `fav build`
- `fav exec`
- `fav check`
- `fav test`
- `fav fmt`
- `fav lint`
- `fav explain`

## 実行モデル

- `run` = interpreter
- `build` = portable artifact
- `exec` = artifact 実行

最初の本命:

- typed IR
- bytecode
- tiny runtime

ネイティブバイナリは第一目標にしない。

## リポジトリ構造

重要な分離:

- `language/`
- `selfhost/`
- `runes/`
- `apps/`
- `tests/`
- `docs/`
- root `fav.toml`

## 後回し

- trait / constraint system
- do 記法
- 自動カリー化
- coroutine
- deploy
- field-level visibility
- macro

## 一言

Favnir は、

> `type + bind + trf + flw + rune + effect`

をコアにして、説明可能で軽い実行系を持つ関数型データ言語として進める。
