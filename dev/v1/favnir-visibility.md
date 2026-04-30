# Favnir Visibility and Construction Draft

更新日: 2026-04-26

## 結論

Favnir では:

- インスタンス化の仕組みは必要
- ただし class 的 constructor は不要
- 可視性は `private/public` の二値だけでは少し弱い

公開指定は `pub` ではなく **`public`** を採る方針にする。

## 1. コンストラクタ

### 基本方針

Favnir では、インスタンス化は必要。

ただし、OOP 的な `new` や class constructor を中核に置くのではなく、

- record literal
- variant literal
- 必要なら `fn` による smart constructor

で表現する方が自然。

### record 的な型

```fav
type User = {
    name: String
    email: String
}

bind user <- User {
    name: "A",
    email: "a@example.com"
}
```

この形を基本コンストラクタとみなす。

### ADT / variant

```fav
type Session =
    | Guest
    | Authenticated { user: User }

bind s1 <- Guest
bind s2 <- Authenticated { user: user }
```

variant 自体がコンストラクタになる。

### smart constructor

検証を伴う生成は `fn` で表現する。

```fav
fn make_user(name: String, email: String) -> User! {
    ...
}
```

### 結論

- 基本コンストラクタ = record literal / variant literal
- 検証付き生成 = `fn`
- class 的 constructor = 不要

## 2. 可視性

### 問題意識

`public` / `private` の二値だけだと、Favnir では少し粗い。

理由:

- `trf`
- `flw`
- `type`
- `rune`
- `namespace`

の層があるため、「同じまとまりの中では使いたいが外には見せたくない」ケースが自然に出る。

### 推奨段階

最初に考えるべきは次の 3 段階。

1. `private`
2. `internal`
3. `public`

### `private`

- デフォルト
- 定義された file / module 内だけで使える

### `internal`

- 同じ `rune` の中では使える
- 外部 rune からは見えない

### `public`

- 外部へ公開する

## 3. `protected` ではなく `internal`

`protected` は継承ベースの世界観を連想しやすい。

Favnir は継承中心ではないので、`protected` より `internal` の方がしっくり来る。

つまり、Favnir の可視性は:

- OOP 的な継承階層

ではなく、

- file / module
- rune
- public API

の階層で考える方が自然。

## 4. 例

```fav
internal trf ValidateUser: List<Row> -> List<User> = |rows| {
    ...
}

public flw ImportUsers =
    ParseCsv
    |> ValidateUser
    |> SaveUsers
```

この場合:

- `ValidateUser` は同じ rune 内の `flw` からは使える
- 外部には公開しない

## 5. field access

field access まで最初から細かく制御するかは別問題。

```fav
user.email
```

自分のおすすめは:

- 最初は field access は普通に許す
- 可視性制御はトップレベル定義に集中する

つまり初期段階では、

- `type`
- `fn`
- `trf`
- `flw`

の公開面を優先して制御する。

field-level visibility は後回しでよい。

## 6. `public` を採る理由

Favnir では `pub` ではなく `public` を使う方針にする。

理由:

- 略語より意味が明快
- `fn`, `trf`, `flw` は短くても、可視性は少し明示的な方が読みやすい
- API の意図を見落としにくい

例:

```fav
public fn normalize_email(value: String) -> String {
    ...
}

public trf ParseCsv: String -> List<Row> = |text| {
    ...
}
```

## 7. 初期仕様の提案

最初に入れるなら次で十分。

- `private` はデフォルト
- `internal` は同じ rune 内で可視
- `public` は外部公開

後で検討するもの:

- field-level visibility
- namespace-level visibility
- protected 的な追加階層

## 短い結論

Favnir では:

- 生成は record literal / variant literal を基本にする
- `fn` は smart constructor にも使う
- 可視性は `private / internal / public`
- `pub` ではなく `public`

この整理が一番自然。
