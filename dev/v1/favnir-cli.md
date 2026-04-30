# Favnir CLI Draft

更新日: 2026-04-26

## 結論

Favnir の CLI は、単なる実行入口ではなくプロダクトの顔になる。

特に重要なのは:

- `fmt`
- `lint`
- `check`
- `test`
- `explain`

である。

Favnir は typed pipeline 言語なので、一般的な CLI より **`explain` を中心価値として強く持つ**のがよい。

## 初期コアコマンド

最初に必要なのは次。

- `fav run`
- `fav build`
- `fav exec`
- `fav check`
- `fav test`
- `fav fmt`
- `fav lint`
- `fav explain`

## 1. `fav run`

source を interpreter で実行する。

用途:

- notebook
- playground
- 小さい検証
- debug

例:

```text
fav run main.fav
```

## 2. `fav build`

portable artifact を生成する。

候補:

- bytecode
- wasm
- bundle

例:

```text
fav build main.fav -o dist/main.fvc
```

## 3. `fav exec`

build artifact を実行する。

例:

```text
fav exec dist/main.fvc
```

## 4. `fav check`

実行せずに検査だけ行う。

対象:

- 名前解決
- 型検査
- effect 検査
- `trf / flw` 接続検査

例:

```text
fav check
fav check apps/importer
```

これは CI でもかなり重要。

## 5. `fav test`

自前ランナーでテストを実行する。

例:

```text
fav test
fav test rune data.csv
fav test --jobs 4
fav test --max-memory 4GB
fav test --shard 1/4
```

## 6. `fav fmt`

コード整形。

例:

```text
fav fmt
fav fmt path apps/importer
```

CI 向けには:

```text
fav fmt --check
```

を持てるとよい。

## 7. `fav lint`

静的診断。

例:

```text
fav lint
```

見るもの:

- naming
- unused binding
- unreachable branch
- suspicious effect usage
- type naming suggestion

## 8. `fav explain`

Favnir の中心価値になりうるコマンド。

見せたいもの:

- `flw` の入出力型
- 各 `trf` の effect
- `T!` の発生箇所
- `bind / chain` の展開
- `type` の種類

例:

```text
fav explain apps/importer
fav explain rune data.csv
```

将来的には:

```text
fav explain --json
```

もあるとよい。

## 次段階で欲しいコマンド

初期コアの次に考えるもの:

- `fav repl`
- `fav init`
- `fav bundle`
- `fav publish`
- `fav doctor`

## `fav repl`

小さい式や `trf` を試す。

## `fav init`

rune / app / workspace の雛形を作る。

## `fav bundle`

配布用 artifact を束ねる。

例:

```text
fav bundle apps/importer
```

## `fav publish`

rune を registry へ公開する。

## `fav doctor`

環境確認。

## 後段階のコマンド

後で考えればよいもの:

- `fav deploy`
- `fav release`

## `deploy` を急がない理由

`deploy` は環境依存が強い。

Favnir の初期段階では:

- `bundle`
- `publish`

を先に持つ方が自然。

整理:

- `bundle` = 配布物を作る
- `publish` = rune を公開する
- `deploy` = 実行環境へ置く

## CI/CD での基本セット

最初の品質ゲートとしては次が自然。

```text
fav fmt --check
fav lint
fav check
fav test
```

これに:

- `--jobs`
- `--shard`
- `--max-memory`

が加わると、かなり実用的。

## コマンド体系の意図

Favnir の CLI は次の 3 層に分かれる。

### 言語コア

- `run`
- `build`
- `exec`
- `check`
- `fmt`
- `lint`
- `test`
- `explain`

### 開発支援

- `repl`
- `init`
- `doctor`

### 配布 / 公開

- `bundle`
- `publish`
- `release`
- `deploy`

## 短い結論

Favnir の CLI では、

- `fmt`
- `lint`
- `check`
- `test`
- `explain`

が特に重要。

中でも `explain` は、Favnir を他言語と差別化する顔になりうる。
