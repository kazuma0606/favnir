# Favnir v9.10.0 Spec

Date: 2026-06-02
Theme: `fav repl` — インタラクティブ REPL

---

## 概要

v9.10.0 は対話的に Favnir を評価できる REPL（Read-Eval-Print Loop）を追加する。

```
$ fav repl
Favnir v9.10.0 — type :help for commands
> 1 + 2
3
> fn greet(name: String) -> String = "Hello, " + name
defined: greet
> greet("world")
"Hello, world!"
> :type List.first([1, 2, 3])
Option<Int>
> :quit
```

新規ユーザーのオンボーディング・探索的なパイプライン開発・型の即時確認に使う。
`.fav` ファイルを作らずに Favnir を試せる最初の入口になる。

---

## ユーザーインターフェース

### 起動

```
fav repl
```

### 入力の種類

| 入力 | 動作 |
|---|---|
| 式 (`1 + 2`、`List.map(...)` 等) | 評価して結果を表示 |
| 関数定義 (`fn f(x: T) -> U = ...`) | セッションに追加、`defined: f` を表示 |
| stage 定義 (`stage S: T -> U = ...`) | セッションに追加、`defined: S` を表示 |
| type 定義 (`type Foo = { ... }`) | セッションに追加、`defined: Foo` を表示 |
| `:help` | ヘルプを表示 |
| `:quit` / `:q` | REPL を終了 |
| `:reset` | セッションの定義をすべてクリア |
| `:env` | 現在のセッション定義を表示 |
| `:type <expr>` | 式の型を表示 |

### 表示形式

**数値・文字列・Bool**:
```
> 42
42
> "hello"
"hello"
> true
true
```

**リスト・レコード**:
```
> [1, 2, 3]
[1, 2, 3]
> { name: "Alice", age: 30 }
{name: "Alice", age: 30}
```

**Result / Option**:
```
> Result.ok(42)
Ok(42)
> Option.none()
None
```

**エラー**:
```
> 1 + "hello"
error: E0009: type mismatch
> undefined_fn()
error: E0007: undefined function 'undefined_fn'
```

---

## 実装方針

### アーキテクチャ

REPL のメインループは `driver.rs`（Rust）に実装する。
セルフホスト化は v9.10.0 のスコープ外（将来の改善余地として残す）。

```
fav repl
  └── main.rs: "repl" → cmd_repl()
        └── driver.rs: cmd_repl()
              ├── stdin readline (Rust 標準ライブラリ)
              ├── メタコマンド処理 (Rust)
              ├── 入力種別判定 (Rust)
              └── 評価: checker.fav → compiler.fav → VM
```

cli.fav は `CmdRepl` のルーティングのみ担当。

### セッション管理

セッション状態は `ReplSession` 構造体（Rust）で管理する。

```rust
struct ReplSession {
    definitions: String,   // 累積した fn/stage/type 定義のソース
    def_names: Vec<String>, // 定義済み名前一覧（:env 用）
}
```

### 式の評価

式 `expr` を次のソースにラップして Favnir pipeline でコンパイル・実行する:

```
<accumulated definitions>

fn main() -> Unit = IO.println(Debug.show_raw(expr))
```

`Debug.show_raw` は `Unknown` 型の値を人間が読める文字列に変換する新規 primitive。
内部実装は `format!("{}", vmvalue_display(v))` — JSON に近いが `"ok"(42)` → `Ok(42)` のように
Favnir らしい表現にする。

### 定義の検出

入力行が定義かどうかの判定（Rust 実装）:

```rust
fn is_definition(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("fn ")
        || trimmed.starts_with("public fn ")
        || trimmed.starts_with("stage ")
        || trimmed.starts_with("seq ")
        || trimmed.starts_with("type ")
        || trimmed.starts_with("effect ")
}
```

定義の場合はコンパイルエラーがなければセッションに追加、
エラーがあれば追加せず警告を表示。

### `:type` の実装

`:type expr` は以下の手順で型を推定する:

1. `fn _type_probe_() -> ___PROBE___ = <expr>` をセッション定義と合わせてコンパイル
2. checker.fav が E0009（戻り型不一致）を出す場合、エラーメッセージから推定型を抽出
   - `"E0009: ... inferred: Int"` → `Int`
3. 型が取得できない場合は `(unknown)` を表示

E0009 メッセージのフォーマットが安定していれば有効、そうでなければ best-effort 表示で可。

### エラー回復

コンパイルエラー・実行時エラーが発生してもセッションを継続する。
エラーがあった定義はセッションに追加しない。

---

## 新規 Rust 要素

### `Debug.show_raw` (vm.rs)

```rust
"Debug.show_raw" => {
    let v = args.into_iter().next()...;
    Ok(VMValue::Str(display_vmvalue(&v)))
}
```

`display_vmvalue` は `VMValue` を人間可読な文字列に変換する内部ヘルパー:
- `VMValue::Int(n)` → `"42"`
- `VMValue::Str(s)` → `"\"hello\""` (クォート付き)
- `VMValue::Bool(b)` → `"true"` / `"false"`
- `VMValue::List(xs)` → `"[1, 2, 3]"`
- `VMValue::Record(fields)` → `"{name: \"Alice\", age: 30}"`
- `VMValue::Variant(tag, Some(inner))` → `"Ok(42)"` / `"Some(1)"`
- `VMValue::Variant(tag, None)` → `"None"` / `"Unit"`

### `cmd_repl` (driver.rs)

```rust
pub fn cmd_repl() {
    let mut session = ReplSession::new();
    println!("Favnir v9.10.0 — type :help for commands");
    loop {
        print!("> ");
        stdout().flush().ok();
        let mut line = String::new();
        match stdin().read_line(&mut line) {
            Ok(0) | Err(_) => { println!(); break; } // EOF
            Ok(_) => {}
        }
        let line = line.trim_end_matches(['\n', '\r']).trim();
        if line.is_empty() { continue; }
        match line {
            ":quit" | ":q" => break,
            ":reset" => session.reset(),
            ":help" => print_repl_help(),
            ":env" => println!("{}", session.definitions),
            _ if line.starts_with(":type ") => {
                handle_type_cmd(&line[6..], &session);
            }
            _ => handle_repl_input(line, &mut session),
        }
    }
}
```

---

## 完了条件

| 条件 | 確認 |
|---|---|
| 式を評価して結果を表示できる | |
| `fn` / `stage` / `type` 定義がセッションに累積される | |
| 累積した定義を後続の式から参照できる | |
| `:quit` で正常終了する | |
| `:reset` で定義がクリアされる | |
| `:env` でセッション定義を確認できる | |
| `:type expr` で型を確認できる（best-effort） | |
| コンパイルエラー後もセッションが継続する | |
| EOF（Ctrl+D）で正常終了する | |
| `cargo test v9100` — 3 件以上通過 | |
| `cargo test checker_fav_wire_self_check` 通過 | |

---

## スコープ外（将来版へ延期）

- REPL ループの cli.fav セルフホスト化（現バージョンは driver.rs に実装）
- 補完（Tab キー）/ ヒストリ（↑↓キー）— rustyline 等の導入は v9.11.0 LSP と合わせて検討
- マルチライン入力（`fn f(x: Int) ->`で Enter → 続きを入力）
- `fav repl --db <url>` — DB 接続付き REPL
- `:load file.fav` — ファイルを読み込んでセッションに追加
