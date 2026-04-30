# Favnir Syntax Draft

更新日: 2026-04-26

## 目的

このメモは、Favnir の最小構文を整理するための草案。

方針:

- 型付き pipeline を中核にする
- immutable を前提にする
- effect を型で明示する
- decorator ではなく構文と型で意味を表す

## 必須構文

### `bind <-`

ローカル束縛の基本構文。

```fav
bind rows <-
    text
    |> ParseCsv
```

特徴:

- 初回束縛専用
- 再代入不可
- `=` による代入連想を避ける

pattern binding も同じ構文で扱う。

```fav
bind { name, email } <- user
bind ok(value) <- parse_user(row)
```

### `trf`

再利用可能な処理片。

```fav
trf ParseCsv: String -> List<Row> = |text| {
    ...
}
```

役割:

- pipeline の基本単位
- effect を持てる
- 型付き合成の対象

`trf` は `transform` 由来の短縮形で、`fn` と `flw` の中間にある処理片を表す。

### `flw`

複数 `trf` の合成済み定義。

```fav
flw ImportUsers =
    ParseCsv
    |> ValidateUser
    |> SaveUsers
```

役割:

- 再利用可能な処理列
- 既存 mix-in の代替候補

`flw` は `flow` の短縮形。

### `|>`

型付き合成演算子。

```fav
bind users <-
    text
    |> ParseCsv
    |> ValidateUser
```

意味:

- 左辺出力と右辺入力の接続を型で検査する
- effect も同時に追跡する

### effect 注釈

```fav
trf SaveUsers: List<User> -> List<UserId> !Db = |users| {
    ...
}
```

最小候補:

- `Pure`
- `Io`
- `Db`
- `Emit<Event>`

### `type`

ADT 定義。

```fav
type ImportResult =
    | Success { count: Int }
    | Failure { message: String }
```

### `struct`

record 定義。

```fav
struct User {
    name: String
    email: String
}
```

ジェネリクスも許可する。

```fav
struct Box<T> {
    value: T
}
```

### `T?` と `T!`

```fav
fn find_user(id: UserId) -> User?
fn parse_user(row: Row) -> User!
```

意味:

- `T?` は optional value
- `T!` は fallible value

内部モデルの例:

- `T?` = `Option<T>`
- `T!` = `Result<T, Error>` または `Fallible<T, E>`

注意:

- `User!` は失敗しうる値
- `!Db` は effect

### `match`

型に基づく分岐の中核。

```fav
match parse_user(row) {
    ok(user) => ...
    err(message) => ...
}
```

`bind` と `match` は同じ pattern 体系を共有するのが望ましい。

### `if`

軽い条件分岐。

```fav
if rows.is_empty() {
    return []
}
```

Favnir では `if` は式として扱う方針にする。

```fav
bind users <- if rows.is_empty() {
    []
} else {
    rows |> ValidateUser
}
```

### `fn`

通常の関数定義。

```fav
fn normalize_email(value: String) -> String {
    value.trim().lower()
}
```

## 関数まわり

### 関数は第一級値

Favnir では関数を第一級値として扱う。

```fav
fn normalize_email(value: String) -> String {
    value.trim().lower()
}

bind f <- normalize_email
bind g <- |value| value.trim().lower()
```

つまり、名前付き関数も:

- `bind` で束縛できる
- 引数として渡せる
- 返り値として返せる

### 関数定義 / 関数値 / 適用結果

次の 3 つは区別する。

1. 関数定義
2. 関数値
3. 関数適用の結果

```fav
fn add(x: Int) -> Int {
    x + 1
}

bind f <- add
bind y <- add(1)
```

このとき:

- `fn add ...` は関数定義
- `add` と `f` は関数値
- `add(1)` と `y` は結果値

### 関数型

`Fn<T, U>` ではなく、矢印型を中心にする。

```fav
String -> String
Row -> User!
T -> U
```

高階関数もこの表記で受ける。

```fav
fn map<T, U>(items: List<T>, f: T -> U) -> List<U> {
    ...
}
```

### クロージャ

```fav
|row| parse_user(row)
|user| user.email
|x, y| x + y
```

方針:

- lexical scope から自動 capture
- capture は immutable のみ
- mutable capture はなし

### `fn` と `trf` の関係

- `fn` は通常の関数
- `trf` は pipeline に載る名前付き処理片
- `trf` は effect を持てる

概念的には:

- `fn`: `A -> B`
- `trf`: `A -> B !Fx`

### カリー化

初期仕様では自動カリー化は入れない。

方針:

- 最初はカリー化なし
- クロージャはあり
- 必要なら後で明示的部分適用を検討

## 式指向

Favnir は式指向の言語として整理する。

方針:

- `if` は式
- `match` は式
- block は式
- 関数適用は式
- block の最後の式が値になる

例:

```fav
bind result <- {
    bind rows <- text |> ParseCsv
    if rows.is_empty() {
        []
    } else {
        rows |> ValidateUser
    }
}
```

一方で `bind` 自体は式ではなく、束縛構文として扱う。

## block

block は最後の式を返す。

```fav
{
    bind x <- 1
    x + 1
}
```

この block 全体の値は `2`。

## optional / fallible / effect の分離

Favnir ではこの 3 つを分ける。

- `User?` = absence
- `User!` = failure
- `!Db`, `!Io`, `!Emit<E>` = effect

## スコープ

Favnir は lexical scope を基本にする。

最初に必要な範囲:

- ファイルスコープ
- `fn` / `trf` スコープ
- ブロックスコープ
- `match` アームごとのスコープ

`namespace` はトップレベル名の整理に使い、lexical scope とは別に扱う。

## あってよい構文

### `try / catch`

導入してもよいが、中心には置かない。

```fav
try {
    read_file(path)
} catch err {
    ...
}
```

原則:

- 通常の失敗は `T!` や ADT + `match`
- `try/catch` は effect 境界寄り

### `event`

残す価値がある。

```fav
event UserImported {
    id: String
}
```

### `emit`

残す価値がある。

```fav
emit UserImported { id: user.id }
```

ただし意味は effect として型に表すべき。

### `job`

将来的には入れてよいが、最初のコアにはしない。

- `flw` を外部実行単位に載せるための上位構文
- アプリケーション実行モデル寄り

### `test`

初期仕様では、次のようなテスト構文で十分。

```fav
test "parse user" {
    assert(true)
}
```

最小セット:

- `test "..." { ... }`
- `assert(cond)`
- `fail(message)`

`expect ...` は後で sugar として足せばよい。

### `IO`

標準入出力は `IO` namespace にまとめる。

```fav
IO.print("hello")
IO.println("hello")
```

方針:

- `print` / `println` は pure ではない
- `IO` 操作は `!Io` effect に属する
- built-in を散らさず、namespace で整理する

## 入れない構文

### decorator / annotation 中心の構文

最初は入れない。

- `@service`
- `@repository`
- `@timed`
- `@validate`
- `@on(Event)`

### `let`

使わない。`bind <-` に統一する。

### mutable / `mut`

使わない。

### trait 中心の抽象化

使わない。責務は generics / ADT / effect / `flw` に分離する。

### `Fn<T, U>` 中心の表記

使わない。`T -> U` を使う。

### ownership / lifetime の表面露出

使わない。runtime / compiler 側で吸収する。

## 構文の優先順位

最初に固めるべきもの:

1. `bind <-`
2. `trf`
3. `flw`
4. `|>`
5. effect 注釈
6. `type` / ADT
7. `T?` / `T!`
8. ジェネリクス
9. `match`
10. `if`

その後に検討するもの:

- `event`
- `emit`
- `job`
- `try/catch`

## 仮の結論

Favnir は、decorator で意味を増やす言語ではなく、

- `bind <-`
- `trf`
- `flw`
- effect
- ADT
- `T?` / `T!`
- `match`

を正面に置く言語として設計するのがよい。
