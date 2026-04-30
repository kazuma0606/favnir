# Favnir Modules and Namespace Draft

更新日: 2026-04-26

## 結論

Favnir には `namespace` を入れた方がよい。

ただし、Rust の `mod` のような重い構成管理ではなく、  
**薄い名前境界**として設計するのがよい。

方針:

- 基本はファイルベース module
- その上で明示的な `namespace` を持てる
- `namespace` はトップレベル名の整理に使う
- lexical scope とは別物として扱う

## なぜ必要か

### 1. 名前衝突を避けやすい

Forge でも、クレート名や既存名との衝突が起きていた。  
Favnir でも標準ライブラリや userland が育つと、同じ問題が出る。

`namespace` があれば、

- 標準ライブラリ
- アプリケーションコード
- 外部ライブラリ

の名前を整理しやすい。

### 2. 標準ライブラリの設計がきれいになる

たとえば:

- `std.list`
- `std.result`
- `std.string`
- `data.csv`
- `data.json`

のように、意味ごとに整理しやすい。

### 3. 名前解決の実装が整理しやすい

名前解決は少なくとも次の層に分かれる。

- local lexical scope
- file/module scope
- namespace scope

この層が明確だと、checker や resolver が整理しやすい。

## lexical scope との違い

`namespace` はクロージャやブロックのスコープを置き換えるものではない。

分けるべき:

- lexical scope
  - `bind`
  - `fn`
  - `stage`
  - block
  - `match` arm

- namespace / module scope
  - トップレベルの名前整理
  - import/export
  - 標準ライブラリ整理

つまり、クロージャ capture の問題は lexical scope の問題であり、  
`namespace` はトップレベル名衝突の問題を解く仕組み。

## 推奨モデル

### 基本

- 1 ファイル = 1 module
- file path からデフォルト module path を作る
- 必要なら `namespace` で明示的に上書きできる

### 例

```fav
namespace data.csv

fn parse(text: String) -> List<Row> {
    ...
}
```

これにより、このファイルのトップレベル定義は `data.csv` 配下に属する。

## `use`

import 構文としては `use` を採用するのが自然。

例:

```fav
use std.list.map
use data.csv.parse
```

将来的には:

```fav
use std.list.{ map, filter, fold }
use data.csv as csv
```

のような形もありえるが、最初は単純な形からでよい。

## 最小仕様

最初に必要なのは次。

### 1. file-based module

- ファイル単位でトップレベル定義を持つ

### 2. explicit namespace

```fav
namespace data.csv
```

役割:

- トップレベル定義の所属先を明示する
- 名前衝突を避ける

### 3. `use`

```fav
use std.result.ok
use data.csv.parse
```

役割:

- 他 namespace の定義を参照する

## 最初は入れないもの

初期段階では次は後回しでよい。

- 複雑な re-export
- wildcard import
- import macro
- nested `mod`
- path alias を多用する仕組み

理由:

- まずは名前解決を単純に保ちたい
- parser / resolver / diagnostics を軽く保ちたい

## ファイルパスと namespace の関係

おすすめは次の形。

- デフォルトではファイルパスから module path を導く
- `namespace` が書かれていればそれを優先する

つまり:

- `src/data/csv.fav` -> 既定では `data.csv`
- ただしファイル先頭に `namespace import.csv` があればそれを採用

これなら:

- ファイルベースの手軽さ
- 明示 namespace の整理力

の両方を取れる。

## export の扱い

最初は単純でよい。

候補:

1. トップレベル定義はすべて公開
2. `pub` を後で足す

初期段階では 1 でもよい。  
名前空間と `use` が先に固まる方が重要。

## 名前解決の優先順

Favnir では、概念的には次の順で解決するのが自然。

1. local lexical scope
2. enclosing block / function scope
3. file/module top-level
4. imported names
5. fully qualified namespace path

これならクロージャ capture と module import が混ざりにくい。

## 例

### ローカルと namespace の分離

```fav
namespace data.csv

fn parse(text: String) -> List<Row> {
    bind rows <- ...
    rows
}
```

ここで `rows` は lexical scope、`data.csv.parse` は namespace 側の話。

### import

```fav
use data.csv.parse

bind rows <- parse(text)
```

## 仮の結論

Favnir の module system は、最初から重くしない方がよい。

ベストバランスは次。

- 基本はファイルベース
- 明示的な `namespace` を持つ
- `use` で取り込む
- lexical scope とは別管理

これで、見た目の整理、名前衝突回避、実装のきれいさをかなり両立できる。
