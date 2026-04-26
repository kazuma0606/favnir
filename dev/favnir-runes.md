# Favnir Runes Draft

更新日: 2026-04-26

## 結論

Favnir では、`flw` 単体を公開単位にするだけでは小さすぎる。  
その上に、公開・配布・依存解決のまとまり単位が必要。

その単位を `rune` とするのはかなり自然。

## 役割の階層

Favnir では、次の階層で考えるのがよい。

- `fn`
  - 純粋関数
- `trf`
  - 結合可能な処理片
- `flw`
  - 再利用可能な処理列
- `rune`
  - 公開・配布・再利用のまとまり単位

この整理だと、`rune` は Favnir 版の crate / library / package に相当する。

## なぜ `rune` か

`rune` は `trf` の名前として使うには少し意味が広すぎたが、公開単位としてはむしろちょうどよい。

理由:

- 世界観に合う
- 単なるファイルや module より強い意味を持てる
- コードだけでなく「意味情報ごと束ねる」感じを出せる

## `rune` に含まれるもの

`rune` は複数のトップレベル定義を束ねる。

対象:

- `type`
- `struct`
- `fn`
- `trf`
- `flw`

将来的には:

- metadata
- explain 情報
- AI 補完向け情報
- capability 要求情報

も含められると強い。

## 基本イメージ

```fav
rune data.csv

use std.list.map

pub type Row = {
    name: String
    email: String
}

pub trf ParseCsv: String -> List<Row> = |text| {
    ...
}

pub flw ImportUsers =
    ParseCsv
    |> ValidateUser
    |> SaveUsers
```

## `rune` の役割

### 1. 公開単位

どの `type`, `fn`, `trf`, `flw` を外へ見せるかを管理する。

### 2. 配布単位

依存関係や version を持てる単位にする。

### 3. 再利用単位

Favnir の資産は `flw` 単位だけでなく、`rune` 単位で再利用される方が自然。

### 4. tooling 単位

将来的に:

- hover
- explain
- lint
- AI 補完

のための metadata を束ねる単位にもできる。

## `flw` との違い

`flw` は再利用可能な処理列。  
`rune` はそれを含む公開単位。

つまり:

- `flw` はコード構造
- `rune` は配布構造

である。

## `module` / `namespace` との違い

`module` や `namespace` は名前解決の整理。

`rune` はそれより一段上の概念で、

- 依存
- version
- 公開面
- metadata

を持つ。

整理すると:

- lexical scope = ローカル名の範囲
- namespace = トップレベル名の整理
- rune = 公開・配布単位

## 公開面

最初は `pub` を使う形でよい。

```fav
pub fn normalize_email(value: String) -> String {
    ...
}
```

```fav
pub trf ParseCsv: String -> List<Row> = |text| {
    ...
}
```

## 依存関係

最初は `use` と設定ファイルの組み合わせでもよい。

将来的には:

- rune name
- version
- dependencies

を持てる形にする。

例:

```toml
[rune]
name = "data.csv"
version = "0.1.0"
```

## Explain と metadata

`rune` は Favnir の Explain-First 方針とも相性が良い。

たとえば:

- この rune が公開する `flw`
- 各 `trf` の effect
- 依存している capability
- 型の種類

を rune 単位で説明できる。

これは単なる package より Favnir らしい価値になる。

## 仮の最小仕様

最初に必要なのは次。

1. `rune <name>`
2. `pub`
3. `use`
4. rune ごとの名前空間

後で足すもの:

- version
- dependency manifest
- metadata export
- explain export

## 仮の結論

`rune` は `trf` よりも、Favnir の crate / package / library 単位として使う方がしっくり来る。

特に、

- `fn`
- `trf`
- `flw`
- `rune`

の階層はかなり整理が良い。

Favnir の世界観としても、  
**処理片を束ね、意味情報ごと公開する単位**として `rune` を置くのは強い。
