# Favnir v1.6.0 実装計画 — 言語表現力 + 開発ループ改善

作成日: 2026-05-08

> **実装順序**: Phase 0 → 1 → 2 → 3 → 4 → 5
> Phase 1/2 は AST 変更を伴うため先に完成させる。
> Phase 3/4 は driver.rs / main.rs のみで依存が少ない。

---

## Phase 0 — バージョン更新

```toml
# Cargo.toml
version = "1.6.0"

# 新規依存追加 (Phase 4)
notify = { version = "6", default-features = false, features = ["macos_kqueue"] }
```

```rust
// main.rs HELP
const HELP: &str = r#"
Favnir Compiler v1.6.0
...
  watch [--cmd <check|test|run>] [file]
                  Watch .fav files and re-run command on change.
"#;
```

---

## Phase 1 — 文字列補間

### 1-1. AST の変更（`ast.rs`）

```rust
// Expr に追加
pub enum Expr {
    // ... 既存 ...
    FString(Vec<FStringPart>, Span),
}

#[derive(Debug, Clone)]
pub enum FStringPart {
    Lit(String),
    Expr(Box<Expr>),
}
```

### 1-2. 字句解析の変更（`lexer.rs`）

```rust
// TokenKind に追加
FStringRaw(String),  // $"..." の中身 raw 文字列（{ } を含む生テキスト）

// lex_token に追加: '$' を読んだら次が '"' か確認
'$' if self.peek() == Some('"') => {
    self.advance(); // '"' を消費
    let raw = self.lex_fstring_raw()?;
    Token { kind: TokenKind::FStringRaw(raw), span: ... }
}

fn lex_fstring_raw(&mut self) -> Result<String, LexError> {
    let mut raw = String::new();
    let mut depth = 0usize;

    loop {
        match self.current() {
            Some('"') if depth == 0 => {
                self.advance();
                return Ok(raw);
            }
            Some('\\') => {
                self.advance();
                match self.current() {
                    Some('{') => { raw.push('{'); self.advance(); }
                    Some('n') => { raw.push('\n'); self.advance(); }
                    Some('t') => { raw.push('\t'); self.advance(); }
                    Some('"') => { raw.push('"');  self.advance(); }
                    Some('\\') => { raw.push('\\'); self.advance(); }
                    other => {
                        raw.push('\\');
                        if let Some(c) = other { raw.push(c); self.advance(); }
                    }
                }
            }
            Some('{') => { depth += 1; raw.push('{'); self.advance(); }
            Some('}') => {
                if depth > 0 { depth -= 1; }
                raw.push('}');
                self.advance();
            }
            Some(c) => { raw.push(c); self.advance(); }
            None => return Err(LexError::UnterminatedFString),
        }
    }
}
```

### 1-3. パーサーの変更（`parser.rs`）

```rust
// parse_primary に追加
TokenKind::FStringRaw(raw) => {
    let span = self.current_span();
    let raw = raw.clone();
    self.advance();
    parse_fstring_parts(&raw, span, self)
}

// FString のパート分割
fn parse_fstring_parts(raw: &str, base_span: Span, parser: &mut Parser) -> Result<Expr, ParseError> {
    let mut parts: Vec<FStringPart> = Vec::new();
    let mut chars = raw.char_indices().peekable();
    let mut lit_buf = String::new();
    let mut depth = 0usize;
    let mut expr_buf = String::new();
    let mut in_expr = false;

    while let Some((_, c)) = chars.next() {
        match c {
            '{' if !in_expr && depth == 0 => {
                if !lit_buf.is_empty() {
                    parts.push(FStringPart::Lit(std::mem::take(&mut lit_buf)));
                }
                in_expr = true;
                depth = 0;
            }
            '{' if in_expr => {
                depth += 1;
                expr_buf.push('{');
            }
            '}' if in_expr && depth > 0 => {
                depth -= 1;
                expr_buf.push('}');
            }
            '}' if in_expr && depth == 0 => {
                // expr_buf をパース
                let inner_expr = Parser::parse_str_expr(&expr_buf, base_span.file)?;
                parts.push(FStringPart::Expr(Box::new(inner_expr)));
                expr_buf.clear();
                in_expr = false;
            }
            c if in_expr => expr_buf.push(c),
            c => lit_buf.push(c),
        }
    }
    if !lit_buf.is_empty() {
        parts.push(FStringPart::Lit(lit_buf));
    }

    Ok(Expr::FString(parts, base_span))
}
```

### 1-4. チェッカーの変更（`checker.rs`）

```rust
Expr::FString(parts, span) => {
    for part in parts {
        match part {
            FStringPart::Lit(_) => {}
            FStringPart::Expr(inner) => {
                let ty = self.check_expr(inner);
                match &ty {
                    Type::String | Type::Int | Type::Float | Type::Bool => {}
                    Type::Unknown => {}
                    Type::Named(_, _) => {
                        if !self.has_show_impl(&ty) {
                            self.errors.push(CheckError::new(
                                "E054",
                                format!("type `{}` does not implement Show; cannot use in string interpolation", ty.display()),
                                *span,
                            ));
                        }
                    }
                    _ => {}
                }
                // FString 内の $"..." は E053
                if matches!(inner.as_ref(), Expr::FString(..)) {
                    self.errors.push(CheckError::new(
                        "E053",
                        "nested string interpolation is not supported",
                        *span,
                    ));
                }
            }
        }
    }
    Type::String
}
```

### 1-5. コンパイラの変更（`compiler.rs`）

```rust
// compile_expr の FString ケース
Expr::FString(parts, _) => {
    // 各パートを IRExpr に変換し、文字列連結で結合
    let mut acc: Option<IRExpr> = None;
    for part in parts {
        let part_ir = match part {
            FStringPart::Lit(s) => IRExpr::Lit(Literal::Str(s.clone()), Type::String),
            FStringPart::Expr(inner) => {
                let inner_ir = self.compile_expr(inner)?;
                let inner_ty = inner_ir.ty();
                match inner_ty {
                    Type::String => inner_ir,
                    // 自動 Debug.show 適用
                    _ => IRExpr::Call(
                        Box::new(IRExpr::Global(
                            ctx.global_idx("Debug.show").unwrap_or(u16::MAX),
                            Type::Unknown,
                        )),
                        vec![inner_ir],
                        Type::String,
                    ),
                }
            }
        };
        acc = Some(match acc {
            None => part_ir,
            Some(prev) => IRExpr::BinOp(
                "++".into(),
                Box::new(prev),
                Box::new(part_ir),
                Type::String,
            ),
        });
    }
    acc.unwrap_or_else(|| IRExpr::Lit(Literal::Str(String::new()), Type::String))
}
```

---

## Phase 2 — レコード分解パターン

### 2-1. AST の変更（`ast.rs`）

```rust
// Pattern に追加
Pattern::Record(Vec<RecordPatternField>, Span),

#[derive(Debug, Clone)]
pub struct RecordPatternField {
    pub field:   String,
    pub pattern: Option<Box<Pattern>>,  // None = pun
    pub span:    Span,
}
```

### 2-2. パーサーの変更（`parser.rs`）

```rust
// parse_pattern に追加（TokenKind::LBrace を検出）
TokenKind::LBrace => {
    let start = self.current_span();
    self.advance(); // '{' を消費
    let mut fields = vec![];
    while !self.check(&TokenKind::RBrace) {
        let field_span = self.current_span();
        let field_name = self.expect_ident("field name")?;
        let sub_pat = if self.eat(&TokenKind::Colon) {
            Some(Box::new(self.parse_pattern()?))
        } else {
            None  // pun: フィールド名を変数名として使う
        };
        fields.push(RecordPatternField {
            field: field_name,
            pattern: sub_pat,
            span: field_span,
        });
        if !self.eat(&TokenKind::Comma) { break; }
    }
    self.expect(&TokenKind::RBrace)?;
    let end = self.prev_span();
    Ok(Pattern::Record(fields, start.merge(end)))
}
```

### 2-3. チェッカーの変更（`checker.rs`）

```rust
Pattern::Record(fields, span) => {
    // スクルーティニーの型取得
    let record_ty = scrutinee_ty.clone();
    let type_def = match &record_ty {
        Type::Named(name, _) => self.type_registry.get(name).cloned(),
        _ => None,
    };
    let record_fields: HashMap<String, Type> = match &type_def {
        Some(TypeDef { body: TypeBody::Record(fs), .. }) => {
            fs.iter().map(|f| (f.name.clone(), self.resolve_type_expr(&f.ty))).collect()
        }
        _ => {
            self.errors.push(CheckError::new("E055",
                format!("record destructuring pattern requires a record type, got `{}`", record_ty.display()),
                *span));
            return record_ty;
        }
    };
    for field_pat in fields {
        let field_ty = match record_fields.get(&field_pat.field) {
            Some(t) => t.clone(),
            None => {
                self.errors.push(CheckError::new("E056",
                    format!("field `{}` does not exist in record type `{}`", field_pat.field, record_ty.display()),
                    field_pat.span));
                Type::Unknown
            }
        };
        match &field_pat.pattern {
            None => {
                // pun: フィールド名を変数として環境に追加
                self.env.define(field_pat.field.clone(), field_ty);
            }
            Some(sub_pat) => {
                self.check_pattern(sub_pat, &field_ty);
            }
        }
    }
    record_ty
}
```

### 2-4. IR の変更（`ir.rs`）

```rust
// IRPattern に追加
IRPattern::Record(Vec<(String, IRPattern)>),
```

### 2-5. コンパイラの変更（`compiler.rs`）

```rust
Pattern::Record(fields, _) => {
    let compiled: Vec<(String, IRPattern)> = fields.iter().map(|f| {
        let sub = match &f.pattern {
            None => IRPattern::Bind(f.field.clone()),  // pun
            Some(p) => self.compile_pattern(p),
        };
        (f.field.clone(), sub)
    }).collect();
    IRPattern::Record(compiled)
}
```

### 2-6. VM の変更（`vm.rs`）

```rust
// match_pattern に追加
IRPattern::Record(fields) => {
    match value {
        VMValue::Record(map) => {
            for (field, sub_pat) in fields {
                let field_val = match map.get(field) {
                    Some(v) => v.clone(),
                    None => return false,
                };
                if !match_pattern(sub_pat, &field_val, env) {
                    return false;
                }
            }
            true
        }
        _ => false,
    }
}
```

---

## Phase 3 — `fav test` 強化

### 3-1. `--filter` フラグの追加（`driver.rs`）

```rust
pub fn cmd_test(file: Option<&str>, filter: Option<&str>, no_capture: bool) {
    // 既存の test 実行ロジックに filter を追加
    let tests = collect_tests(&program);
    let matched: Vec<_> = tests.iter().filter(|t| {
        match filter {
            None => true,
            Some(f) => f.split(',').any(|pat| t.description.contains(pat.trim())),
        }
    }).collect();
    // ...
}
```

### 3-2. テスト統計の改善

```rust
struct TestResult {
    description: String,
    passed: bool,
    error_msg: Option<String>,
    elapsed_ms: f64,
}

fn format_test_results(results: &[TestResult], filtered: usize) -> String {
    let passed  = results.iter().filter(|r| r.passed).count();
    let failed  = results.iter().filter(|r| !r.passed).count();
    let total   = results.len();
    let elapsed = results.iter().map(|r| r.elapsed_ms).sum::<f64>();

    let mut out = String::new();
    for r in results {
        if r.passed {
            out.push_str(&format!("  PASS  {}  ({:.1}ms)\n", r.description, r.elapsed_ms));
        } else {
            out.push_str(&format!("  FAIL  {}\n", r.description));
            if let Some(msg) = &r.error_msg {
                for line in msg.lines() {
                    out.push_str(&format!("        {}\n", line));
                }
            }
        }
    }
    out.push_str(&format!(
        "\ntest result: {} passed; {} failed; {} filtered; finished in {:.1}ms\n",
        passed, failed, filtered, elapsed
    ));
    out
}
```

### 3-3. `assert_matches` ビルトインの追加

```rust
// ast.rs Expr に追加
Expr::AssertMatches(Box<Expr>, Box<Pattern>, Span),

// parser.rs
// "assert_matches" キーワード → parse_assert_matches
fn parse_assert_matches(&mut self) -> Result<Expr, ParseError> {
    let span = self.current_span();
    self.expect(&TokenKind::LParen)?;
    let expr = self.parse_expr()?;
    self.expect(&TokenKind::Comma)?;
    let pattern = self.parse_pattern()?;
    self.expect(&TokenKind::RParen)?;
    Ok(Expr::AssertMatches(Box::new(expr), Box::new(pattern), span))
}

// checker.rs
Expr::AssertMatches(expr, pattern, span) => {
    let expr_ty = self.check_expr(expr);
    self.check_pattern(pattern, &expr_ty);
    Type::Unit
}

// compiler.rs / vm.rs
// assert_matches(expr, pattern) →
//   bind val <- expr;
//   match val { pattern -> () | _ -> runtime_error("assert_matches failed") }
```

### 3-4. `main.rs` の変更

```rust
Some("test") => {
    let mut filter: Option<&str> = None;
    let mut no_capture = false;
    // ...
    "--filter" => filter = Some(&args[i+1]),
    "--no-capture" => no_capture = true,
    // ...
    cmd_test(file, filter, no_capture);
}
```

---

## Phase 4 — `fav watch`

### 4-1. `driver.rs` の変更

```rust
pub fn cmd_watch(file: Option<&str>, cmd: &str) {
    use notify::{RecommendedWatcher, RecursiveMode, Watcher};
    use std::sync::mpsc;
    use std::time::Duration;

    // 初回実行
    println!("[watch] starting...");
    run_watch_cmd(file, cmd);

    // 監視対象のパスを収集
    let watch_paths = collect_watch_paths(file);
    println!("[watch] watching {} files for changes...", watch_paths.len());

    let (tx, rx) = mpsc::channel();
    let mut watcher: RecommendedWatcher = notify::recommended_watcher(move |res| {
        let _ = tx.send(res);
    }).expect("watcher init failed");

    for path in &watch_paths {
        if path.is_dir() {
            watcher.watch(path, RecursiveMode::Recursive).ok();
        } else {
            watcher.watch(path, RecursiveMode::NonRecursive).ok();
        }
    }

    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                // 変更イベントのみ反応（Create/Modify/Remove）
                use notify::EventKind::*;
                if matches!(event.kind, Create(_) | Modify(_) | Remove(_)) {
                    // デバウンス
                    std::thread::sleep(Duration::from_millis(80));
                    while rx.try_recv().is_ok() {}

                    // ターミナルクリア
                    print!("\x1b[2J\x1b[H");
                    let changed = event.paths.first()
                        .map(|p| p.display().to_string())
                        .unwrap_or_default();
                    println!("[watch] changed: {}", changed);
                    run_watch_cmd(file, cmd);
                    println!("[watch] watching {} files for changes...", watch_paths.len());
                }
            }
            Err(_) | Ok(Err(_)) => break,
        }
    }
}

fn collect_watch_paths(file: Option<&str>) -> Vec<std::path::PathBuf> {
    if let Some(f) = file {
        return vec![std::path::PathBuf::from(f)];
    }
    // fav.toml のあるプロジェクトルートから .fav ファイルを収集
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    if let Some(root) = crate::toml::FavToml::find_root(&cwd) {
        let toml = crate::toml::FavToml::load(&root).unwrap_or_default();
        crate::driver::collect_fav_files(&toml.src_dir(&root))
    } else {
        // fav.toml なし: カレントディレクトリの .fav を収集
        std::fs::read_dir(&cwd)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map(|e| e == "fav").unwrap_or(false))
            .collect()
    }
}

fn run_watch_cmd(file: Option<&str>, cmd: &str) {
    for c in cmd.split(',') {
        match c.trim() {
            "check" => crate::driver::cmd_check(file.as_ref().map(|f| f)),
            "test"  => crate::driver::cmd_test(file, None, false),
            "run"   => crate::driver::cmd_run(file, false, None),
            other   => eprintln!("[watch] unknown command: {}", other),
        }
    }
}
```

### 4-2. `main.rs` の変更

```rust
// use 宣言に追加
use driver::cmd_watch;

// コマンドルーティングに追加
Some("watch") => {
    let mut cmd_str = String::from("check");
    let mut file: Option<&str> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--cmd" => {
                cmd_str = args.get(i + 1).unwrap_or_else(|| {
                    eprintln!("error: --cmd requires a value");
                    process::exit(1);
                }).clone();
                i += 2;
            }
            f if !f.starts_with('-') => {
                file = Some(f);
                i += 1;
            }
            _ => { i += 1; }
        }
    }
    cmd_watch(file, &cmd_str);
}
```

---

## Phase 5 — テスト・ドキュメント

### example ファイル

```
// examples/fstring_demo.fav
type User = {
    name: String
    age:  Int
}

public fn greet(user: User) -> String {
    $"Hello, {user.name}! You are {user.age} years old."
}

public fn main() -> String !Io {
    bind user <- User { name: "Alice"  age: 30 };
    bind msg <- greet(user);
    IO.println(msg);
    msg
}
```

```
// examples/record_match.fav
type Shape = Circle(Int) | Rect(Int, Int)

type Point = {
    x: Int
    y: Int
}

fn describe_point(p: Point) -> String {
    match p {
        { x, y } if x == 0 && y == 0 -> "origin"
        { x: 0, y } -> $"on y-axis at {y}"
        { x, y: 0 } -> $"on x-axis at {x}"
        { x, y }    -> $"at ({x}, {y})"
    }
}

public fn main() -> String {
    describe_point(Point { x: 0  y: 5 })
}
```

```
// examples/fstring_demo.test.fav
test "greet returns correct string" {
    bind user <- User { name: "Bob"  age: 25 };
    assert_eq(greet(user), "Hello, Bob! You are 25 years old.")
}

test "record match origin" {
    assert_eq(describe_point(Point { x: 0  y: 0 }), "origin")
}

test "assert_matches some" {
    bind result <- Option.some(42);
    assert_matches(result, some(_))
}
```

### テストの実装例

```rust
// driver.rs テスト

#[test]
fn fstring_simple_parse() {
    let program = Parser::parse_str(
        r#"public fn main() -> String { $"Hello {name}!" }"#,
        "fstring.fav"
    ).expect("parse");
    assert!(program.items.iter().any(|item| {
        // FString が含まれる FnDef を探す
        matches!(item, ast::Item::FnDef(_))
    }));
}

#[test]
fn fstring_exec_correct_output() {
    let source = r#"
public fn main() -> String {
    bind name <- "Alice";
    $"Hello {name}!"
}
"#;
    let program = Parser::parse_str(source, "fstr.fav").expect("parse");
    let artifact = build_artifact(&program);
    let value = exec_artifact_main(&artifact, None).expect("exec");
    assert_eq!(value, crate::value::Value::Str("Hello Alice!".into()));
}

#[test]
fn fstring_int_auto_show() {
    let source = r#"
public fn main() -> String {
    bind age <- 30;
    $"Age: {age}"
}
"#;
    let program = Parser::parse_str(source, "fstr_int.fav").expect("parse");
    let artifact = build_artifact(&program);
    let value = exec_artifact_main(&artifact, None).expect("exec");
    assert_eq!(value, crate::value::Value::Str("Age: 30".into()));
}

#[test]
fn record_pat_pun_exec() {
    let source = r#"
type User = { name: String  age: Int }
public fn main() -> String {
    bind u <- User { name: "Alice"  age: 30 };
    match u {
        { name, age } -> name
    }
}
"#;
    let program = Parser::parse_str(source, "rec_pat.fav").expect("parse");
    let artifact = build_artifact(&program);
    let value = exec_artifact_main(&artifact, None).expect("exec");
    assert_eq!(value, crate::value::Value::Str("Alice".into()));
}

#[test]
fn record_pat_e055_non_record() {
    let source = r#"
public fn main() -> Int {
    match 42 {
        { x } -> x
    }
}
"#;
    let errors = Checker::check_program_str(source);
    assert!(errors.iter().any(|e| e.code == "E055"), "expected E055");
}

#[test]
fn record_pat_e056_unknown_field() {
    let source = r#"
type User = { name: String }
public fn main() -> String {
    bind u <- User { name: "Alice" };
    match u {
        { name, age } -> name
    }
}
"#;
    let errors = Checker::check_program_str(source);
    assert!(errors.iter().any(|e| e.code == "E056"), "expected E056");
}

#[test]
fn test_filter_matches_description() {
    // cmd_test の filter 動作を確認
    // テスト関数名に "user" を含むものだけ実行
    let source = r#"
test "user login" { assert_eq(1, 1) }
test "payment flow" { assert_eq(2, 2) }
"#;
    // filter="user" で user login のみ実行、payment は skipped になることを確認
    let program = Parser::parse_str(source, "test_filter.test.fav").expect("parse");
    let results = collect_and_run_tests(&program, Some("user"));
    assert_eq!(results.passed, 1);
    assert_eq!(results.filtered, 1);
}

#[test]
fn watch_collect_paths_returns_fav_files() {
    let dir = tempdir().expect("tempdir");
    let f1 = dir.path().join("main.fav");
    let f2 = dir.path().join("other.fav");
    let f3 = dir.path().join("readme.md");
    std::fs::write(&f1, "").ok();
    std::fs::write(&f2, "").ok();
    std::fs::write(&f3, "").ok();
    // collect_watch_paths に dir を渡して .fav ファイルが2件返ることを確認
    let paths = collect_fav_files(dir.path());
    assert_eq!(paths.len(), 2);
    assert!(paths.iter().any(|p| p.ends_with("main.fav")));
    assert!(paths.iter().any(|p| p.ends_with("other.fav")));
}
```

---

## 実装順序まとめ

```
Phase 0: Cargo.toml + main.rs HELP + notify 依存追加
Phase 1: ast.rs(FString/FStringPart) → lexer.rs(FStringRaw) → parser.rs(parse_fstring_parts)
       → checker.rs(E053/E054) → compiler.rs(FString脱糖)
Phase 2: ast.rs(Pattern::Record/RecordPatternField) → parser.rs(parse_record_pattern)
       → checker.rs(E055/E056) → ir.rs(IRPattern::Record) → compiler.rs → vm.rs
Phase 3: ast.rs(Expr::AssertMatches) → parser.rs(parse_assert_matches)
       → checker.rs → compiler.rs → driver.rs(filter/statistics/no_capture) → main.rs
Phase 4: driver.rs(cmd_watch/collect_watch_paths/run_watch_cmd) → main.rs(watch コマンド)
Phase 5: tests + examples + langspec.md + README.md
```
