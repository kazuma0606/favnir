# Favnir v2.6.0 Language Specification

作成日: 2026-05-13

---

## テーマ

複数ファイルにわたる実用規模のプログラムを書けるようにする。

v2.6.0 では Favnir に `import "path"` 構文を追加し、
プロジェクト内の複数ファイルと rune ライブラリを名前空間ベースで参照できるようにする。

---

## 1. `import` 構文

### 1-1. ローカルファイル import

`src/` 起点の相対パスで他ファイルのシンボルを namespace として参照する。

```favnir
import "models/user"           // namespace: user.User, user.ParseUser
import "models/user" as u      // alias: u.User, u.ParseUser
```

- パスはファイルシステムの `/` 区切り（`src/` 起点）
- namespace は末尾セグメント（`"models/user"` → `user`）から自動決定
- `as alias` で任意の namespace 名を付けられる
- インポートしたシンボルは `namespace.SymbolName` で参照する

### 1-2. rune import

`runes/` ディレクトリ起点で rune ライブラリをインポートする。

```favnir
import rune "validate"         // namespace: validate.Required, validate.Email
import rune "stat" as s        // namespace: s.int, s.float
```

- `runes/<name>/<name>.fav` を読み込む
- `fav.toml` の `[runes] path` が設定されている場合はそのディレクトリから探す
- `fav.toml` が見つからない場合は `runes/` ディレクトリをデフォルトとして使う

### 1-3. 公開宣言

```favnir
// models/user.fav
public type User = { name: String  age: Int }   // 外部から参照可
public stage ParseUser: String -> User = ...    // 外部から参照可
fn internal_helper: String -> String = ...     // このファイル内のみ
```

`public` キーワードはすでに存在する。新しいキーワードは不要。

### 1-4. バレルファイル（namespace の re-export）

ディレクトリをまとめて公開する場合は `public import` を使う。

```favnir
// models/models.fav
public import "models/user"    // user.* を models namespace に re-export
public import "models/post"
```

---

## 2. シンボル参照

import したシンボルは `namespace.SymbolName` で参照する。

```favnir
import "models/user"

public fn main() -> Unit !Io {
    bind u <- user.ParseUser("Alice,30")
    IO.println(u.name)
}
```

`as` alias を使った場合：

```favnir
import "models/user" as m

bind u <- m.ParseUser("Alice,30")
```

---

## 3. エラーコード

### E080 — 循環 import

```
E080: circular import detected
  "models/user" imports "models/post" which imports "models/user"
```

- import グラフにサイクルがある場合に報告する
- サイクルを構成するパスを表示する

### E081 — namespace 競合

```
E081: namespace conflict: 'user' is imported from both "models/user" and "auth/user"
  hint: use `as` to resolve:
    import "models/user" as model_user
    import "auth/user"   as auth_user
```

- 同じ namespace 名を持つ複数の import がある場合に報告する
- `as` を使った解決ヒントを表示する

---

## 4. `fav check --dir`

ディレクトリ以下の全 `.fav` ファイルを一括チェックする。

```bash
fav check --dir src/
```

- `src/` 以下の `*.fav` ファイルを再帰的に収集する
- 各ファイルを `import` グラフに従ってトポロジカル順にチェックする
- エラーがあれば全ファイル分まとめて出力し、終了コード 1 を返す

---

## 5. 実装箇所サマリ

### 新規追加

| 箇所 | 内容 |
|---|---|
| `src/frontend/lexer.rs` | `TokenKind::Import` トークンを追加 |
| `src/ast.rs` | `Item::ImportDecl { path: String, alias: Option<String>, is_rune: bool, is_public: bool, span: Span }` を追加 |
| `src/frontend/parser.rs` | `parse_import_decl` — `import "path"` / `import rune "path"` / `public import "path"` を解析 |
| `src/middle/checker.rs` | import 処理: `ImportDecl` ごとにモジュールをロードし namespace テーブルに登録; `namespace.Symbol` の型解決; E080/E081 検出 |
| `src/driver.rs` | `cmd_check_dir(dir)` — ディレクトリ以下の全 .fav ファイルを一括チェック |
| `src/main.rs` | `fav check --dir <dir>` の引数解析とルーティングを追加 |

### 既存コードとの関係

- 既存の `use dotted.path.symbol` 構文は**残す**（後方互換）
- 既存の `Resolver` は内部的に引き続き使用する（`import` の実装も Resolver 経由で可）
- `import "path"` と `use dotted.path.symbol` は共存可能

---

## 6. 互換性

- v2.5.0 までの全コードはそのまま有効
- `use` 構文は引き続き動作する
- `public` キーワードは従来通り（新しい用途は `public import` のみ）
- `fav check <file>` の動作は変わらない（`--dir` は新フラグ）
