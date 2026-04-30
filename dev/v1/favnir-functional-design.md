# Favnir Functional Design Draft

更新日: 2026-04-26

## 目的

このメモは、Favnir を「Forge 由来の DSL」ではなく、「実用に寄せた関数型言語」として整理するための補助設計メモ。

主な論点は次の 4 つ。

- `job / event / app` をどの層に置くか
- カリー化をどう扱うか
- 高階関数とモナド的操作をどう入れるか
- プレイスホルダーをどう設計するか

## 1. `job / event / app` の位置づけ

Forge で進めていた方向性自体は悪くなかった。  
ただし、Favnir の言語コアとして前面に出すと少し重い。

Favnir では層を分ける。

### 言語コア

- `bind <-`
- `fn`
- `stage`
- `flow`
- `type`
- `struct`
- `match`
- `if`
- effect

### アプリケーション層

- `event`
- `emit`
- `job`
- composition root

### 構成 / モジュール層

- `module`
- `use`
- capability binding

結論:

- `job / event / app` は消さない
- ただし言語コアの制御構文ではなく、一段上のアプリケーション構文として扱う

これにより、

- コア言語は関数型らしく保てる
- それでも実用 DSL 的な強みは残せる

## 2. カリー化

カリー化は、複数引数関数を「1引数関数を返す関数の連鎖」として扱う考え方。

例:

```fav
fn add(x: Int) -> Int -> Int {
    |y| x + y
}
```

これにより:

```fav
bind inc <- add(1)
```

のような部分適用ができる。

### 方針

Favnir では、カリー化そのものは相性が良い。

ただし初期段階では、全面的な自動カリー化は入れない方がよい。

理由:

- 多引数関数のエラーメッセージが複雑になる
- 暗黙の部分適用が AI にも人間にも誤用されやすい
- `stage` と通常関数の境界が曖昧になりやすい

### 推奨案

- カリー化可能なモデルは持つ
- ただし、明示的な 1 引数関数の連鎖として書く
- `fn add(x, y)` を暗黙に `Int -> Int -> Int` とみなすことは最初はしない

つまり:

- カリー化の素地はある
- 自動カリー化は後回し

## 3. 高階関数

Favnir では高階関数は必須。

最低限ほしいもの:

- `map`
- `filter`
- `fold`
- `flat_map`
- `group`
- `compose`

理由:

- `stage / flow` があっても、局所的な変換には高階関数が必要
- data-centric な記述と相性が良い
- pipeline 言語らしさを強く出せる

関数型は `Fn<T, U>` ではなく `T -> U` で扱う。

```fav
fn map<T, U>(items: List<T>, f: T -> U) -> List<U> {
    ...
}
```

## 4. モナド的操作

モナドは理論としてはかなり有用。

特に次の型では自然に出てくる。

- `T?`
- `T!`
- effect 付き計算

ただし、Favnir では最初から「モナド」という言葉や概念を前面に出しすぎない方がよい。

### 方針

- 内部意味論や標準ライブラリではモナド的に設計してよい
- ただしユーザー向けには操作として見せる

最初に見せるべきもの:

- `map`
- `flat_map`
- `and_then`
- `or_else`

例:

```fav
parse_user(row)
    .and_then(validate_user)
    .map(normalize_user)
```

あるいは pipeline と組み合わせるなら:

```fav
row
|> parse_user
|> and_then(validate_user)
|> map(normalize_user)
```

結論:

- モナド的な設計は入れる
- ただし「モナド」を前面に出すより、combinator を標準で持つ

## 5. プレイスホルダー

これはかなり重要。

Favnir が関数型・pipeline 言語として読みやすくなるかどうかに直結する。

### ほしいユースケース

```fav
rows |> map(_.email)
rows |> filter(_.active)
users |> group(_.department)
```

この形は:

- 読みやすい
- 高階関数と相性が良い
- 小さいクロージャの冗長さを減らせる

### 最小方針

最初は単一プレイスホルダーだけに絞る。

候補:

- `_`
- `.field` sugar

### 推奨案

- `_` を基本プレイスホルダーにする
- `.email` のような field shorthand は `_ .email` の sugar とみなす

例:

```fav
rows |> map(_.email)
rows |> filter(_.active)
```

### 最初は入れないもの

- 複数プレイスホルダー
- `$0`, `$1`, `$2`
- 曖昧な暗黙引数規則

理由:

- 曖昧さが増える
- checker が複雑になる
- AI も解釈を誤りやすくなる

## 6. 名前を付けることのコストと利点

完全に無名の関数や合成だけで言語を作ると、実装コストは上がる。

特に:

- parser
- checker
- diagnostics
- tooling

が難しくなる。

そのため、Favnir でも名前付き構文は持つべき。

ただし、名前を付ける場所を絞る。

持つべきもの:

- `fn`
- `stage`
- `flow`
- `type`
- `event`
- `job`

前面に出しすぎないもの:

- decorator
- runtime binding 構文
- framework 的 sugar

## 7. 関数型らしさの優先順位

Favnir を関数型寄りにするなら、優先順位は次の通り。

1. immutable
2. ADT
3. `match`
4. 高階関数
5. 関数型 `A -> B`
6. `T?` / `T!`
7. カリー化の素地
8. effect を型で持つ
9. モナド的 combinator
10. プレイスホルダー

## 8. 仮の結論

Favnir では次の整理が自然。

- `job / event / app` はコア構文ではなくアプリケーション層へ下げる
- 高階関数は必須
- カリー化は可能性を残すが、最初は自動化しない
- モナドは内部思想として採用し、表面には combinator として見せる
- プレイスホルダーは単一 `_` を有力候補にする

これなら、実用性を残しつつ、関数型言語としての一貫性もかなり高くできる。
