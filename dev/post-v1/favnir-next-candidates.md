# Favnir Next Candidates

更新日: 2026-04-26

## 目的

このメモは、今あるアイデアの中から「次に詰める価値が高いもの」だけを絞って整理するための一覧。

現時点では次の 5 件が有力。

1. ADT を `enum` 相当としてどう見せるか
2. `inspect`
3. `pipe match`
4. `collect / yield`
5. pattern guard としての `where`

## 1. ADT を `enum` 相当としてどう見せるか

### 論点

Favnir ではすでに ADT を中心に据える方針がある。  
そのため、別途 `enum` を導入するのではなく、

- `struct` = product
- `type ... | ...` = sum

で十分かもしれない。

### 価値

- `T?`
- `T!`
- `match`
- pattern binding

の理解がまとまりやすい。

### 注意点

- ユーザーにとって `enum` という言葉の方が直感的な可能性はある
- `type` を ADT に使う方針を強く打ち出す必要がある

### 方向

まずは:

- `type` を和型の中心にする
- `enum` は導入しない

でよい可能性が高い。

## 2. `inspect`

### 論点

pipeline の途中観測をどう扱うか。

例:

```fav
rows
|> ParseCsv
|> inspect("parsed")
|> ValidateUser
```

### 価値

- データ分析用途とかなり相性が良い
- debugging / notebook / explain とつながる
- `print` より構造化しやすい

### 注意点

- effect として扱うべき
- `Pure` を壊さない位置づけが必要

### 方向

- `inspect` は有望
- `!Trace` または `!Inspect` のような effect として扱う案が自然

## 3. `pipe match`

### 論点

`match` を pipeline に自然に載せる sugar を入れるか。

例:

```fav
row
|> parse_user
|> match {
    ok(user) => normalize_user(user)
    err(msg) => ...
}
```

### 価値

- 読み味がかなり良くなる
- `T!` / ADT / `match` と pipeline がつながる

### 注意点

- core ではなく sugar
- どこまで式として自然に展開できるかを決める必要がある

### 方向

- 有望
- まずは sugar として検討する

## 4. `collect / yield`

### 論点

列変換を、完全な generator ではなく限定構文として持つか。

例:

```fav
trf ActiveEmails: List<User> -> List<String> = |users| {
    collect {
        for user in users {
            if user.active {
                yield user.email
            }
        }
    }
}
```

### 価値

- 高階関数だけでは読みにくい処理をきれいに書ける
- 遅延評価や `Stream<T>` に伸ばしやすい

### 注意点

- loop / iteration 構文の話に入りやすい
- 最初から全面 lazy にしない方がよい

### 方向

- 有望
- ただし `Stream<T>` や collection API との関係を見ながら導入する

## 5. pattern guard としての `where`

### 論点

型制約ではなく、pattern guard 側に先に `where` を入れるか。

例:

```fav
match user {
    User { age } where age >= 18 => ...
    _ => ...
}
```

### 価値

- `match` がかなり強くなる
- ADT / pattern binding の使い勝手が上がる
- `if` と `match` の橋渡しになる

### 注意点

- 将来 generic constraint にも `where` を使う可能性がある
- 意味の混線を避ける必要がある

### 方向

- かなり有望
- まずは pattern guard 用途だけに限定するのがよい

## 今回落としたもの

今は優先度を下げるもの:

- `view`
- `check`
- `using`
- `schema`
- 初期段階の `derive`

理由:

- `fn` / `trf` / file/module 分離で十分代替できる
- 専用構文を増やすコストの方が大きい

## 仮の優先順位

次に詰めるなら、この順が自然。

1. ADT を `enum` 相当としてどう見せるか
2. pattern guard としての `where`
3. `pipe match`
4. `inspect`
5. `collect / yield`

## 短い結論

今の Favnir で次に価値が高いのは、

- ADT の見せ方
- `match` の強化
- pipeline の観測と分岐

まわりである。

つまり、関数型らしさを強めるにも、データ分析寄りの強みを伸ばすにも、  
まずは `ADT + match + pipeline` の連携を詰めるのがよい。
