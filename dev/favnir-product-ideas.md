# Favnir Product Ideas

更新日: 2026-04-26

## 目的

このメモは、Favnir を単なる言語仕様で終わらせず、「育つプロダクト」として考えるための原則を整理するための一覧。

ここでの論点は、構文そのものではなく、

- どういう体験を重視するか
- どういうツール文化を持つか
- どうやって言語の価値を伝えるか

である。

## 1. Explain-First

Favnir は `trf` と `flw` を型付きで結合する言語なので、最初から「何がどうつながっているか」を説明しやすい。

この性質はかなり強い。

見せたいもの:

- `flw` 全体の入出力型
- 各 `trf` の effect
- どこで `T!` が発生するか
- どの branch が `match` で処理されるか

将来的には `fav explain` のような形で、コードの読み解きを支援するのがかなり有効。

## 2. Notebook / REPL 前提

Favnir は data-centric なので、ファイル実行だけでなく、対話的に試せる環境と相性が良い。

たとえば:

- 小さい式を試す
- `trf` 単体を流す
- `inspect` で途中を見る
- `match` や pattern をその場で試す

この用途には REPL または notebook が向いている。

## 3. Playground-Driven Design

Favnir は読み味と書き味がかなり重要。

そのため、新しい構文を入れるたびに、

- 小さいサンプル
- playground
- hover / explain 表示

で試せる方が、仕様書だけで詰めるより設計がぶれにくい。

## 4. Type Kind Awareness

`type` を統一入口にするなら、tooling 側で型の種類を区別して見せる価値が高い。

例:

- record type
- sum type
- fallible
- effectful

これは:

- hover
- lint
- explain
- AI 補完

のすべてに効く。

## 5. Flow-as-Asset

`flw` は単なるコード片ではなく、再利用可能な資産として扱える。

イメージ:

- 小さい `trf`
- 再利用可能な `flw`
- テスト済みの flow library
- 可視化可能な pipeline asset

Favnir の価値は、単発のコードより「安全に組める処理資産」にある。

## 6. Capability Profiles

effect と capability を持つなら、将来的に profile 概念と相性が良い。

例:

- local profile
- test profile
- prod profile

これにより:

- test では fake capability
- prod では real capability

のような切り替えが整理しやすくなる。

## 7. Data Contract Culture

Favnir は data-centric なので、外部入出力との契約を明示する文化が重要。

たとえば:

- 外部から何を受けるか
- どこで失敗するか
- 何を返すか

を `type`, `T?`, `T!`, `match` で明示できる。

`schema` を今すぐ専用構文にしなくても、この文化自体は強い。

## 8. Small Core, Strong Tooling

Favnir は、言語コアを増やしすぎるより、

- hover
- lint
- explain
- formatter
- playground
- REPL

を強くした方が育つ。

方針としては:

- コアは小さく
- sugar は慎重に
- tooling は強く

がよい。

## 9. AI-Native Metadata

今の時代なら、AI 補完や生成と相性の良い metadata を最初から意識すると強い。

例:

- record type
- sum type
- pure / effectful
- safe composition candidates

こうした情報を compiler / checker が持っていれば、

- 補完
- explain
- 自動生成
- リファクタ支援

の精度が上がる。

## 10. Example-Driven Standard Library

stdlib は理論から作るより、代表ユースケースから作る方が実用性が高い。

おすすめ題材:

- CSV import
- validation pipeline
- auth flow
- event enrichment
- group / aggregate / export

この 5 本くらいを通せば、必要な API がかなり見えるはず。

## 特に重要な原則

今の Favnir と特に相性が良いのは次の 5 つ。

1. Explain-First
2. Notebook / REPL
3. Type Kind Awareness
4. Flow-as-Asset
5. Small Core, Strong Tooling

## 短い結論

Favnir を育てる上で大事なのは、

- 言語コアを増やしすぎないこと
- `trf` / `flw` の価値を explain できること
- data-centric な体験を notebook / playground / tooling で支えること

である。

つまり、Favnir は「仕様の多さ」で勝つより、  
**小さいコアと強い可視化・補助ツール**で育てるのがよい。
