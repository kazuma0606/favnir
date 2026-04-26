# Favnir Self-Hosting Plan

更新日: 2026-04-26

## 結論

Favnir は Forge よりかなりセルフホスト向き。

方針としては:

- Fav 本体はできるだけ早く Fav に寄せる
- Rust は当面ホスト基盤に留める
- 最初から大きな VM を目標にしない

つまり、

- **Fav が本体**
- **Rust は外周**

という構図で進めるのが自然。

## Rust に残すもの

初期段階で Rust に残すのは次のようなホスト基盤。

- CLI launcher
- file IO
- process / OS 境界
- sandbox
- artifact loader
- capability bridge
- 必要なら tiny runtime shell

## Fav に寄せたいもの

できるだけ早い段階で Fav に寄せたいのは次。

- parser
- AST
- ADT / type checker
- effect checker
- `bind / chain` 展開
- `trf / flw` 合成検査
- explain
- test orchestration
- lint / formatter の一部
- rune / package metadata handling の一部

## なぜ Forge と違うのか

Forge:

- Rust が実装本体
- 言語はその上に載る
- セルフホストは後から追いかける

Fav:

- 最初からセルフホスト前提
- Rust は仮の母艦
- 言語本体はなるべく早く Fav 側へ移す

## フェーズ

### Phase 0: コア仕様固定

先に固めるもの:

- syntax
- ADT / pattern
- effect
- `trf / flw / rune`
- module / namespace
- test model

完了条件:

- 小さいコア仕様が文書とサンプルで固定されている

### Phase 1: Rust host + Fav frontend design

最初の実装:

- Rust で `fav run/check/test/explain`
- ただし内部モデルは Fav 本体へ移しやすく設計する
- AST / IR / metadata を Fav で再実装しやすい形にする

ここでは大きな VM は不要。  
まず interpreter で十分。

### Phase 2: Fav selfhost subset

Fav 自身で書き始める対象:

- parser subset
- type representation
- pattern logic
- explain
- lint / test の一部

いきなり全部ではなく、独立しやすい層から始める。

### Phase 3: Selfhost interpreter

ここで「Fav で Fav を解釈する」に入る。

対象:

- `bind`
- `fn`
- `trf`
- `flw`
- ADT
- `match`
- `if`
- `T?`
- `T!`

まだ不要なもの:

- full async
- full parallel
- bundle / publish
- heavy stdlib

### Phase 4: Artifact 化

必要になったら:

- interpreter の次に軽い artifact 化
- bytecode
- bundle
- 後で WASM

最初から大きな VM を作らない方がよい。

### Phase 5: Rust host の薄化

最終的に Rust に残すのは:

- launcher
- IO bridge
- capability bridge
- sandbox
- artifact/runtime shell

Fav 側に寄せるのは:

- compiler frontend
- checker
- explain
- test runner
- package / rune handling の大半

## 推奨順

1. コア仕様固定
2. Rust host で最小実装
3. explain / check / test から selfhost 化
4. interpreter subset selfhost
5. 軽い artifact 化
6. Rust host を薄くする

## 短い結論

Fav は Forge よりずっとセルフホスト向き。

最初から VM を目標にせず、

- Rust host
- selfhost frontend
- interpreter

の順で進めるのが一番自然。
