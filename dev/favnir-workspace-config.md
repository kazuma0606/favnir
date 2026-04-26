# Favnir Workspace and Config Draft

更新日: 2026-04-26

## 結論

Favnir では、設定は **root first** にした方がよい。

おすすめ:

- ルートに 1 つメイン設定ファイルを置く
- 形式は TOML を第一候補にする
- workspace を最初から意識する
- `settings/` は補助用に留める

## なぜ root first か

モノレポや workspace を考えると、起点になる設定が root にないと扱いづらい。

理由:

- どこがプロジェクト起点か分かりやすい
- tooling が簡単になる
- workspace 解決がしやすい
- bundler / build / explain が一つの入口を持てる

逆に、`settings/` を主入口にすると:

- どれが本設定か分かりにくい
- monorepo の root 解決が面倒
- 実装も複雑になる

## 設定形式

候補は:

- JSON
- YAML
- TOML

### JSON

利点:

- 機械向けに単純

弱点:

- コメントに弱い
- 手書き設定としては硬い

### YAML

利点:

- 柔らかい

弱点:

- 曖昧さが多い
- インデント事故が起きやすい

### TOML

利点:

- 人が書きやすい
- 構造が十分ある
- dependency / build / workspace 記述と相性が良い

### 推奨

Favnir では TOML が最も自然。

## 設定ファイル名

第一候補:

```text
fav.toml
```

このファイルを workspace / build / bundle / entry の起点にする。

## `settings/` の位置づけ

`settings/` を全否定する必要はない。

ただし用途は補助に限定するのがよい。

例:

```text
fav.toml
settings/
  local.toml
  test.toml
```

つまり:

- `fav.toml` = main config
- `settings/*` = environment / local override

## workspace

Favnir はモノレポをやりたくなる可能性が高い。  
そのため、最初から workspace 概念を意識した方がよい。

例:

```toml
[workspace]
members = [
  "runes/core",
  "runes/data-csv",
  "apps/importer",
]
```

## `rune` との関係

整理:

- `workspace`
  - リポジトリ管理単位
- `rune`
  - 公開・配布単位
- `namespace`
  - コード上の名前解決単位

この 3 つは別物として分ける方が自然。

## `namespace` との違い

たとえば:

- workspace: `apps/importer`
- rune: `data.csv`
- namespace: `data.csv.parse`

はそれぞれ違う層の概念。

`namespace` はトップレベル名の整理、  
`rune` は公開単位、  
`workspace` はルート管理単位。

## `rune` ごとの設定

各 rune 側も設定を持てるとよい。

例:

```toml
[rune]
name = "data.csv"
version = "0.1.0"

[dependencies]
core = { path = "../core" }
```

## build / bundle

workspace と rune があるなら、bundler はかなり自然に入る。

例:

```toml
[build]
target = "bytecode"
out_dir = "dist"

[bundle]
entry = "apps/importer"
```

これにより:

- workspace 全体を解決
- 必要 rune を集める
- dependency graph を固定
- bundle / bytecode / WASM を出す

という流れが作れる。

## ディレクトリ構成の例

```text
fav.toml
settings/
  local.toml
runes/
  core/
  data-csv/
apps/
  importer/
```

## 推奨初期仕様

最初に必要なのは次。

1. `fav.toml`
2. `[workspace]`
3. `[rune]`
4. `[dependencies]`
5. `[build]`

その後に足すもの:

- `[bundle]`
- profile ごとの設定
- override
- publish metadata

## 短い結論

Favnir の設定は:

- root に置く
- TOML を使う
- workspace を最初から意識する
- `settings/` は補助にする
- `namespace` / `rune` / `workspace` を分離する

この整理が一番自然。
