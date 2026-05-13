# Favnir v2.6.0 Language Specification

更新日: 2026-05-13

---

## 概要

v2.6.0 では、モジュール単位の読み込みを行う `import` 構文と、ディレクトリ全体を検査する `fav check --dir` を追加する。

既存の `use dotted.path.symbol` は引き続き有効であり、`import` はそれとは別の手段として導入される。

---

## 1. import 構文

### 1-1. ローカルファイル import

`src/` 配下の `.fav` ファイルを namespace 単位で読み込む。

```favnir
import "models/user"
import "models/user" as u
```

- `import "models/user"` は `src/models/user.fav` を解決する
- 既定の namespace 名はパス末尾のセグメント名である
- `as <alias>` を付けると namespace 名を上書きできる
- 読み込んだ公開シンボルは `namespace.Symbol` 形式で参照する

例:

```favnir
import "models/user"

public fn main() -> Unit !Io {
    bind u <- user.ParseUser("Alice,30")
    IO.println(u.name)
}
```

### 1-2. rune import

`runes/` 配下の rune ライブラリを import できる。

```favnir
import rune "validate"
import rune "stat" as s
```

- `import rune "validate"` は `runes/validate/validate.fav` を解決する
- `fav.toml` に `[runes] path = "custom-runes"` がある場合は、そのディレクトリを優先する
- `fav.toml` に設定がない場合の既定値は `runes/`

### 1-3. public import

別モジュールが公開している namespace を再公開したい場合は `public import` を使う。

```favnir
public import "models/user"
public import "models/post"
```

- `public import` された namespace は、そのモジュールを import した側からも参照できる
- 再公開対象は import 先モジュールの公開シンボルに限られる

### 1-4. 公開シンボル

import 先から見えるのは公開シンボルのみである。

```favnir
public type User = { name: String, age: Int }
public stage ParseUser: String -> User = ...
fn internal_helper: String -> String = ...
```

- `User` と `ParseUser` は import 先から参照できる
- `internal_helper` は参照できない

---

## 2. 名前解決

`import` で導入された namespace は `namespace.Symbol` で解決される。

```favnir
import "models/user" as m

public fn main() -> Unit !Io {
    bind u <- m.ParseUser("Alice,30")
    IO.println(u.name)
}
```

- 左辺の `namespace` は import 時に導入された名前である必要がある
- 右辺の `Symbol` は import 先モジュールの公開シンボルである必要がある
- namespace が解決できない場合や、公開されていないシンボルを参照した場合は既存の名前解決エラーになる

---

## 3. エラー

### E080: circular import detected

循環 import を検出した場合に報告する。

```text
E080: circular import detected
  "models/user" imports "models/post" which imports "models/user"
```

- ローカル import / rune import の両方が対象
- import 解決中の経路に同じモジュールが再入した時点で失敗する

### E081: namespace conflict

同じ namespace 名に対して複数の import が競合した場合に報告する。

```text
E081: namespace conflict: 'user' is imported from both "models/user" and "auth/user"
  hint: use `as` to resolve:
    import "models/user" as model_user
    import "auth/user" as auth_user
```

- 既定 namespace 同士の衝突
- 既定 namespace と `as` alias の衝突
- 既存シンボル名と import namespace 名の衝突

---

## 4. CLI

### 4-1. fav check --dir

ディレクトリ配下の全 `.fav` ファイルを再帰的に検査する。

```bash
fav check --dir src/
```

- 指定ディレクトリ以下の `.fav` ファイルを列挙する
- 各ファイルに対して通常の型検査と import 解決を実行する
- 1 件でもエラーがあれば終了コード `1` を返す
- エラーがなければ終了コード `0` を返す

### 4-2. 既存コマンドとの互換性

- `fav check <file>` の挙動は変更しない
- `fav run` / `fav build` でも import 解決は有効

---

## 5. 設定ファイル

`fav.toml` では rune 解決先を設定できる。

```toml
[runes]
path = "custom-runes"
```

- `path` がある場合は `<project-root>/custom-runes` を rune ルートとして使う
- `path` がない場合は `<project-root>/runes` を使う

---

## 6. 実装方針

- `use` は後方互換のため維持する
- `import` は top-level item として扱う
- import 先モジュールはチェック時にロードし、公開シンボルを namespace 単位で保持する
- `public import` は re-export 用の公開スコープへ転写する

---

## 7. 期待される状態

- `import "models/user"` で `user.Symbol` が解決できる
- `import "models/user" as u` で `u.Symbol` が解決できる
- `import rune "validate"` で rune ライブラリが解決できる
- `public import` により namespace の再公開ができる
- 循環 import は E080 になる
- namespace 競合は E081 になる
- `fav check --dir <dir>` でディレクトリ全体を検査できる
