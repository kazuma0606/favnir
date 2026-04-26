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

## 短いまとめ

- 積型 = 複数の値を同時に持つ
- 和型 = 複数の候補のうち一つを取る
- ADT = 積型と和型を組み合わせた型

Favnir では、`struct`, `type`, `T?`, `T!`, `match` を支える基礎になる。
