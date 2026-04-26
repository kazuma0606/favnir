# Favnir Testing Strategy

更新日: 2026-04-26

## 結論

Favnir では、早い段階で **自前のテストランナー** を持つべき。

理由:

- 外部ホスト言語の test runner に依存すると重くなりやすい
- workspace 全体を一気に起こす設計はメモリ的に厳しい
- explain / trace / effect 情報を活かしにくい

Forge のように「ホスト言語の test 実行にかなり依存する」設計は避けた方がよい。

## 背景

Forge では Rust と Cargo への依存が強く、workspace 全体を `cargo test` すると、かなり重くなった。

問題:

- build 単位が大きい
- test 実行がホスト都合に巻き込まれる
- workspace 全体が一度に立ち上がる
- メモリ予算を言語側で制御しにくい

Favnir では、最初からこの問題を避ける方がよい。

## 基本方針

### 1. `fav test` を持つ

テストは最初から自前ランナーで実行する。

例:

```text
fav test
fav test rune data.csv
fav test path apps/importer
```

### 2. build 生成物前提にしない

テストはまず:

- interpreter
- typed IR
- bytecode

のいずれかで走るようにする。

ネイティブバイナリ生成を前提にしない方が、

- 軽い
- metadata を保持しやすい
- explain と相性が良い

## 並列実行

並列実行は欲しいが、制御可能であることが重要。

### 最低限ほしいオプション

```text
fav test --jobs 4
fav test --jobs 1
fav test --jobs auto
```

### 方針

- デフォルトは安全寄り
- 明示的に並列度を指定できる
- CI / ローカルで使い分けられる

## メモリ制御

これはかなり重要。

少なくとも concept として最初から入れたい。

### 例

```text
fav test --max-memory 4GB
fav test --per-test-memory 256MB
```

### 意義

- 普通の PC で回る
- workspace が大きくなっても崩れにくい
- runaway test を抑制できる

## 分割実行

workspace や CI では shard がかなり有効。

### 例

```text
fav test --shard 1/4
fav test --shard 2/4
```

### 意義

- 大規模 rune 群でも分割できる
- CI 並列化しやすい
- メモリピークを抑えやすい

## 実行オプション

最初から欲しいもの:

- `--jobs N`
- `--max-memory`
- `--filter`
- `--fail-fast`
- `--trace`
- `--shard i/n`

## trace と explain

Favnir の test runner は、単に pass/fail を返すだけでは弱い。

失敗時に見たいもの:

- どの `trf`
- どの `flw`
- どこで `T!`
- どの effect

つまり、test runner は explain / trace と強くつながるべき。

## deterministic mode

並列実行を入れるなら、再現性も重要。

例:

```text
fav test --seed 42
fav test --deterministic
```

これにより:

- flaky test を減らしやすい
- 再現性の高い debug ができる

## profile

テストプロファイルを持てると便利。

例:

```text
fav test --profile quick
fav test --profile full
```

イメージ:

- `quick`
  - 単体中心
  - 軽い
  - 低メモリ
- `full`
  - integration を含む
  - trace 多め

## テストと言語モデル

テストはホスト言語の副産物ではなく、Favnir 自身の意味論を理解して動くべき。

つまり:

- rune
- flw
- trf
- type/effect metadata

を前提に実行・報告する。

## 推奨実装順

1. `fav test`
2. interpreter ベース実行
3. `--filter`
4. `--jobs`
5. `--fail-fast`
6. `--trace`
7. `--max-memory`
8. `--shard`

## 短い結論

Favnir では、**自前テストランナーと資源制御は初期要件**に入れるべき。

最低限必要なのは:

- `fav test`
- 並列度制御
- メモリ制限
- trace
- shard

普通の PC でまともに検証できることを最初から前提にしないと、後でかなり苦しくなる。
