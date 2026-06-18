# v17.5.0 — REPL 品質向上 実装計画

## 方針

AST / IR / VM の変更は不要。すべて `driver.rs` の REPL 関連関数に閉じた実装。

以下の優先順位で実装する：
1. テスト可能な純粋関数（`repl_doc_str`, `repl_complete_prefix`）を先に実装
2. `ReplSession` に `history` を追加
3. `:paste` / `:load` / `:save` のコマンドハンドラを追加
4. `cmd_repl` のディスパッチループを更新
5. `print_repl_help` を更新

rustyline は追加しない（インタラクティブ履歴・矢印キーは future work）。

---

## 実装ステップ

### Step 1: `ReplSession` に `history` を追加

```rust
struct ReplSession {
    definitions: String,
    def_names: Vec<String>,
    history: Vec<String>,   // ← 追加
}

impl ReplSession {
    fn new() -> Self {
        Self { definitions: String::new(), def_names: Vec::new(), history: Vec::new() }
    }
    fn add_history(&mut self, line: &str) {
        self.history.push(line.to_string());
    }
    fn print_history(&self) {
        if self.history.is_empty() {
            println!("(no history)");
        } else {
            for (i, line) in self.history.iter().enumerate() {
                println!("{}: {}", i + 1, line);
            }
        }
    }
}
```

### Step 2: `BUILTIN_DOCS` テーブル

`cmd_repl` の上に静的配列を追加：

```rust
const BUILTIN_DOCS: &[(&str, &str, &str)] = &[
    ("List.map",      "List.map(list: List<A>, fn: A -> B) -> List<B>",     "Apply fn to each element"),
    ("List.filter",   "List.filter(list: List<A>, fn: A -> Bool) -> List<A>", "Keep elements where fn is true"),
    ("List.length",   "List.length(list: List<A>) -> Int",                   "Return the number of elements"),
    ("List.group_by", "List.group_by(fn: A -> K, list: List<A>) -> Map<K, List<A>>", "Group by key"),
    ("List.sort_by",  "List.sort_by(list: List<A>, fn: A -> K) -> List<A>", "Sort by key function"),
    ("List.flat_map", "List.flat_map(list: List<A>, fn: A -> List<B>) -> List<B>", "Map then flatten"),
    ("List.fold",     "List.fold(list: List<A>, init: B, fn: B -> A -> B) -> B", "Reduce to a single value"),
    ("List.push",     "List.push(list: List<A>, elem: A) -> List<A>",        "Append element to end"),
    ("List.singleton","List.singleton(elem: A) -> List<A>",                   "Create single-element list"),
    ("List.empty",    "List.empty() -> List<A>",                              "Create empty list"),
    ("List.take",     "List.take(list: List<A>, n: Int) -> List<A>",          "Take first n elements"),
    ("List.drop",     "List.drop(list: List<A>, n: Int) -> List<A>",          "Drop first n elements"),
    ("List.head",     "List.head(list: List<A>) -> Result<A, String>",        "Get first element"),
    ("List.tail",     "List.tail(list: List<A>) -> Result<List<A>, String>",  "Get all but first"),
    ("List.zip",      "List.zip(a: List<A>, b: List<B>) -> List<Pair<A, B>>", "Zip two lists"),
    ("String.trim",   "String.trim(s: String) -> String",                     "Remove leading/trailing whitespace"),
    ("String.length", "String.length(s: String) -> Int",                      "Character count"),
    ("String.to_upper","String.to_upper(s: String) -> String",                "Convert to uppercase"),
    ("String.to_lower","String.to_lower(s: String) -> String",                "Convert to lowercase"),
    ("String.split",  "String.split(s: String, sep: String) -> List<String>", "Split by separator"),
    ("String.contains","String.contains(s: String, sub: String) -> Bool",     "Check for substring"),
    ("String.starts_with","String.starts_with(s: String, prefix: String) -> Bool","Prefix check"),
    ("Json.stringify","Json.stringify(val: A) -> Result<String, String>",     "Serialize to JSON"),
    ("Json.parse",    "Json.parse(s: String) -> Result<A, String>",           "Parse JSON string"),
    ("Map.empty",     "Map.empty() -> Map<K, V>",                             "Create empty map"),
    ("Map.insert",    "Map.insert(map: Map<K, V>, key: K, val: V) -> Map<K, V>", "Insert key-value pair"),
    ("Map.get",       "Map.get(map: Map<K, V>, key: K) -> Result<V, String>", "Look up by key"),
    ("Map.keys",      "Map.keys(map: Map<K, V>) -> List<K>",                  "All keys"),
    ("Map.values",    "Map.values(map: Map<K, V>) -> List<V>",                "All values"),
    ("Result.ok",     "Result.ok(val: A) -> Result<A, E>",                    "Wrap value in ok"),
    ("Result.err",    "Result.err(err: E) -> Result<A, E>",                   "Wrap error in err"),
    ("IO.println",    "IO.println(s: String) -> Unit",                        "Print line to stdout"),
];
```

### Step 3: `repl_doc_str` 関数

```rust
pub fn repl_doc_str(target: &str) -> Option<String> {
    for (name, sig, desc) in BUILTIN_DOCS {
        if *name == target {
            return Some(format!("{}\n  {}", sig, desc));
        }
    }
    None
}
```

`handle_doc_cmd` はこれを呼んで `println!` する：

```rust
fn handle_doc_cmd(target: &str) {
    match repl_doc_str(target) {
        Some(doc) => println!("{}", doc),
        None => println!("no documentation found for '{}'", target),
    }
}
```

### Step 4: `repl_complete_prefix` 関数

```rust
const REPL_COMMANDS: &[&str] = &[
    ":help", ":quit", ":q", ":reset", ":env", ":type ", ":doc ", ":load ", ":save", ":history", ":paste",
];

pub fn repl_complete_prefix(prefix: &str) -> Vec<String> {
    let mut result = Vec::new();
    // コマンド補完 (":" で始まる場合)
    if prefix.starts_with(':') {
        for cmd in REPL_COMMANDS {
            if cmd.starts_with(prefix) {
                result.push(cmd.to_string());
            }
        }
        return result;
    }
    // builtin 補完
    for (name, _, _) in BUILTIN_DOCS {
        if name.starts_with(prefix) {
            result.push(name.to_string());
        }
    }
    result
}
```

### Step 5: `handle_load_cmd` 関数

```rust
pub fn handle_load_cmd(path: &str, session: &mut ReplSession) {
    let src = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => { eprintln!("error: cannot read '{}': {}", path, e); return; }
    };
    // トップレベル定義を 1 つずつ抽出してセッションに追加
    // 単純な実装: ファイル全体を 1 定義ブロックとして追加
    let prev_count = session.def_names.len();
    // ファイルを型チェックして全定義を session に追加
    let merged = format!("{}\n{}\n", session.definitions, src);
    let errors = check_source_str(&merged);
    if errors.is_empty() {
        // 追加成功: ファイル内の定義名を抽出
        let added: Vec<String> = extract_top_level_names(&src);
        if added.is_empty() {
            println!("loaded: (no definitions found)");
        } else {
            println!("loaded: {}", added.join(", "));
        }
        for name in &added {
            session.def_names.push(name.clone());
        }
        if !session.definitions.is_empty() {
            session.definitions.push('\n');
        }
        session.definitions.push_str(&src);
    } else {
        for e in &errors {
            eprintln!("error: {}", e.message);
        }
    }
    let _ = prev_count; // suppress warning
}

fn extract_top_level_names(src: &str) -> Vec<String> {
    src.lines()
        .filter(|l| is_definition(l.trim()))
        .map(|l| extract_def_name(l.trim()))
        .collect()
}
```

### Step 6: `handle_paste_block` 関数

`:paste` モードでは `cmd_repl` が `:end` まで行を収集し、
それを `handle_paste_block` に渡す：

```rust
pub fn handle_paste_block(src: &str, session: &mut ReplSession) {
    let trimmed = src.trim();
    if is_definition(trimmed) {
        handle_definition(trimmed, session);
    } else {
        handle_expression(trimmed, session);
    }
}
```

`cmd_repl` でのディスパッチ：

```rust
":paste" => {
    let mut lines = Vec::new();
    loop {
        let mut l = String::new();
        if stdin.lock().read_line(&mut l).unwrap_or(0) == 0 { break; }
        let l = l.trim_end_matches(['\n', '\r']);
        if l.trim() == ":end" { break; }
        lines.push(l.to_string());
    }
    handle_paste_block(&lines.join("\n"), &mut session);
}
```

### Step 7: `:save` ハンドラ

```rust
fn handle_save_cmd(path: &str, session: &ReplSession) {
    match std::fs::write(path, &session.definitions) {
        Ok(_) => println!("saved {} definitions to {}", session.def_names.len(), path),
        Err(e) => eprintln!("error: cannot write '{}': {}", path, e),
    }
}
```

### Step 8: `cmd_repl` ディスパッチループ更新

```rust
pub fn cmd_repl() {
    // ... （既存の stdin/stdout 設定）
    println!("Favnir {} — type :help for commands", env!("CARGO_PKG_VERSION"));
    loop {
        // ... （プロンプト表示）
        let line = ...; // 入力読み取り
        session.add_history(line);
        match line {
            ":quit" | ":q" => break,
            ":reset" => session.reset(),
            ":help" | ":h" => print_repl_help(),
            ":env" => { /* 既存 */ }
            ":history" => session.print_history(),
            ":paste" => { /* Step 6 */ }
            _ if line.starts_with(":type ") => handle_type_cmd(line[6..].trim(), &session),
            _ if line.starts_with(":doc ")  => handle_doc_cmd(line[5..].trim()),
            _ if line.starts_with(":load ") => handle_load_cmd(line[6..].trim(), &mut session),
            _ if line.starts_with(":save ") => handle_save_cmd(line[6..].trim(), &session),
            _ if is_definition(line) => handle_definition(line, &mut session),
            _ => handle_expression(line, &session),
        }
    }
}
```

### Step 9: `print_repl_help` 更新

コマンド一覧に新コマンドを追加。

### Step 10: テスト追加 + バージョン更新

`v175000_tests` モジュールを `driver.rs` に追加（5件）。
`Cargo.toml` を `17.5.0` に更新。

---

## 実装順序まとめ

1. `ReplSession` に `history` 追加
2. `BUILTIN_DOCS` テーブル追加
3. `repl_doc_str` + `handle_doc_cmd` 追加
4. `repl_complete_prefix` 追加（`REPL_COMMANDS` テーブル）
5. `handle_load_cmd` + `extract_top_level_names` 追加
6. `handle_paste_block` 追加
7. `handle_save_cmd` 追加
8. `cmd_repl` ループ更新
9. `print_repl_help` 更新
10. テスト追加 + バージョン更新

---

## リスク・注意点

- **`:paste` の EOF 処理**: `:end` が来ない場合（Ctrl+D / stdin 終了）はそれまでの行を処理する。
- **`check_source_str` の公開性**: `handle_load_cmd` は `check_source_str` を使う。現在 `pub` かどうか要確認。
- **`extract_top_level_names` の精度**: 正規表現なしで `is_definition` + `extract_def_name` を流用するため、シングルライン定義のみ正確に抽出できる。マルチライン定義（`fn foo() {\n  ...\n}`）はファイル全体をまとめて 1 ブロックとして扱う。
- **`repl_complete_prefix` の公開性**: テストから呼べるよう `pub` にする。
- **バージョン文字列**: `cmd_repl` 内の `"Favnir v9.10.0"` ハードコードを `env!("CARGO_PKG_VERSION")` に変更する。
