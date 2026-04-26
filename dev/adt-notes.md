# ADT Notes

更新日: 2026-04-26

## 目的

このメモは、Favnir で使う ADT の意味を短く整理するための基礎ノート。

## 結論

ADT は **Algebraic Data Type** の略。

ここでいう「代数的」は、群論そのものではなく、  
**型を和と積のように組み立てられる**という意味。

## 1. 積型

複数の値を同時に持つ型。

例:

```fav
struct User {
    name: String
    email: String
}
```

これは:

- `name`
- `email`

を両方持つので、イメージとしては

```text
String × String
```

のような product type。

他の例:

- tuple
- record
- `struct`

## 2. 和型

複数の候補のうち、どれか一つを取る型。

例:

```fav
type ImportResult =
    | Success { count: Int }
    | Failure { message: String }
```

これは:

- `Success`
- `Failure`

のどちらか一つなので、イメージとしては

```text
Int + String
```

のような sum type。

他の例:

- enum
- variant
- tagged union

## 3. ADT

ADT は、積型と和型を組み合わせた型。

つまり:

- tuple / record / `struct` のような積
- variant / enum のような和

を組み合わせてデータを表現する。

## 4. 代表例

### `Option<T>`

```text
Option<T> = Some(T) | None
```

これは和型。

- `Some(T)`
- `None`

のどちらか。

### `Result<T, E>`

```text
Result<T, E> = Ok(T) | Err(E)
```

これも和型。

- `Ok(T)`
- `Err(E)`

のどちらか。

### record を含む和型

```fav
type ImportResult =
    | Success { count: Int }
    | Partial { count: Int, invalid: Int }
    | Failure { message: String }
```

ここでは:

- 各 variant の中に record 的な積型が入っている
- 全体としては和型

なので、積と和の組み合わせになっている。

## 5. Favnir でなぜ重要か

ADT は Favnir でかなり重要。

理由:

- `T?` を `Option<T>` として表現できる
- `T!` を `Result<T, E>` 的に表現できる
- `match` の中心になる
- pattern binding の対象になる
- parser / checker の内部表現にも向いている
- domain result を安全に表現できる

## 6. `match` との関係

ADT は `match` とセットで力を発揮する。

```fav
match parse_user(row) {
    ok(user) => ...
    err(message) => ...
}
```

和型の各ケースを明示的に扱えるので、安全性が高い。

## 7. 関数型言語での位置づけ

ADT は関数型言語ではかなり中核的な概念。

特に:

- immutable data
- pattern match
- pure function

と組み合わせると非常に強い。

## 8. `type` を統一入口にする考え方

Favnir では、将来的に `type` をユーザー定義型の統一入口として前面に出す考え方が自然。

例:

```fav
type User = {
    name: String
    email: String
}

type Session =
    | Guest
    | Authenticated { user: User }
```

このときユーザーは両方とも `type` で定義する。  
一方で compiler / checker は内部的に:

- record 的な型なのか
- sum type なのか
- generic な合成型なのか

を ADT として解析できる。

つまり、表面構文は統一しつつ、中身の種類は内部意味論で判定する。

## 9. tooling での活用

この設計は IDE や補完とかなり相性が良い。

たとえば hover で:

```text
User
record type
fields:
- name: String
- email: String
```

```text
Session
sum type
variants:
- Guest
- Authenticated { user: User }
```

のように表示できる。

## 10. 命名支援

型の種類が分かるなら、命名規約も支援できる。

たとえば:

- record type には entity / noun 的な名前を推奨
- sum type には state / result / mode 的な名前を推奨

hover や lint で次のような支援ができる。

```text
Type `User` looks like a sum type.
Consider a state/result-style name for readability.
```

あるいは逆に:

```text
Type `SessionState` looks like a record type.
Consider a noun/entity-style name if this is not a state union.
```

## 11. 意義

この方針の利点:

- `struct` / `enum` / `union` を表面構文で増やしすぎずに済む
- それでも tooling 上では型の種類を明確に見せられる
- 命名や設計の一貫性を静的に支援できる
- AI 補完にも metadata として活用しやすい

## 短いまとめ

- 積型 = 複数の値を同時に持つ
- 和型 = 複数の候補のうち一つを取る
- ADT = 積型と和型を組み合わせた型

Favnir では、`type`, `T?`, `T!`, `match` を支える基礎になり、  
将来的には tooling 上で型の種類を可視化する基盤にもなる。
