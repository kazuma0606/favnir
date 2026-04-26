# Favnir Execution Model Draft

更新日: 2026-04-26

## 結論

Favnir では、Forge のように「ネイティブバイナリ生成を最初の正解」にしない方がよい。

Favnir の性格には、次の構成の方が合う。

- `run` は interpreter
- `build` は軽い portable artifact
- `exec` は artifact 実行

つまり、TS -> JS に近い発想で、

- source
- compiled artifact
- tiny runtime

を分けた方が自然。

## なぜそうするのか

Forge では、Rust を経由することで:

- build 生成物が重くなる
- notebook / REPL / quick iteration と相性が悪い
- ホスト言語都合が成果物に強く出る

という問題が出やすかった。

Favnir では:

- data-centric
- notebook / playground と相性が良い
- explain / metadata を残したい

ので、軽い artifact の方がかなり筋が良い。

## 1. `run`

`run` は interpreter 実行。

用途:

- notebook
- REPL
- 小さい検証
- explain / debug
- playground

ここでは、開発ループの速さが最優先。

## 2. `build`

`build` はネイティブ exe を吐くのではなく、portable artifact を生成する。

候補:

- bytecode
- WASM module
- self-contained script bundle
- IR package + tiny runtime

### 推奨

最初の本命は:

- typed IR -> bytecode
- tiny VM / runtime

その後、必要なら WASM backend へ伸ばす。

## 3. `exec`

`exec` は build artifact を実行する。

例:

```text
fav run main.fav
fav build main.fav -o main.fvc
fav exec main.fvc
```

または:

```text
fav build main.fav -o main.wasm
fav exec main.wasm
```

## bytecode と WASM

### bytecode

利点:

- 自前制御がしやすい
- explain / debug と相性が良い
- source metadata を残しやすい

欠点:

- runtime / VM を自前で持つ必要がある

### WASM

利点:

- sandbox しやすい
- 配布しやすい
- Rust 以外の host にも載せやすい

欠点:

- lowering 設計がやや重い
- notebook / explain 用 metadata は別途必要になりやすい

### 推奨順

1. interpreter
2. bytecode
3. 必要なら WASM

## script / bundle 発想

Favnir は「実行可能スクリプト」を生成する方向とも相性が良い。

例:

```text
fav build app.fav --target script
```

で、

- bytecode / IR
- tiny runtime stub
- metadata

を束ねた単一ファイルを作る。

これは:

- 配布しやすい
- notebook とは別に軽量実行しやすい
- TS/JS 的な感覚に近い

## explain / metadata との相性

軽い artifact 方式の方が、Favnir らしい explain を保ちやすい。

残したい情報:

- source span
- type info
- effect info
- flow trace

ネイティブバイナリへ完全に落とすより、こうした情報を保持しやすい。

## エラーハンドリングとの相性

Favnir ではエラーハンドリングを強くしたいので、source-level 情報を捨てにくい実行方式の方がよい。

portable artifact は:

- span
- type
- effect
- trace

を runtime metadata として残しやすい。

これは Favnir にかなり合っている。

## 実行方式の分担

### `fav run`

- source を interpreter で実行
- notebook / REPL / debug 向け

### `fav build`

- source から artifact を生成
- bytecode / WASM / bundle など

### `fav exec`

- artifact を実行

## 推奨アーキテクチャ

最初の現実的な構成は次。

1. parser
2. typed IR
3. interpreter
4. bytecode compiler
5. tiny VM
6. 必要なら WASM backend

## 何を避けるか

最初から避けたいもの:

- ネイティブ exe を唯一の build 目標にする
- host 言語依存が強すぎる artifact
- source metadata を捨てる設計

## 短い結論

Favnir では、実行モデルとして

- `run` = interpreter
- `build` = portable artifact
- `exec` = artifact 実行

を分けるのがよい。

最初の本命は:

- interpreter
- bytecode
- tiny runtime

であり、ネイティブバイナリは最初の目標にしない方が、言語の性格とかなり合う。
