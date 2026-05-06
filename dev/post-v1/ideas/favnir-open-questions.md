# Favnir Open Questions

更新日: 2026-04-26

## 目的

このメモは、Favnir の残件を整理し、どれを今決めるべきか、どれを後回しにすべきかを分けるための一覧。

分類は次の 3 つ。

- `must`: 初期仕様を成立させるために先に決める
- `later`: 初期実装後に詰めればよい
- `maybe`: 必要性が見えてから検討する

## 判断軸

各論点は次の観点で判断する。

- これは core か sugar か
- これは lexical scope の話か module の話か
- これは value の型か effect の型か
- 初期仕様で必要か
- AI が安全に扱いやすくなるか

## must

### 1. ADT の具体構文

決めること:

- variant 記法
- payload の書き方
- `match` の pattern 構文

理由:

- `T?` / `T!` の内部意味に直結する
- parser / checker の中核になる

### 2. effect の合成規則

決めること:

- `Pure + Db = Db` のような基本規則
- effect の順序や集合性をどう考えるか

決定済み:

- `Emit<A> + Emit<B> = Emit<A | B>`
  - `Emit` は型パラメータを union で合成する
  - 複数 event を発火する `seq` の effect は `Emit<A | B>` として畳まれる

理由:

- `stage` と `seq` の型検査に必要
- Favnir の特徴そのもの

### 3. module / namespace / use の解決規則

決めること:

- file-based module の既定
- `namespace` の優先順位
- `use` の最小仕様

理由:

- 名前解決の基盤
- lexical scope と分離して早めに固めたい

### 4. 型推論の範囲

決めること:

- `bind` 右辺からどこまで推論するか
- `fn` / `stage` の引数注釈を必須にするか
- return type をどこまで省略可能にするか

理由:

- 書き味と checker の複雑さに直結する

### 5. generic inference の範囲

決めること:

- `map(_.email)` の `T, U` をどこまで推論するか
- generic `fn` / `stage` の型引数を明示させる場面をどこまで残すか

理由:

- 高階関数と placeholder の使いやすさに影響する

### 6. 標準ライブラリの最小集合

決めること:

- `List`
- `Map`
- `String`
- `Option`
- `Result`
- `map/filter/fold/flat_map`
- `IO.print` / `IO.println`

理由:

- syntax 単体では言語の使い勝手が決まらない
- core API を最初に絞る必要がある

## later

### 1. `event / emit / job` を初期仕様にどこまで入れるか

理由:

- 重要ではある
- ただしアプリケーション層なので、コア言語が固まってからでもよい

### 2. `try/catch` の導入範囲

理由:

- 外部境界に必要になる可能性は高い
- ただし `T!` と `match` を先に固める方が先

### 3. placeholder の拡張

論点:

- `_ + _`
- `_1`, `_2`
- `.email` shorthand

理由:

- 単一 `_` だけでも初期価値は十分ある

### 4. 明示的な部分適用

理由:

- カリー化の代替として便利
- ただし初期仕様に必須ではない

### 5. `event` と effect の統合の細部

論点:

- `Emit<Event>` の型表現
- event payload の generic 化

理由:

- `event` を入れる段階で詰めればよい

### 6. export / visibility

論点:

- すべて公開で始めるか
- `pub` を入れるか

理由:

- module system の次の段階でよい

## maybe

### 1. 自動カリー化

理由:

- 面白いが、初期仕様としては複雑さが勝ちやすい

### 2. do 記法

理由:

- モナド的処理をきれいに書ける
- ただし最初は combinator で十分

### 3. custom operator

理由:

- expressive ではある
- parser と可読性の負担が大きい

### 4. macro system

理由:

- 強力だが、今は設計を濁らせやすい

### 5. async / concurrency モデル

理由:

- 将来必要になる可能性は高い
- しかし effect / runtime / capability が先

### 6. trait に相当する compile-time constraint system

理由:

- generic constraint として後で要る可能性はある
- ただし今は generics + ADT + stage を先に固めたい

## ベストプラクティス

残件を進めるときは、次を守るのがよい。

### 1. semantics を先に決める

表面構文より先に、

- 型の意味
- effect の意味
- 名前解決

を固める。

### 2. sugar は core に落とせる形だけにする

例:

- placeholder
- shorthand
- future decorator-like syntax

はいずれも core 構文へ展開できる形に限定する。

### 3. runtime 都合を表面構文に漏らさない

例:

- ownership
- lifetime
- allocator 都合

はユーザー構文に出さない。

### 4. 1つの記法に1つの責務だけ持たせる

例:

- `bind <-` は束縛だけ
- `T!` は failure だけ
- `!Db` は effect だけ

### 5. AI が安全に使えるかを判断基準にする

曖昧さが増える機能は慎重に入れる。

特に注意:

- 暗黙規則
- 多義的 sugar
- mutable state
- 複雑な placeholder

## 仮の次ステップ

次に決めるべき順はこのあたり。

1. ADT と pattern の具体構文
2. effect 合成規則
3. module / namespace / use の詳細
4. 型推論の範囲
5. 標準ライブラリの最小集合

ここまで決まると、初期仕様としてかなり骨格が固まる。
