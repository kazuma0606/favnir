# v17.5.0 — REPL 品質向上 Spec

Date: 2026-06-15

## 概要

`fav repl`（v9.10.0 実装）に `:doc` / `:load` / `:paste` / `:history` / `:save` と
タブ補完を追加する。データ探索ツールとして Jupyter ノートブックの代替として使えるレベルにする。

---

## 現在の REPL 状態（v9.10.0）

```
ReplSession { definitions: String, def_names: Vec<String> }
コマンド: :quit / :reset / :help / :env / :type <expr>
入力: stdin の 1 行ずつ（rustyline なし）
```

---

## 追加コマンド

### `:doc <Module.fn>`

```
favnir> :doc List.map
List.map(list: List<A>, fn: A -> B) -> List<B>
  Apply fn to each element of list.

favnir> :doc String.trim
String.trim(s: String) -> String
  Remove leading and trailing whitespace.
```

**実装**: 静的な builtin doc テーブル（`BUILTIN_DOCS`）を driver.rs に追加。
MDX ファイルのパース不要。既知の stdlib 関数の型シグネチャ + 1 行説明を収録。

### `:load <path>`

```
favnir> :load src/helpers.fav
loaded: safe_div, clamp, normalize
```

**実装**: ファイルを読み込み、トップレベル定義を `handle_definition` で一つずつ追加。

### `:save <path>`

```
favnir> :save session.fav
saved 3 definitions to session.fav
```

**実装**: `session.definitions` をファイルに書き出す。

### `:history`

```
favnir> :history
1: fn double(x: Int) -> Int { x * 2 }
2: double(21)
3: :type double(21)
```

**実装**: `ReplSession` に `history: Vec<String>` を追加。
各入力行（コマンド・定義・式）を追記。

### `:paste` / `:end` — 複数行入力

```
favnir> :paste
... fn double(x: Int) -> Int {
...   x * 2
... }
... :end
defined: double
```

**実装**: `:paste` を検出したらループで行を収集。`:end` を受け取ったら連結して `handle_definition`/`handle_expression` に渡す。

### タブ補完

```
favnir> List.<Tab>
List.map   List.filter   List.group_by   List.sort_by   List.length   ...

favnir> :d<Tab>
:doc
```

**実装**: `repl_complete_prefix(prefix: &str) -> Vec<String>` 関数。
rustyline を使わず、`cmd_repl` が非インタラクティブのためテスト可能な独立関数として実装。
インタラクティブ実行時は rustyline の `Completer` に接続。

---

## 更新後のコマンド一覧

```
favnir> :help
Commands:
  :help              show this help
  :quit / :q         exit the REPL
  :reset             clear all session definitions
  :env               show accumulated definitions
  :type <expr>       show the type of an expression
  :doc <Module.fn>   show function signature and description
  :load <path>       load .fav file definitions into session
  :save <path>       save session definitions to file
  :history           show input history
  :paste ... :end    enter multi-line definition mode
```

---

## API 設計

### `ReplSession` 拡張

```rust
struct ReplSession {
    definitions: String,
    def_names: Vec<String>,
    history: Vec<String>,      // v17.5.0 追加
}

impl ReplSession {
    fn add_history(&mut self, line: &str) { ... }
    fn print_history(&self) { ... }
}
```

### 新規関数

```rust
// :doc コマンドの処理
pub fn repl_doc_str(target: &str) -> Option<String>

// :load コマンドの処理
pub fn handle_load_cmd(path: &str, session: &mut ReplSession) -> Result<(), String>

// :paste ブロックの処理（収集済み文字列を受け取る）
pub fn handle_paste_block(src: &str, session: &mut ReplSession)

// タブ補完（prefix に続く候補を返す）
pub fn repl_complete_prefix(prefix: &str) -> Vec<String>
```

---

## builtin docs テーブル

`BUILTIN_DOCS: &[(&str, &str, &str)]` = `(name, signature, description)` の配列。

収録関数（最低限）:

| 名前 | シグネチャ |
|---|---|
| `List.map` | `(list: List<A>, fn: A -> B) -> List<B>` |
| `List.filter` | `(list: List<A>, fn: A -> Bool) -> List<A>` |
| `List.length` | `(list: List<A>) -> Int` |
| `List.group_by` | `(fn: A -> K, list: List<A>) -> Map<K, List<A>>` |
| `List.sort_by` | `(list: List<A>, fn: A -> K) -> List<A>` |
| `List.flat_map` | `(list: List<A>, fn: A -> List<B>) -> List<B>` |
| `List.fold` | `(list: List<A>, init: B, fn: B -> A -> B) -> B` |
| `String.trim` | `(s: String) -> String` |
| `String.length` | `(s: String) -> Int` |
| `String.to_upper` | `(s: String) -> String` |
| `String.to_lower` | `(s: String) -> String` |
| `String.split` | `(s: String, sep: String) -> List<String>` |
| `Json.stringify` | `(val: A) -> Result<String, String>` |
| `Json.parse` | `(s: String) -> Result<A, String>` |

---

## タブ補完ロジック

```rust
pub fn repl_complete_prefix(prefix: &str) -> Vec<String> {
    // "List." → List の全メソッド
    // "String." → String の全メソッド
    // ":d" → [":doc"]
    // ":" → 全コマンド
    // "" → 空
}

// モジュール補完テーブル
const COMPLETIONS: &[&str] = &[
    "List.map", "List.filter", "List.length", "List.group_by", ...
    "String.trim", "String.length", ...
    ":help", ":quit", ":reset", ":env", ":type", ":doc", ":load", ":save", ":history", ":paste",
];
```

---

## Cargo 依存

rustyline は現在未追加。インタラクティブ補完には必要だが、
テスト対象の関数（`repl_complete_prefix` 等）は rustyline に依存しないため、
rustyline 追加は optional とし、まず補完ロジックを独立関数として実装する。

ただし `cmd_repl` の対話ループを `rustyline` で置き換えると補完・履歴が向上する。

---

## テスト（v175000_tests、5件）

| テスト名 | 内容 |
|---|---|
| `version_is_17_5_0` | バージョン文字列が "17.5.0" であること |
| `repl_doc_command` | `repl_doc_str("List.map")` が Some を返しシグネチャを含む |
| `repl_type_command` | `:type 1 + 2` が型情報を出力できる（エラーなし） |
| `repl_load_file` | 一時ファイルを作成し `handle_load_cmd` でセッションに取り込める |
| `repl_paste_mode` | `handle_paste_block` で複数行定義をセッションに追加できる |

---

## 完了条件（PASS=5）

1. `repl_doc_str("List.map")` が型シグネチャを返す
2. `:type <expr>` が既存通り動作する（リグレッションなし）
3. `:load file.fav` でファイルの定義がセッションに追加される
4. `:paste` ... `:end` で複数行の関数定義が動作する
5. `repl_complete_prefix("List.")` が `List.map` 等を含む候補を返す

---

## 非対応（スコープ外）

- rustyline によるインタラクティブ矢印キー補完（インタラクティブ実行では接続するが、テストスコープ外）
- レコードフィールド補完（`row.` → フィールド一覧）— 型推論結果が必要のため後続バージョン
- `:doc` の MDX ファイルからの自動抽出 — 静的テーブルで代替
