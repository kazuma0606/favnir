# Favnir Product Next

更新日: 2026-04-26

## 目的

このメモは、Favnir を「良い言語」だけでなく「育つプロダクト」にするために、次に重視したい概念を絞って整理するための一覧。

今の段階では、次の 5 つが特に有望。

1. Explain-By-Default
2. Flow Catalog
3. Traceable Runtime
4. Notebook Native
5. AI-Native Authoring

## 1. Explain-By-Default

### 考え方

`explain` を後付けの補助機能ではなく、Favnir の中心体験にする。

見せたいもの:

- この `flw` の入出力型
- 各 `trf` の effect
- どこで `T!` が発生するか
- `bind / chain` がどう展開されるか
- `match` の分岐と型の対応

### なぜ重要か

Favnir は:

- typed pipeline
- effect
- ADT
- `bind / chain`

を持つので、説明可能性が高い。  
この性質を前面に出すと、理解コストを下げられる。

### 価値

- 学習しやすい
- デバッグしやすい
- AI にも説明しやすい
- `fav explain` が単なる補助でなく「Favnir の顔」になる

## 2. Flow Catalog

### 考え方

`rune` の中の `flw` を、単なるコードではなく再利用資産として一覧化する。

載せたい情報:

- 入出力型
- effect
- 依存 rune
- 使用例
- explain snapshot

### なぜ重要か

Favnir の価値は、単発コードより「組める処理資産」にある。  
そのため、`flw` を探せる・比べられる・再利用できる体験が強い。

### 価値

- flow の再利用が進む
- AI 補完が候補を出しやすい
- チーム開発時にも資産が見える

## 3. Traceable Runtime

### 考え方

実行時に、型や flow と結びついた trace を残せる runtime を持つ。

残したいもの:

- どの `trf` を通ったか
- どこで failure したか
- effect がどこで発生したか
- `inspect` の出力

### なぜ重要か

普通の log だけだと、Favnir の意味論が見えない。  
Favnir らしい trace は:

- pipeline
- effect
- failure

に沿って見せられる必要がある。

### 価値

- explain と runtime がつながる
- notebook とも相性が良い
- エラーハンドリングの強さにも直結する

## 4. Notebook Native

### 考え方

notebook を後付けではなく、最初から一級ユースケースとして扱う。

向いている操作:

- 小さい式の試行
- `trf` 単体の実行
- `inspect`
- `match` の確認
- `T!` の失敗可視化

### なぜ重要か

Favnir は data-centric なので、対話的な試行とかなり相性が良い。  
ファイル実行だけに閉じるのはもったいない。

### 価値

- データ分析用途に強い
- 学習と実験が速い
- playground 文化ともつながる

## 5. AI-Native Authoring

### 考え方

AI が書きやすい言語にするだけでなく、AI に説明しやすい metadata を持つ。

候補:

- record type / sum type 判定
- pure / effectful 判定
- safe composition candidates
- rune catalog
- explain graph

### なぜ重要か

Favnir は:

- 型が強い
- effect が明示される
- `trf / flw` の接続候補が見える

ので、AI 補完との相性がかなり良い。

### 価値

- 補完の質が上がる
- サンプル生成が安定する
- explain と連動できる

## 優先順位

今の Favnir で特に効果が大きい順なら:

1. Explain-By-Default
2. Traceable Runtime
3. Flow Catalog
4. Notebook Native
5. AI-Native Authoring

## ベストプラクティス

これらを進めるときの指針:

- コア言語を増やしすぎない
- まず metadata と explain を整える
- runtime trace を source / type / effect と結びつける
- notebook / AI はその上に載せる

## 短い結論

Favnir を唯一無二にするのは、構文の多さではなく、

- `trf / flw` を説明できること
- 実行を trace できること
- 処理資産を catalog 化できること

にある。

つまり、次に伸ばすべきは「言語そのもの」だけでなく、  
**Explain / Trace / Catalog / Notebook / AI** の層である。
