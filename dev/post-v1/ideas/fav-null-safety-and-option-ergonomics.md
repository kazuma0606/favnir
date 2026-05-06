# Favnir Null Safety と Option/Result エルゴノミクス

日付: 2026-05-01

## 基本方針: null を持たない

Favnir は Rust と同じく、**null を言語に持ち込まない**。

「値がないかもしれない」は `Option<T>` = `T?` で表現する。
「失敗するかもしれない」は `Result<T, E>` = `T!` で表現する。

null 参照例外は存在しない。コンパイラが `T?` / `T!` の未処理を検出する。

---

## フィールド宣言

### 基本形

```fav
type User {
    name:  String
    email: String
    age:   Int?      -- Option<Int>
    bio:   String?   -- Option<String>
}
```

### 糖衣構文（`member?` 記法）

フィールド名側に `?` を付ける TypeScript 風の記法も使える。

```fav
type User {
    name:   String
    email:  String
    age?:   Int      -- age: Int? と同じ意味
    bio?:   String   -- bio: String? と同じ意味
}
```

`age?: Int` と `age: Int?` は完全に同じ型。
「このフィールドは存在しないかもしれない」という意図がフィールド名側に出るため、
フィールドが多い型では読みやすくなることがある。

---

## 推奨パターン

### 1. `match`（基本・推奨）

明示的で最も Favnir らしい書き方。

```fav
match user.age {
    some(age) => show_age(age)
    none      => show_default()
}

match fetch_user(id) {
    ok(user)    => process(user)
    err(reason) => log_error(reason)
}
```

`match` は exhaustive（全ケース網羅）であることをコンパイラが保証する。

---

### 2. pipeline + combinator（変換チェーン）

`Option` / `Result` の変換には combinator を pipeline で繋ぐ。

```fav
-- Option の変換
bind display <-
    user.age
    |> Option.map(|age| age * 2)
    |> Option.unwrap_or(0)

-- Result の変換
bind normalized <-
    fetch_user(id)
    |> Result.map(normalize_user)
    |> Result.map_err(|e| AppError.from(e))
```

主要な combinator:

| combinator | 対象 | 意味 |
|---|---|---|
| `Option.map(f)` | `T?` | some のとき f を適用 |
| `Option.and_then(f)` | `T?` | some のとき f を適用（f が T? を返す） |
| `Option.unwrap_or(default)` | `T?` | some なら値、none ならデフォルト |
| `Option.or_else(f)` | `T?` | none のとき f を試みる |
| `Result.map(f)` | `T!` | ok のとき f を適用 |
| `Result.and_then(f)` | `T!` | ok のとき f を適用（f が T! を返す） |
| `Result.map_err(f)` | `T!` | err のとき err を変換 |

---

### 3. `chain`（伝播）

関数全体が `T?` または `T!` を返す文脈では `chain` で伝播させる。
どちらも同じ構文で扱える。

```fav
-- Option の伝播
fn find_display_name(id: UserId) -> String? {
    bind user  <- find_user(id)       -- User?   none なら即 none を返す
    chain age  <- user.age            -- Int?    none なら即 none を返す
    chain label <- format_age(age)    -- String? none なら即 none を返す
    some(label)
}

-- Result の伝播
fn import_user(row: String) -> UserId! {
    bind user <- parse_user(row)      -- User!   err なら即 err を返す
    chain user <- validate_user(user) -- User!   err なら即 err を返す
    chain id   <- save_user(user)     -- UserId! err なら即 err を返す
    ok(id)
}
```

---

## 採用・不採用

### 採用

| 機能 | 構文 | 理由 |
|---|---|---|
| オプショナルフィールド | `age?: Int` | `age: Int?` の糖衣、意味は同じ |
| match | `match x { some(v) => ... none => ... }` | 明示的・exhaustive |
| combinator pipeline | `Option.map / and_then / unwrap_or` | 変換チェーンに適切 |
| chain 伝播 | `chain x <- expr` | Option/Result 両方に一貫した伝播 |

### 不採用

| 機能 | 理由 |
|---|---|
| `?.` オプショナルチェーン | null 的な思考を持ち込む。`Option.map` で代替できる |
| `if some(x) <- expr` | `if` は Bool 式を受け取るものとして保つ。Option の分岐は `match` |
| null / nil / undefined | 存在しない |

### 保留

| 機能 | 構文 | 状況 |
|---|---|---|
| デフォルト値演算子 | `expr ?? default` | `Option.unwrap_or` の糖衣。なくても困らない |

---

## `if` と `match` の役割分離

```fav
-- if は Bool 式のみ受け取る
if user.active {
    process(user)
}

-- Option/Result の分岐は match
match user.age {
    some(age) => show_age(age)
    none      => show_default()
}
```

`if` で `Option` を扱う記法（`if some(x) <- expr` など）は採用しない。
`if` と `match` の責務を明確に分離することで、コードの読み方が一貫する。

---

## `T?` / `T!` の内部表現

```
T?  =  Option<T>  =  some(T) | none
T!  =  Result<T>  =  ok(T)   | err(Error)
```

表面構文は `T?` / `T!` で軽く書けるが、内部は ADT として一貫して扱われる。
`match` のパターンは `some` / `none` / `ok` / `err` のバリアント名で書く。

---

## 典型的なコードパターン

### 単純な値取り出し

```fav
-- match（推奨）
match user.age {
    some(age) => IO.println_int(age)
    none      => IO.println("不明")
}

-- combinator（変換が必要な場合）
bind display <- user.age |> Option.map(Int.show) |> Option.unwrap_or("不明")
```

### 複数の Option を組み合わせる

```fav
-- and_then で連鎖
bind result <-
    find_user(id)
    |> Option.and_then(|user| user.profile)
    |> Option.and_then(|profile| profile.avatar_url)
    |> Option.unwrap_or("default.png")
```

### エラーを扱うパイプライン

```fav
seq ImportUser =
    ParseRow       -- String -> User!
    |> ValidateUser  -- User -> User!
    |> EnrichUser    -- User -> User! !Network
    |> SaveUser      -- User -> UserId !Db

-- 各 stage は T! を返し、パイプラインが自動的に err を伝播する
```

### テストでの Option/Result

```fav
test "find_user returns some for existing id" {
    bind result <- find_user(UserId(1))
    match result {
        some(user) => assert_eq(user.name, "Alice")
        none       => fail("user should exist")
    }
}

test "import fails for invalid row" {
    bind result <- ImportUser("invalid,data")
    match result {
        ok(_)  => fail("should have failed")
        err(e) => assert(String.contains(e.message, "parse"))
    }
}
```
