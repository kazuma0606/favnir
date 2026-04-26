# Favnir Data Structures Draft

更新日: 2026-04-26

## 結論

Favnir は data-centric な言語を目指す以上、基本データ構造はかなり重要。

ただし、すべてを同じレベルで「組み込み」にする必要はない。  
コア構文や型システムと強く結び付くものと、標準ライブラリ寄りでよいものを分けた方がよい。

## 基本方針

データ構造は次の 3 層に分ける。

1. core
2. stdlib
3. later extensions

## 1. core

最初から強く持つべきもの。

- `Bool`
- `Int`
- `Float`
- `String`
- tuple
- `struct`
- ADT
- `List<T>`
- `T?`
- `T!`

理由:

- pattern binding に直結する
- `match` に直結する
- pipeline の主要な入出力になる
- 言語の書き味そのものを決める

## 2. stdlib

初期から欲しいが、構文や意味論の中心にしなくてもよいもの。

- `Map<K, V>`
- `Set<T>`
- `Vector<T>`
- `Tensor<T, Shape>`
- `Stream<T>`

理由:

- 重要ではある
- ただし構文より API と runtime 表現の話になりやすい

## 3. later extensions

後で拡張すればよいもの。

- matrix
- dataframe
- sparse tensor
- deque / queue
- graph
- domain-specific numeric collections

## `List<T>`

Favnir の基本 sequence は `List<T>` を中心にするのがよい。

```fav
bind users <- rows |> map(parse_user)
```

役割:

- pipeline の基本対象
- `map/filter/fold/flat_map` の中心
- immutable sequence

### 配列 / ベクトルとの関係

最初から `Array` と `Vector` を細かく分けるより、表面上は `List<T>` を中心にした方がよい。

理由:

- 書き味を単純に保てる
- 実装の最適化は runtime 側で後から寄せられる

## tuple

tuple はかなり重要。

```fav
bind (x, y) <- point
fn split_name(name: String) -> (String, String)
```

役割:

- 多値返却
- 一時的な grouping
- pattern binding と相性が良い

これは stdlib ではなく、言語コア寄りに扱う価値がある。

## `struct`

named record として重要。

```fav
struct User {
    name: String
    email: String
}
```

役割:

- domain data
- pipeline の主要な中間表現
- pattern binding の対象

## ADT

Favnir では `struct` だけでなく ADT も中心に置く。

```fav
type ImportResult =
    | Success { count: Int }
    | Failure { message: String }
```

役割:

- `T?`
- `T!`
- parser/checker の内部表現
- domain result

## `T?` / `T!`

表面構文としては軽いが、意味としてはコアデータ構造。

- `T?` = optional value
- `T!` = fallible value

内部的には generic ADT として扱う。

## `Map<K, V>`

重要だが、最初は stdlib 寄りで十分。

```fav
fn group_by_dept(users: List<User>) -> Map<String, List<User>> {
    ...
}
```

注意点:

- `K` にどんな制約を課すか
- hash / equality をどう扱うか

は後で必要になる。

## `Set<T>`

かなり欲しいが、初期段階では stdlib 扱いが自然。

```fav
fn unique_ids(ids: List<UserId>) -> Set<UserId> {
    ...
}
```

注意点:

- `Eq` / `Hash` 的な制約が必要になる
- generic constraint の話とつながる

## `Vector<T>`

欲しいが、最初は `List<T>` を主役にする方がよい。

`Vector<T>` の役割:

- random access
- 数値計算寄りの最適化
- packed representation

ただしこれは API / runtime 最適化寄りの話なので、最初は stdlib で十分。

## `Tensor<T, Shape>`

データ分析用途では重要。  
ただし最初から言語コアに入れると重い。

おすすめ方針:

- 最初は stdlib 型として置く
- shape system は後回し
- 必要なら後で `Tensor<T, Shape>` を強化する

つまり、存在は早めに意識するが、コア意味論には入れすぎない。

## pattern とデータ構造

pattern binding / `match` の対象として強く扱いたいのは次。

- tuple
- `struct`
- ADT

後で検討するもの:

- `List` の head/tail pattern
- `Map` pattern
- `Set` pattern

理由:

- 最初は record / variant / tuple だけで十分強い
- collection pattern は複雑化しやすい

## immutable

基本データ構造はすべて immutable 前提にする。

これはかなり重要。

理由:

- typed pipeline と相性が良い
- closure capture を安全にしやすい
- AI が扱いやすい
- runtime 設計が単純になる

## collection API

Favnir では、標準 API の一貫性がかなり大事。

最低限ほしい操作:

- `map`
- `filter`
- `fold`
- `flat_map`
- `group`
- `count`
- `first`
- `last`

データ構造そのものより、これらの API の統一感が「関数型言語らしさ」をかなり左右する。

## 仮の結論

Favnir のデータ構造は次の整理がよい。

### core

- tuple
- `struct`
- ADT
- `List<T>`
- `T?`
- `T!`

### stdlib

- `Map<K, V>`
- `Set<T>`
- `Vector<T>`
- `Tensor<T, Shape>`
- `Stream<T>`

### later

- dataframe
- sparse tensor
- graph

これにより、言語コアを軽く保ちつつ、データ分析言語としての拡張余地も残せる。
