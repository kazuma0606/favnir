# Favnir v2.1.0 実装プラン

作成日: 2026-05-11

---

## Phase 0 — バージョン更新

```toml
# Cargo.toml
version = "2.1.0"
```

```rust
// src/main.rs
const HELP: &str = "fav v2.1.0 ...";
// fav new コマンドを HELP に追加
```

---

## Phase 1 — 標準ライブラリ補完

### 1-1. Math モジュール（`src/backend/vm.rs`）

`vm_call_builtin` に "Math.*" の分岐を追加する。

```rust
"Math.abs" => {
    match &args[0] {
        VMValue::Int(n) => Ok(VMValue::Int(n.abs())),
        _ => Err(runtime_err("Math.abs: expected Int")),
    }
}
"Math.abs_float" => {
    match &args[0] {
        VMValue::Float(f) => Ok(VMValue::Float(f.abs())),
        _ => Err(runtime_err("Math.abs_float: expected Float")),
    }
}
"Math.min" => {
    // args[0]: Int, args[1]: Int
    match (&args[0], &args[1]) {
        (VMValue::Int(a), VMValue::Int(b)) => Ok(VMValue::Int(*a.min(b))),
        _ => Err(runtime_err("Math.min: expected Int Int")),
    }
}
"Math.max" => { /* 同様 */ }
"Math.min_float" | "Math.max_float" => { /* Float 版 */ }
"Math.clamp" => {
    // args: value, lo, hi
    match (&args[0], &args[1], &args[2]) {
        (VMValue::Int(v), VMValue::Int(lo), VMValue::Int(hi)) =>
            Ok(VMValue::Int((*v).max(*lo).min(*hi))),
        _ => Err(runtime_err("Math.clamp: expected Int Int Int")),
    }
}
"Math.pow" => {
    match (&args[0], &args[1]) {
        (VMValue::Int(base), VMValue::Int(exp)) =>
            Ok(VMValue::Int(base.pow(*exp as u32))),
        _ => Err(runtime_err("Math.pow: expected Int Int")),
    }
}
"Math.pow_float" => { /* f64::powf */ }
"Math.sqrt"  => { /* f64::sqrt */ }
"Math.floor" => { /* f64::floor as i64 */ }
"Math.ceil"  => { /* f64::ceil as i64 */ }
"Math.round" => { /* f64::round as i64 */ }
"Math.pi"    => Ok(VMValue::Float(std::f64::consts::PI)),
"Math.e"     => Ok(VMValue::Float(std::f64::consts::E)),
```

**定数（`Math.pi` / `Math.e`）の扱い**:
引数なしで呼ばれるため `args.len() == 0` のケースとして処理する。
チェッカー側では `Math.pi` / `Math.e` を `Float` 型の定数フィールドとして登録する。

### 1-2. Math モジュール（`src/middle/compiler.rs`）

ビルトイン関数テーブルに Math を登録:

```rust
// builtin_fn_types または同等のテーブルに追加
("Math", "abs",       Fn(vec![Int], Int)),
("Math", "abs_float", Fn(vec![Float], Float)),
("Math", "min",       Fn(vec![Int, Int], Int)),
("Math", "max",       Fn(vec![Int, Int], Int)),
("Math", "min_float", Fn(vec![Float, Float], Float)),
("Math", "max_float", Fn(vec![Float, Float], Float)),
("Math", "clamp",     Fn(vec![Int, Int, Int], Int)),
("Math", "pow",       Fn(vec![Int, Int], Int)),
("Math", "pow_float", Fn(vec![Float, Float], Float)),
("Math", "sqrt",      Fn(vec![Float], Float)),
("Math", "floor",     Fn(vec![Float], Int)),
("Math", "ceil",      Fn(vec![Float], Int)),
("Math", "round",     Fn(vec![Float], Int)),
// 定数: 引数なし
("Math", "pi",        Float),
("Math", "e",         Float),
```

### 1-3. List 補完（`src/backend/vm.rs`）

```rust
"List.unique" => {
    // 重複除去（初出順保持）
    // HashSet で seen を管理しながら fold
    let list = as_list(&args[0])?;
    let mut seen = std::collections::HashSet::new();
    let mut result = vec![];
    for item in list {
        let key = vm_value_to_string(&item); // 比較キー
        if seen.insert(key) {
            result.push(item);
        }
    }
    Ok(VMValue::List(result))
}
"List.flatten" => {
    // List<List<T>> → List<T>
    let outer = as_list(&args[0])?;
    let mut result = vec![];
    for inner in outer {
        result.extend(as_list(&inner)?);
    }
    Ok(VMValue::List(result))
}
"List.chunk" => {
    let list = as_list(&args[0])?;
    let n = as_int(&args[1])? as usize;
    let chunks: Vec<VMValue> = list
        .chunks(n)
        .map(|c| VMValue::List(c.to_vec()))
        .collect();
    Ok(VMValue::List(chunks))
}
"List.sum"       => { /* fold + Int 加算 */ }
"List.sum_float" => { /* fold + Float 加算 */ }
"List.min"       => { /* fold with Option<Int> */ }
"List.max"       => { /* fold with Option<Int> */ }
"List.count"     => {
    // args[0]: List<T>, args[1]: T -> Bool の関数
    // call_fn(pred, item) == Bool(true) の個数
}
```

### 1-4. String 補完（`src/backend/vm.rs`）

```rust
"String.index_of" => {
    let s   = as_str(&args[0])?;
    let pat = as_str(&args[1])?;
    match s.find(pat.as_str()) {
        Some(i) => Ok(make_some(VMValue::Int(i as i64))),
        None    => Ok(make_none()),
    }
}
"String.pad_left" => {
    let s     = as_str(&args[0])?;
    let width = as_int(&args[1])? as usize;
    let fill  = as_str(&args[2])?;
    // fill を繰り返して左から埋める
}
"String.pad_right" => { /* 右から埋める */ }
"String.reverse"   => {
    Ok(VMValue::Str(as_str(&args[0])?.chars().rev().collect()))
}
"String.lines" => {
    // \r\n / \n で分割
    let s = as_str(&args[0])?;
    let lines: Vec<VMValue> = s
        .lines()
        .map(|l| VMValue::Str(l.to_string()))
        .collect();
    Ok(VMValue::List(lines))
}
"String.words" => {
    let s = as_str(&args[0])?;
    let words: Vec<VMValue> = s
        .split_whitespace()
        .map(|w| VMValue::Str(w.to_string()))
        .collect();
    Ok(VMValue::List(words))
}
```

### 1-5. IO.read_line（`src/backend/vm.rs`）

```rust
"IO.read_line" => {
    // SUPPRESS_IO_OUTPUT フラグが立っている場合（fav test）は空文字列を返す
    if is_io_suppressed() {
        return Ok(VMValue::Str(String::new()));
    }
    use std::io::BufRead;
    let mut line = String::new();
    std::io::stdin().lock().read_line(&mut line)?;
    // 末尾の \n / \r\n を除去
    if line.ends_with('\n') { line.pop(); }
    if line.ends_with('\r') { line.pop(); }
    Ok(VMValue::Str(line))
}
```

---

## Phase 2 — 論理演算子

### 2-1. レキサー（`src/frontend/lexer.rs`）

```rust
// 2文字トークンの先読みパターンに追加
'&' => {
    if self.peek_char() == '&' {
        self.advance();
        Token::new(TokenKind::AmpAmp, span)
    } else {
        // 単独 '&' は現状未定義 → エラーまたは将来用
        Token::new(TokenKind::Unknown, span)
    }
}
'|' => {
    if self.peek_char() == '|' {
        self.advance();
        Token::new(TokenKind::PipePipe, span)
    } else {
        // 既存の '|' トークン（クロージャパラメータ区切り）
        Token::new(TokenKind::Pipe, span)
    }
}
```

`TokenKind` に追加:
```rust
AmpAmp,   // &&
PipePipe, // ||
```

### 2-2. AST（`src/frontend/ast.rs`）

```rust
pub enum BinOp {
    // 既存 ...
    And,  // &&
    Or,   // ||
}
```

### 2-3. パーサー（`src/frontend/parser.rs`）

優先順位レベルに追加（既存レベルを参考に挿入）:

```rust
fn get_precedence(op: &TokenKind) -> Option<u8> {
    match op {
        // 既存
        TokenKind::PipePipe   => Some(PREC_OR),    // ?? より低い
        TokenKind::AmpAmp     => Some(PREC_AND),   // || より低い
        TokenKind::EqEq | TokenKind::BangEq
        | TokenKind::Lt | TokenKind::Gt
        | TokenKind::LtEq | TokenKind::GtEq => Some(PREC_CMP),  // && より低い
        // ...
    }
}

// BinOp への変換
TokenKind::AmpAmp  => BinOp::And,
TokenKind::PipePipe => BinOp::Or,
```

### 2-4. 型チェッカー（`src/middle/checker.rs`）

```rust
BinOp::And | BinOp::Or => {
    let lt = self.check_expr(lhs)?;
    let rt = self.check_expr(rhs)?;
    if lt != Type::Bool {
        return Err(TypeError::new(
            if op == BinOp::And { "E070" } else { "E071" },
            "left operand of && must be Bool",
            lhs.span,
        ));
    }
    if rt != Type::Bool {
        return Err(TypeError::new(
            if op == BinOp::And { "E070" } else { "E071" },
            "right operand of || must be Bool",
            rhs.span,
        ));
    }
    Ok(Type::Bool)
}
```

### 2-5. IR（`src/backend/ir.rs`）

```rust
pub enum IRBinOp {
    // 既存 ...
    And,
    Or,
}
```

### 2-6. コード生成（`src/backend/codegen.rs`）

```rust
IRBinOp::And => self.emit(Opcode::And),
IRBinOp::Or  => self.emit(Opcode::Or),
```

### 2-7. opcode（`src/backend/opcode.rs` または同等）

```rust
And = 0x2A,
Or  = 0x2B,
```

> **注**: 0x2A / 0x2B が既存の opcode と衝突しないか実装前に確認すること。
> 衝突する場合は空きスロット（0x55, 0x56 等）を使用する。

### 2-8. VM（`src/backend/vm.rs`）

```rust
Opcode::And => {
    let r = self.pop_bool()?;
    let l = self.pop_bool()?;
    self.push(VMValue::Bool(l && r));
}
Opcode::Or => {
    let r = self.pop_bool()?;
    let l = self.pop_bool()?;
    self.push(VMValue::Bool(l || r));
}
```

---

## Phase 3 — `fav new` コマンド

### 3-1. ドライバー（`src/driver.rs`）

```rust
pub fn cmd_new(name: &str, template: &str) -> Result<(), Box<dyn std::error::Error>> {
    let base = std::path::Path::new(name);
    if base.exists() {
        return Err(format!("directory '{}' already exists", name).into());
    }

    match template {
        "script"   => create_script_project(name)?,
        "pipeline" => create_pipeline_project(name)?,
        "lib"      => create_lib_project(name)?,
        other      => return Err(format!("unknown template: {}", other).into()),
    }

    println!("Created project '{}' (template: {})", name, template);
    println!("  cd {}", name);
    println!("  fav run src/main.fav");
    Ok(())
}

fn write_file(path: &std::path::Path, content: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, content)
}

fn create_script_project(name: &str) -> std::io::Result<()> {
    let base = std::path::Path::new(name);
    write_file(&base.join("fav.toml"), &fav_toml(name))?;
    write_file(&base.join("src/main.fav"), SCRIPT_MAIN_FAV)?;
    Ok(())
}

fn fav_toml(name: &str) -> String {
    format!(
        "[project]\nname    = \"{}\"\nversion = \"0.1.0\"\nedition = \"2026\"\nsrc     = \"src\"\n",
        name
    )
}

const SCRIPT_MAIN_FAV: &str = r#"public fn main() -> Unit !Io {
    IO.println(greet("world"))
}

fn greet(name: String) -> String {
    $"Hello {name}!"
}
"#;

// create_pipeline_project / create_lib_project も同様に実装
```

### 3-2. CLI（`src/main.rs`）

```rust
"new" => {
    let mut template = "script".to_string();
    let mut name: Option<String> = None;
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--template" => { i += 1; template = args[i].clone(); }
            arg          => name = Some(arg.to_string()),
        }
        i += 1;
    }
    match name {
        Some(n) => driver::cmd_new(&n, &template)?,
        None    => eprintln!("Usage: fav new <name> [--template script|pipeline|lib]"),
    }
}
```

---

## Phase 4 — CLI ウェルカム画面

### 4-1. `Cargo.toml` 依存追加

```toml
[dependencies]
viuer           = "0.7"
supports-color  = "3"
```

### 4-2. `src/main.rs`

```rust
fn print_welcome() {
    let dragon_img = include_bytes!("../../versions/favnir.png");

    // NO_COLOR チェック
    if std::env::var("NO_COLOR").is_ok() {
        eprintln!("🐉  Favnir {} — The pipeline-first language", env!("CARGO_PKG_VERSION"));
    } else {
        // viuer でターミナルに画像表示
        let conf = viuer::Config {
            width: Some(12),
            height: Some(6),
            ..Default::default()
        };
        let img = image::load_from_memory(dragon_img).unwrap_or_default();
        let _ = viuer::print(&img, &conf);
        eprintln!("  Favnir {} — The pipeline-first language\n", env!("CARGO_PKG_VERSION"));
    }

    eprintln!(
        "  fav run <file>          Run a .fav file\n\
         \  fav check <file>        Type-check without running\n\
         \  fav test <file>         Run tests\n\
         \  fav new <name>          Create a new project\n\
         \  fav fmt <file>          Format source code\n\
         \  fav lint <file>         Run linter\n\
         \  fav bench <file>        Run benchmarks\n\
         \  fav migrate <file>      Migrate v1.x code to v2.x\n\
         \  fav explain <file>      Show pipeline structure\n\
         \  fav watch <file>        Watch and re-run on change\n\
         \n\
         \  fav help <command>      Show detailed help\n"
    );
}

// main() の引数なし分岐:
// args.len() == 1 || (args.len() == 2 && args[1] == "--help") => print_welcome()
```

---

## Phase 5 — テスト・ドキュメント

### 5-1. テスト追加場所

**`src/backend/vm_stdlib_tests.rs`**（または既存の stdlib テストファイル）:
- Math 全関数のテスト（約 15 件）
- List 補完テスト（約 12 件）
- String 補完テスト（約 10 件）
- IO.read_line のテスト（suppress モードで空文字列を返す確認）

**`src/frontend/parser.rs`**:
- `logical_and_parses`: `true && false` がパースされる
- `logical_or_parses`: `false || true` がパースされる
- `and_precedence_over_comparison`: `1 == 1 && 2 == 2` が `(1 == 1) && (2 == 2)` として解析される

**`src/middle/checker.rs`**:
- `logical_and_bool_bool_ok`: `true && false` が型チェックを通る
- `logical_or_bool_bool_ok`: `false || true` が型チェックを通る
- `logical_and_non_bool_e070`: `1 && true` が E070
- `logical_or_non_bool_e071`: `true || "x"` が E071

**`src/driver.rs`**:
- `fav_new_creates_script_project`: `cmd_new("test_proj", "script")` でファイルが生成される
- `fav_new_creates_pipeline_project`: pipeline テンプレートが生成される
- `fav_new_fails_if_dir_exists`: 既存ディレクトリ名でエラーになる

### 5-2. ファイル変更一覧

| ファイル | 変更内容 |
|---|---|
| `Cargo.toml` | version = "2.1.0", viuer/supports-color 依存追加 |
| `src/main.rs` | HELP v2.1.0, new コマンド, print_welcome() |
| `src/frontend/lexer.rs` | AmpAmp / PipePipe トークン追加 |
| `src/frontend/ast.rs` | BinOp::And / Or 追加 |
| `src/frontend/parser.rs` | && / || の優先順位テーブル追加 |
| `src/middle/checker.rs` | BinOp::And/Or の型検査, E070/E071 |
| `src/middle/compiler.rs` | Math/List/String/IO ビルトイン登録追加, IRBinOp::And/Or |
| `src/backend/ir.rs` | IRBinOp::And / Or |
| `src/backend/codegen.rs` | And/Or opcode emit |
| `src/backend/opcode.rs` | And = 0x2A, Or = 0x2B |
| `src/backend/vm.rs` | Math/List/String/IO.read_line 実装, And/Or opcode ハンドラ |
| `src/backend/vm_stdlib_tests.rs` | 新規テスト追加 |
| `src/driver.rs` | cmd_new, create_*_project |
| `versions/v2.1.0/langspec.md` | NEW: v2.1.0 言語仕様書 |

---

## 実装上の注意事項

### `List.unique` の比較

`VMValue` は `Eq` / `Hash` を実装していない可能性がある。
文字列表現（`Debug.show` 相当）でキー化する方法か、`VMValue` に `PartialEq` を derive する方法を選択する。

### `List.count` の高階関数

`args[1]` が `VMValue::Closure` / `VMValue::FnPtr` の場合に `call_fn` を呼び出す必要がある。
既存の `List.filter` / `List.map` の実装を参考にする。

### opcode 衝突確認

`And = 0x2A` / `Or = 0x2B` の値は `src/backend/opcode.rs`（または同等ファイル）の
既存定義と照合してから使用すること。衝突する場合は末尾の空きに割り当てる。

### `viuer` の image クレート依存

`viuer` は `image` クレートに依存する。`Cargo.toml` に `image` の明示追加が必要な場合は追加する。
ビルドサイズの増大を許容する（バイナリに PNG を埋め込むため）。
