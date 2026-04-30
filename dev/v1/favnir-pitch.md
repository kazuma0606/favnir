# Favnir Draft

更新日: 2026-04-26

## 概要

Favnir は、型付き pipeline と effect を中核にした関数型データ言語。

目的は次の 3 つ。

- データ変換を読みやすく安全に書けること
- 副作用を型レベルで明示し、純粋な処理と分離すること
- AI が誤りにくい、結合可能性の高いコードを書けること

Favnir は Rust の `struct` / `trait` モデルをそのまま引き継ぐのではなく、次の考え方を中心に置く。

- データは immutable
- 計算は `fn` / `stage` / `flow` の合成
- 副作用は effect として明示
- 型は Rust 写像ではなく言語意味として定義

## 一言でいうと

Favnir は、

> 副作用を明示しながら、安全に結合できる処理片を型で扱うための言語

である。

特に重要なのは、単なる関数合成ではなく、**名前付きの `stage` を安全に結合できる**こと。

## なぜ別言語にするのか

現行 Forge は、Rust のラッパーとして始まったため、`struct` や `trait` を軸にした設計の影響を強く受けている。

しかし Favnir で目指したい方向は次の通り。

- `stage / flow / effect` を中核にする
- `|>` を単なる構文糖ではなく型付き合成にする
- 副作用あり / なしを厳密に区別する
- 再代入を原則禁止する
- 型システムを Rust 型表現から切り離す

ここまで行くと、Forge の後方互換を気にしながら進めるより、Favnir として切り出した方が設計の純度を保てる。

## 言語の核

Favnir のコア概念は次の 5 つ。

1. `struct` / ADT
2. `stage`
3. `flow`
4. effect
5. pattern match

`trait` は中心概念ではない。  
能力や差し替えは `capability`、`effect`、`wire` のような別概念に分離して扱う。

この中でも、`stage` を安全に結合できることは Favnir の中核価値の一つ。

## `stage` 合成の価値

普通の関数合成は `A -> B` と `B -> C` をつなぐだけで終わる。

Favnir の `stage` はそれより強い。

- 名前付き
- pipeline に載る
- effect を持てる
- checker が接続可能性を検証できる

つまり、単に合成できるだけでなく、**effect を含めて安全に合成できる処理片**を第一級で扱える。

これは次の価値を生む。

- 再利用しやすい
- explain / 可視化しやすい
- 最適化単位として扱いやすい
- AI が安全に探索しやすい
- アプリケーション層より低いレベルで意味を組み立てられる

## 設計原則

### 1. immutable を基本にする

- `bind` は初回束縛専用
- 再代入は原則禁止
- 状態更新が必要な場合のみ、明示的な effect か state 機構に閉じ込める

### 2. 純粋処理と副作用を分ける

- 純粋な変換は `Pure`
- DB, IO, network, event emission などは effect として明示
- 副作用を含む pipeline は型に現れる

### 3. 結合可能性を型で判定する

- ある処理片と次の処理片がつながるかは型で決まる
- つながらないものは checker が拒否する
- 再利用単位は「型」ではなく「処理片」

### 4. データ処理を第一級にする

- `|>` を中心にしたデータ変換を自然に書ける
- `map`, `filter`, `fold`, `group`, `emit_each` のような処理が中核に来る
- object-centric ではなく data-centric

### 5. AI に優しい言語にする

- 曖昧な慣習より明示的な型規則を優先する
- 副作用の境界を visible にする
- mutable state を抑え、局所的な推論で安全性が崩れにくい設計にする

## 表面構文の方向

ユーザーには次のような構文を見せる。

```fav
stage ParseCsv: String -> List<Row> = |text| {
    ...
}

stage ValidateUser: List<Row> -> List<User> = |rows| {
    ...
}

stage SaveUsers: List<User> -> List<UserId> !Db = |users| {
    ...
}

flow ImportUsers =
    ParseCsv
    |> ValidateUser
    |> SaveUsers
```

このとき:

- `stage` は再利用可能な処理片
- `flow` は複数 stage の合成
- `!Db` は effect 注釈
- `|>` は型付き合成

Favnir では、`stage` 合成は単なる sugar ではない。  
接続は型検査され、effect も追跡される。

## `bind <-`

通常の束縛には `let` ではなく `bind <-` を使う。

```fav
bind rows <-
    text
    |> ParseCsv

bind users <-
    rows
    |> ValidateUser
    |> NormalizeUser
```

狙い:

- `=` による代入連想を避ける
- 束縛と更新を文法レベルで分ける
- immutable 言語としての顔を明確にする

## 内部モデル

表面は読みやすくしつつ、内部では厳密に扱う。

```text
Stage<Input, Output, Fx>
```

例:

- `ParseCsv : Stage<String, List<Row>, Pure>`
- `ValidateUser : Stage<List<Row>, List<User>, Pure>`
- `SaveUsers : Stage<List<User>, List<UserId>, Db>`
- `ImportUsers : Stage<String, List<UserId>, Db>`

この `Stage<Input, Output, Fx>` モデルがあることで、Favnir は:

- 再利用可能な処理片
- 型付き接続
- effect 追跡
- explain / 可視化
- 最適化単位

を同時に扱える。

## effect の最小集合

初期案としては次を持つ。

- `Pure`
- `State`
- `Io`
- `Db`
- `Network`
- `Emit<Event>`

最初は増やしすぎない方がよい。

## ADT と match

Favnir は `struct` だけでなく ADT を持つべき。

```fav
type ImportResult =
    | Success { count: Int }
    | Partial { count: Int, invalid: Int }
    | Failure { message: String }
```

これにより:

- event の結果表現
- validation 結果
- parser / checker の内部表現

を自然に書ける。

`match` は中核機能として強化する。

## `trait` の代わりに何を置くか

`trait` が担っていた責務は分解する。

- データ定義: `struct`, ADT
- 再利用可能な処理片: `stage`
- 処理列の再利用: `flow`, `bundle`
- 外部依存の要求: capability / effect
- 差し替え: `wire`, `bind`, `app`

これにより、OOP 的な拡張ではなく、pipeline 中心の合成へ設計を寄せる。

## 再代入について

Favnir では、Rust の `mut` のように「付ければ再代入できる」設計は採らない。

原則:

- `bind` は初回束縛のみ
- 再代入は不可
- 状態変化が必要な場合は、明示的な `state` 機構または effect ブロック内に限定する

## 最小仕様

最初の実装スコープは次に絞る。

### 型

- `Int`
- `Float`
- `Bool`
- `String`
- `Unit`
- `List<T>`
- `Map<K, V>`
- `Option<T>`
- `struct`
- ADT

### 構文

- `bind <-`
- `fn`
- `stage`
- `flow`
- `type`
- `match`
- `if`
- `|>`

### effect

- `Pure`
- `Io`
- `Db`
- `Emit<Event>`

## CLI とファイル拡張子

仮案:

- 言語名: `Favnir`
- CLI: `fav`
- 拡張子: `.fav`

例:

```text
fav run main.fav
fav check import.fav
fav fmt pipeline.fav
```

## 最初のユースケース

Favnir の最初の題材として相性が良いのは次の領域。

- CSV / JSON / table の変換
- validation pipeline
- ETL 的なデータ流し込み
- event 発火を伴う業務処理
- self-hosted parser / checker の一部実験

## 実装の入り口

最初の実装順はこのくらいがよい。

1. parser
2. `stage / flow` の AST
3. `Stage<A, B, Fx>` ベースの型 checker
4. `|>` の型付き合成
5. ADT と `match`
6. 純粋 pipeline interpreter
7. effect 付き stage の最小実行機構

## 短い結論

Favnir は Forge の置き換えではない。  
Forge から生まれた、より純粋で、より型主導で、より pipeline 指向の次世代言語案である。

その核は次の一文に尽きる。

> 安全に結合できる処理片を、effect を含めて型で扱う。
