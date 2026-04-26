# Favnir Generics Draft

更新日: 2026-04-26

## 結論

Favnir ではジェネリクスを入れるべき。

理由:

- 型付き pipeline を抽象化するため
- ADT を実用的に使うため
- trait を中心にしない代わりに、型引数ベースの抽象化が必要になるため

ただし、最初から複雑な constraint system までは入れない。

最初は次の方針にする。

- 型引数は入れる
- 制約句は入れない
- trait bound は入れない
- HKT は入れない

## 最小仕様

最初にサポートするのは次の 4 つ。

### 1. generic struct

```fav
struct Box<T> {
    value: T
}
```

### 2. generic ADT

```fav
type Result<T, E> =
    | ok(T)
    | err(E)

type Option<T> =
    | some(T)
    | none
```

### 3. generic function

```fav
fn identity<T>(value: T) -> T {
    value
}
```

### 4. generic stage

```fav
stage Map<T, U>: List<T> -> List<U> = |items| {
    ...
}
```

## 重要な使い道

### ADT

以下はジェネリクス前提でかなり重要。

- `Option<T>`
- `Result<T, E>`
- `Stream<T>`
- `Decoder<T>`

### pipeline 抽象化

Favnir の本体は `stage / flow` なので、再利用可能な変換を抽象化するにはジェネリクスが必須。

```fav
stage DecodeJson<T>: String -> T! = |text| {
    ...
}
```

### collection 操作

`map`, `filter`, `fold`, `group` のような処理はジェネリクス前提。

```fav
fn map<T, U>(items: List<T>, f: Fn<T, U>) -> List<U> {
    ...
}
```

## `T?` / `T!` との関係

`T?` と `T!` は表面構文だが、内部的には generic ADT に落とす。

- `T?` = `Option<T>`
- `T!` = `Result<T, Error>` または `Fallible<T, E>`

これにより、見た目は軽く、内部は厳密に扱える。

## 最初は入れないもの

次は初期段階では入れない方がよい。

- `where`
- trait bound
- HKT
- associated type
- implicit resolution
- specialization

理由:

- checker が急に重くなる
- trait 的な複雑さを持ち込みやすい
- Favnir の初期価値はそこではない

## 将来の制約機構

もし後で必要なら、constraint は trait としてではなく、compile-time rule として別建てにした方がよい。

イメージ:

```fav
fn sort<T>(items: List<T>) -> List<T>
where T: Ord
```

ただし、これは初期仕様には入れない。

Favnir では次の 2 つを分けて考えるべき。

- compile-time constraint
- runtime capability

`Db`, `Io`, `Emit<Event>` は runtime capability / effect の話であり、generic constraint とは別物。

## 実装順

おすすめの導入順:

1. generic `struct`
2. generic `type`
3. generic `fn`
4. generic `stage`
5. `T?` / `T!` を generic ADT に統一
6. 必要なら後で constraint 句を検討

## 仮の結論

Favnir では、ジェネリクスはかなり重要。

ただし、最初に必要なのは「型引数による抽象化」であって、「複雑な制約系」ではない。

つまり最初は:

- generic struct
- generic ADT
- generic fn
- generic stage

これだけで十分強い。
