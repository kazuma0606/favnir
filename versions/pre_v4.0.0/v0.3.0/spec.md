# Favnir v0.3.0 仕様書

更新日: 2026-04-28

## 概要

v0.3.0 はモジュールシステムを追加するバージョン。
複数のファイルに分割して書かれた Favnir プロジェクトを、
`namespace` / `use` / `fav.toml` によって統合・管理できるようにする。

あわせて `rune` の境界概念を最小実装し、
`private` / `internal` / `public` の visibility enforcement を完成させる。

---

## スコープ

### v0.3.0 で追加するもの

- `namespace` 宣言構文 (ファイルのモジュールパスを明示)
- `use` import 構文 (他ファイルの定義を参照)
- ファイルベースのモジュール解決 (ファイルパス → モジュールパスの自動導出)
- `fav.toml` による最小プロジェクト設定
- `rune` の境界概念 (= `fav.toml` が存在するディレクトリ配下)
- visibility enforcement の完成
  - `private` — 同一ファイル内のみ (既存構文に enforcement を追加)
  - `internal` — 同一 rune 内のみ (境界チェックを新規実装)
  - `public` — 外部公開
- 名前解決エラーの改善 (E012〜E016)
- `fav check` のプロジェクトレベル対応 (引数なしで rune 全体をチェック)
- `fav run` の `main` エントリポイント探索改善

### v0.2.0 で追加済み・v0.3.0 では変更なし

- `Visibility::Internal` バリアント (AST)
- `internal` キーワード (lexer / parser)
- `TypeDef.visibility` フィールド
- `fav explain` の VIS 列

### v0.3.0 でも含まないもの

- ワイルドカード import (`use data.csv.*`)
- グループ import (`use data.csv.{ parse, encode }`)
- re-export
- 複数 rune 間の依存 (multi-rune / external dependencies)
- `rune` レジストリ / パッケージマネージャ
- workspace
- `rune` キーワード (構文は v1.0.0 以降)
- field-level visibility
- `fav fmt` / `fav test`
- bytecode / VM

---

## `fav.toml` の最小仕様

プロジェクトルートに置くファイル。`fav.toml` が存在するディレクトリが **rune の境界** になる。

```toml
[rune]
name    = "myapp"
version = "0.1.0"
src     = "src"
```

| キー | 型 | 意味 |
|---|---|---|
| `name` | String | rune 名。`use` での参照パスのルートになる |
| `version` | String | セマンティックバージョン (現時点では参照のみ) |
| `src` | String | ソースルートディレクトリ (省略時は `.`) |

### モジュールパスの導出

`src` からの相対パスをドット区切りに変換したものがデフォルトのモジュールパス。

```
src/data/users.fav  →  data.users
src/main.fav        →  main
```

ファイル先頭に `namespace` があればそれを優先する。

---

## `namespace` 宣言

ファイルのモジュールパスを明示的に宣言する。ファイル先頭に書く。

```
namespace_decl = "namespace" module_path
module_path    = IDENT ("." IDENT)*
```

例:

```fav
namespace data.users

type User = {
    id:    String
    name:  String
    email: String
}

public fn create(name: String, email: String) -> User { ... }
```

- `namespace` はファイルに 1 つだけ書ける
- 宣言がない場合はファイルパスから自動導出する
- `namespace` と自動導出のパスが一致しない場合は警告 (W001) を出す

---

## `use` import 宣言

他ファイルの定義を現在のスコープに取り込む。

```
use_decl = "use" module_path
```

`module_path` の最後のセグメントがシンボル名、それ以前がモジュールパスとして解釈される。

```fav
use data.users.create    // data.users モジュールの create を import
use data.users.User      // data.users モジュールの User 型を import
```

import したシンボルは、ファイル内で修飾なしに参照できる。

```fav
use data.users.create
use data.users.User

public fn main() -> Unit !Io {
    bind user <- create("Alice", "alice@example.com")
    IO.println(user.name)
}
```

完全修飾パスでも参照できる:

```fav
bind user <- data.users.create("Alice", "alice@example.com")
```

### `use` の制約

- `use` はファイルのトップレベルにのみ書ける
- `namespace` 宣言の後に書く (`namespace` → `use` → 定義の順)
- 1 つの `use` で 1 シンボルを import する (グループ化は v0.4.0 以降)
- 循環 import はエラー (E012)

---

## 名前解決の優先順

同名のシンボルが複数の階層に存在する場合の解決順:

1. ローカル lexical スコープ (`bind` 束縛, 関数パラメータ)
2. ファイルのトップレベル定義
3. `use` で import したシンボル
4. 完全修飾パスによる参照 (`data.users.create`)

---

## visibility enforcement

### `private` (デフォルト)

annotation なし、または明示的な `private`。
**同一ファイル内** からのみ参照可能。
他ファイルから参照した場合は E015 エラー。

```fav
// data/users.fav
fn validate(email: String) -> Bool { ... }  // private: このファイル内のみ
```

### `internal`

**同一 rune 内** (= 同じ `fav.toml` が存在するプロジェクト配下) からのみ参照可能。
別 rune から参照した場合は E016 エラー。
v0.3.0 では single-rune のみ対応 (外部 rune への参照は未実装)。

```fav
// data/users.fav
internal fn normalize(user: User) -> User { ... }  // 同一 rune 内なら使える
```

### `public`

制限なし。外部からも参照可能。
`use` で import できるのは `public` なシンボルのみ。

```fav
// data/users.fav
public fn create(name: String, email: String) -> User { ... }
```

### visibility と `use` の関係

| 参照元 | `private` | `internal` | `public` |
|---|---|---|---|
| 同一ファイル | OK | OK | OK |
| 同一 rune の別ファイル | E015 | OK | OK |
| 別 rune (v0.4.0+) | E015 | E016 | OK |

---

## モジュール解決アルゴリズム

`fav run <file>` または `fav check <file>` を実行したとき:

1. 指定ファイルをパース
2. `use` 宣言を収集
3. 各 `use` について:
   a. `module_path` の最後のセグメントをシンボル名 `sym` とする
   b. 残りのセグメントをモジュールパス `mod_path` とする
   c. `fav.toml` の `src` ディレクトリ配下で `mod_path` をファイルパスに変換
      (`data.users` → `<src>/data/users.fav`)
   d. そのファイルをパース・型チェック
   e. `sym` が存在し `public` であることを確認
   f. 現在のスコープに追加
4. チェック済みモジュールをキャッシュ (同じファイルを複数回読まない)

### `fav.toml` なしの場合

単一ファイルモード。`use` は使えない。
既存の `fav run <file>` / `fav check <file>` の動作は変わらない。

---

## CLI 変更点

### `fav run` の変更

```
fav run [--db <url>] [<file>]
```

`<file>` を省略した場合:
1. カレントディレクトリの `fav.toml` を探す
2. `src/main.fav` または `main.fav` を探して実行する

### `fav check` の変更

```
fav check [<file>]
```

`<file>` を省略した場合:
1. カレントディレクトリの `fav.toml` を探す
2. `src` 配下の全 `.fav` ファイルを型チェックする

### `fav explain` の変更

```
fav explain [<file>]
```

`<file>` を省略した場合: `fav.toml` の `src` 配下全体をチェック。
VIS 列は既存の実装をそのまま使う。

---

## エラーコードの追加

| コード | 種類 |
|---|---|
| E012 | `use` — 循環 import |
| E013 | `use` — シンボルが見つからない |
| E014 | `use` — シンボルが `public` でない (visibility 違反) |
| E015 | `private` なシンボルへの別ファイルからの参照 |
| E016 | `internal` なシンボルへの別 rune からの参照 |
| W001 | `namespace` 宣言がファイルパスから導出したパスと一致しない |

---

## プロジェクト構成例

```
myapp/
  fav.toml
  src/
    main.fav
    data/
      users.fav
    service/
      user_service.fav
```

### `fav.toml`

```toml
[rune]
name    = "myapp"
version = "0.1.0"
src     = "src"
```

### `src/data/users.fav`

```fav
namespace data.users

type User = {
    id:    String
    name:  String
    email: String
}

// private: このファイル内のみ参照可
fn validate_email(email: String) -> Bool {
    String.len(email) > 0
}

// internal: 同一 rune 内から参照可
internal fn normalize(user: User) -> User {
    User { id: user.id, name: user.name, email: user.email }
}

// public: 外部からも参照可
public fn create(name: String, email: String) -> User !Db {
    bind id <- Util.uuid();
    bind _ <- Db.execute(
        "INSERT INTO users (id, name, email) VALUES (?, ?, ?)",
        id, name, email
    );
    User { id: id, name: name, email: email }
}

public fn find(id: String) -> Map<String, String>? !Db {
    Db.query_one("SELECT id, name, email FROM users WHERE id = ?", id)
}
```

### `src/service/user_service.fav`

```fav
namespace service.user_service

use data.users.User
use data.users.create
use data.users.normalize   // OK: internal でも同一 rune

public fn register(name: String, email: String) -> User !Db !Emit<UserRegistered> {
    bind user <- create(name, email);
    bind user <- normalize(user);
    emit UserRegistered { user_id: user.id };
    user
}
```

### `src/main.fav`

```fav
use service.user_service.register

public fn main() -> Unit !Io !Db !Emit<UserRegistered> {
    bind _ <- Db.execute("CREATE TABLE IF NOT EXISTS users (id TEXT PRIMARY KEY, name TEXT NOT NULL, email TEXT NOT NULL UNIQUE)");
    bind user <- register("Alice", "alice@example.com");
    IO.println(user.name)
}
```

---

## 型システムへの影響

### モジュールスコープの型チェック

`Checker` に `module_registry: HashMap<String, ModuleScope>` を追加する。
`ModuleScope` は公開済みシンボルの型情報を保持する。

```rust
struct ModuleScope {
    symbols: HashMap<String, (Type, Visibility)>,
}
```

### `use` の型チェック

`use data.users.create` が宣言されたとき:
1. `data.users` モジュールをロードし型チェック
2. `create` が存在し `public` であることを確認
3. `create` の型を現在の `Checker` の env に追加

### 名前解決の変更

現在の `Checker` は `env: TyEnv` (HashMap) でシンボルを管理している。
import されたシンボルは同じ `env` に追加する（スコープ上は区別しない）。
ただし visibility 情報は別途保持して、参照時にチェックする。

---

## 完了条件

- `fav.toml` を持つプロジェクトで `fav run` / `fav check` が動く
- `use data.users.create` で別ファイルの `public fn` を参照できる
- `private` なシンボルを別ファイルから参照すると E015 が出る
- `internal` なシンボルを同一 rune 内から参照できる
- `namespace` 宣言がパースされ、モジュールパスとして使われる
- `fav check` (引数なし) で rune 全体の型チェックができる
- `examples/multi_file/` で複数ファイル構成のサンプルが動く
