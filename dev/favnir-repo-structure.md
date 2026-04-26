# Favnir Repository Structure Draft

更新日: 2026-04-26

## 目的

このメモは、Favnir の初期リポジトリ構造を散らかりにくい形で決めるための草案。

特に分けたいのは次の 2 つ。

- 言語自体を開発する場所
- Fav を使った実装やサンプルを置く場所

Forge ではこの境界が曖昧になりやすかったので、最初から分けておく。

## 基本方針

最初から次の 3 層に分ける。

1. 言語本体
2. Fav で書く実装群
3. 例 / 検証 / ドキュメント

## 推奨構造

```text
favnir/
  dev/
  docs/
  host/
  language/
  selfhost/
  runes/
  apps/
  examples/
  tests/
  notebooks/
  dist/
  settings/
  fav.toml
```

## 各ディレクトリ

### `dev/`

仕様メモ、構想、設計ノート。

今まで作ってきた:

- syntax
- runtime
- async
- testing
- runes
- workspace

などの文書を置く場所。

### `docs/`

外向け・整理済みドキュメント。

例:

- getting started
- language guide
- rune guide
- CLI guide

`dev/` が生メモ、`docs/` が公開向け整理版。

### `host/`

Rust などで作る薄いホスト基盤。

役割:

- CLI launcher
- file IO
- sandbox
- artifact loader
- capability bridge

Fav 本体ではなく外周だけを置く。

### `language/`

Fav 言語そのものの実装。

候補:

- parser
- AST
- resolver
- type/effect checker
- interpreter
- bytecode compiler
- formatter
- lint

ここは「言語実装本体」。

### `selfhost/`

Fav で Fav を実装していく領域。

最初は subset から始める。

例:

- parser subset
- type representation
- explain
- test orchestration の一部

ここは将来 `language/` を置き換えていく候補。

### `runes/`

Fav の公開・配布単位を置く場所。

例:

```text
runes/
  core/
  data-csv/
  auth/
  http/
```

ここに stdlib や共有 rune を置く。

### `apps/`

Fav を使ったアプリケーションや entry project を置く。

例:

```text
apps/
  importer/
  notebook-demo/
  auth-demo/
```

`runes/` は再利用資産、`apps/` は実行単位。

### `examples/`

小さい学習用・説明用サンプル。

役割:

- syntax の見本
- docs から参照する例
- playground 的題材

### `tests/`

言語本体の検証。

用途:

- parser fixture
- type checker fixture
- integration test
- CLI test

ここは `apps/` と分ける。

### `notebooks/`

Notebook Native を見据えた検証置き場。

用途:

- data analysis demo
- REPL/interactive examples
- inspect / explain 確認

### `dist/`

build / bundle / artifact の出力先。

### `settings/`

補助設定。

ただし main config はここではなく root に置く。

## root 設定

main config は root の `fav.toml`。

役割:

- workspace
- build
- bundle
- entry

`settings/` は補助用に留める。

## workspace の例

```text
favnir/
  fav.toml
  runes/
    core/
    data-csv/
  apps/
    importer/
```

```toml
[workspace]
members = [
  "runes/core",
  "runes/data-csv",
  "apps/importer",
]
```

## 一番大事な分離

特に重要なのは次。

### 1. `language/` と `selfhost/`

- `language/` = いま動く実装
- `selfhost/` = Fav で置き換えていく実装

これを最初から分けておくと散らかりにくい。

### 2. `runes/` と `apps/`

- `runes/` = 再利用資産
- `apps/` = 実行可能プロジェクト

### 3. `dev/` と `docs/`

- `dev/` = 生メモ
- `docs/` = 整理済み文書

## 初期段階の最小構成

全部を一気に作らなくてもよい。

最初は次でも十分。

```text
favnir/
  dev/
  docs/
  host/
  language/
  selfhost/
  runes/
  apps/
  tests/
  fav.toml
```

## 短い結論

Favnir では、最初から

- 言語本体
- selfhost
- rune 資産
- app 実装

を分けた方がよい。

特に:

- `language/`
- `selfhost/`
- `runes/`
- `apps/`

の分離が、Forge で起きた散らかりを避ける上でかなり重要。
