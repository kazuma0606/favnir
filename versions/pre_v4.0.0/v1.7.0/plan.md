# Favnir v1.7.0 実装プラン

作成日: 2026-05-08

---

## Phase 0 — バージョン更新

```toml
# Cargo.toml
version = "1.7.0"
walkdir = "2"
```

```rust
// main.rs
const HELP: &str = "fav v1.7.0 ...";
```

---

## Phase 1 — `Task<T>` 非同期基盤

### 1-1. `ast.rs`

```rust
// FnDef に async フラグを追加
pub struct FnDef {
    pub name: String,
    pub is_async: bool,   // 追加
    pub params: Vec<(String, TypeExpr)>,
    pub ret_ty: Option<TypeExpr>,
    pub effects: Vec<Effect>,
    pub body: Expr,
    pub span: Span,
}

// TrfDef にも同様に追加
pub struct TrfDef {
    pub name: String,
    pub is_async: bool,   // 追加
    ...
}
```

### 1-2. `lexer.rs`

```rust
// TokenKind に Async を追加
TokenKind::Async,   // "async" キーワード

// keywords に追加
"async" => TokenKind::Async,
```

### 1-3. `parser.rs`

```rust
// parse_fn_def / parse_trf_def の先頭で async を消費
fn parse_fn_def(&mut self) -> Result<FnDef, ParseError> {
    let is_async = self.eat(TokenKind::Async);
    self.expect(TokenKind::Fn)?;
    ...
    Ok(FnDef { is_async, ... })
}

// parse_trf_def も同様
```

### 1-4. `checker.rs`

```rust
// Task<T> 型の定義（組み込み型として登録）
// async fn の戻り型自動ラップ
fn check_fn_def(&mut self, def: &FnDef) {
    let ret_ty = self.resolve_type(&def.ret_ty);
    let effective_ret = if def.is_async {
        Type::Task(Box::new(ret_ty))
    } else {
        ret_ty
    };
    ...
}

// bind で Task<T> を解除するケース
// Expr::Bind(name, rhs) の rhs の型が Task<T> → T として束縛
fn check_bind(&mut self, name: &str, rhs: &Expr) -> Type {
    let rhs_ty = self.check_expr(rhs);
    match rhs_ty {
        Type::Task(inner) => {
            self.env.define(name, *inner.clone());
            *inner
        }
        other => {
            self.env.define(name, other.clone());
            other
        }
    }
}

// Task.run / Task.map / Task.and_then のビルトイン型
// "Task" ネームスペースをビルトインとして登録
```

### 1-5. `compiler.rs`

```rust
// async fn の compile: 本体を IRExpr::Closure でラップして Task コンストラクタを呼ぶ
// Task.run → クロージャを即時呼び出す IRExpr として lowering
fn compile_fn_def(&mut self, def: &FnDef) -> IRFnDef {
    if def.is_async {
        // 本体を クロージャ化: |()| { body }
        // Task::new(closure) に相当する IR を生成
    }
    ...
}
```

### 1-6. `vm.rs`

```rust
// VMValue::Task を追加
pub enum VMValue {
    ...
    Task(Box<dyn FnOnce() -> VMValue + Send>),   // 追加
}

// Task.run → Box を呼び出す
// Task.map(task, f) → Task(move || f(task()))
// Task.and_then(task, f) → Task(move || match f(task()) { VMValue::Task(t) => t(), v => v })

// vm_call_builtin に Task ネームスペースを追加
"Task.run" => {
    let task = args[0];
    match task {
        VMValue::Task(f) => f(),
        _ => runtime_error("E058: not a Task"),
    }
}
"Task.map" => { ... }
"Task.and_then" => { ... }

// bind で Task を自動解除
// CALL_RESULT で VMValue::Task を検出して即実行する処理を追加
```

---

## Phase 2 — 型エイリアス

### 2-1. `ast.rs`

```rust
// TypeDefBody に Alias バリアントを追加
pub enum TypeDefBody {
    Record(Vec<FieldDef>),
    Sum(Vec<VariantDef>),
    Alias(TypeExpr),    // 追加: type Name = TypeExpr
}

// パーサーで `type Name = TypeExpr` のとき Alias を選択
// `type Name { ... }` のとき Record/Sum を選択
```

### 2-2. `parser.rs`

```rust
fn parse_type_def(&mut self) -> Result<TypeDef, ParseError> {
    self.expect(TokenKind::Type)?;
    let name = self.expect_ident()?;
    let type_params = self.parse_type_params()?;

    if self.eat(TokenKind::Eq) {
        // エイリアス: type Name = TypeExpr
        let target = self.parse_type_expr()?;
        Ok(TypeDef { name, type_params, body: TypeDefBody::Alias(target), ... })
    } else {
        // 既存: type Name { ... } または type Name = Variant | ...
        ...
    }
}
```

### 2-3. `checker.rs`

```rust
// 型エイリアスを登録・展開
fn register_type_alias(&mut self, def: &TypeDef) {
    if let TypeDefBody::Alias(target) = &def.body {
        let resolved = self.resolve_type_expr(target);
        // 循環チェック → E060
        self.type_aliases.insert(def.name.clone(), resolved);
    }
}

fn resolve_type(&mut self, ty: &Type) -> Type {
    match ty {
        Type::Named(name, args) => {
            if let Some(alias_target) = self.type_aliases.get(name) {
                // エイリアス展開
                alias_target.clone()
            } else {
                ty.clone()
            }
        }
        ...
    }
}
```

### 2-4. エラー定義

```rust
// E059: 型エイリアスの参照先が未定義
// E060: 型エイリアスが循環している

// 循環チェック: 解決中のエイリアス名を HashSet で追跡し、再訪したら E060
fn resolve_alias_with_cycle_check(
    &mut self,
    name: &str,
    visiting: &mut HashSet<String>,
) -> Result<Type, Error> {
    if visiting.contains(name) {
        return Err(E060 ...);
    }
    visiting.insert(name.to_string());
    ...
}
```

---

## Phase 3 — `fav test --coverage`

### 3-1. `ir.rs`

```rust
// IRStmt に TrackLine を追加
pub enum IRStmt {
    Expr(IRExpr),
    Bind(String, IRExpr),
    Assign(String, IRExpr),
    Return(IRExpr),
    TrackLine(u32),   // 追加: カバレッジ追跡用
}
```

### 3-2. `compiler.rs`

```rust
// compile_stmt でカバレッジ追跡文を挿入（coverage_mode フラグが true の場合）
fn compile_stmt(&mut self, stmt: &Stmt, stmts: &mut Vec<IRStmt>) {
    if self.coverage_mode {
        stmts.push(IRStmt::TrackLine(stmt.span().line));
    }
    // 本来の compile_stmt 処理
    ...
}

// CompileCtx に coverage_mode: bool を追加
pub struct CompileCtx {
    ...
    pub coverage_mode: bool,   // 追加
}
```

### 3-3. `vm.rs`

```rust
// VM に coverage フィールドを追加
pub struct VM {
    stack: Vec<VMValue>,
    ...
    coverage: Option<HashSet<u32>>,   // 追加
}

pub fn enable_coverage(&mut self) {
    self.coverage = Some(HashSet::new());
}

pub fn take_coverage(&mut self) -> HashSet<u32> {
    self.coverage.take().unwrap_or_default()
}

// TRACK_LINE オペコードまたは IRStmt::TrackLine の処理
// → self.coverage.as_mut().map(|s| s.insert(line));
```

### 3-4. `driver.rs`

```rust
// cmd_test に coverage: bool パラメータを追加
pub fn cmd_test(
    file: Option<&str>,
    filter: Option<&str>,
    no_capture: bool,
    coverage: bool,    // 追加
) {
    ...
    // coverage が true なら vm.enable_coverage() を呼ぶ
    // テスト完了後に vm.take_coverage() でカバレッジを取得
    // format_coverage_report(source, executed_lines, total_executable_lines) を呼ぶ
}

fn format_coverage_report(
    file_path: &str,
    source: &str,
    executed: &HashSet<u32>,
) -> String {
    let executable = count_executable_lines(source);
    let covered = executed.len();
    let pct = if executable == 0 { 0.0 } else { covered as f64 / executable as f64 * 100.0 };
    let uncovered: Vec<u32> = (1..=source.lines().count() as u32)
        .filter(|l| is_executable_line(source, *l) && !executed.contains(l))
        .collect();
    format!(
        "\ncoverage: {}\n  lines covered: {} / {} ({:.1}%)\n  uncovered:     lines {}",
        file_path, covered, executable, pct,
        uncovered.iter().map(|l| l.to_string()).collect::<Vec<_>>().join(", ")
    )
}

fn is_executable_line(source: &str, line: u32) -> bool {
    // コメント行・空行・型定義行は false
    // fn/trf 本体の式・束縛・return を含む行は true
    let line_str = source.lines().nth((line - 1) as usize).unwrap_or("");
    let trimmed = line_str.trim();
    !trimmed.is_empty()
        && !trimmed.starts_with("//")
        && !trimmed.starts_with("type ")
        && !trimmed.starts_with("}")
}
```

---

## Phase 4 — `fav watch` 複数ディレクトリ対応

### 4-1. `driver.rs`

```rust
// cmd_watch のシグネチャ変更
pub fn cmd_watch(
    file: Option<&str>,
    cmd: &str,
    extra_dirs: &[&str],   // 追加
    debounce_ms: u64,      // 追加（デフォルト 80）
) {
    let mut paths = collect_watch_paths(file);

    // extra_dirs から追加パスを収集
    for dir in extra_dirs {
        let dir_paths = collect_watch_paths_from_dir(dir);
        paths.extend(dir_paths);
    }
    paths.dedup();

    // ウォッチャーのデバウンス時間を debounce_ms に変更
    let debounce = Duration::from_millis(debounce_ms);
    ...
}

pub fn collect_watch_paths_from_dir(dir: &str) -> Vec<PathBuf> {
    let base = PathBuf::from(dir);
    collect_fav_files_recursive(&base)
}

fn collect_fav_files_recursive(dir: &PathBuf) -> Vec<PathBuf> {
    let mut result = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                result.extend(collect_fav_files_recursive(&path));
            } else if path.extension().map(|e| e == "fav").unwrap_or(false) {
                result.push(path);
            }
        }
    }
    result
}
```

### 4-2. `main.rs`

```rust
// watch コマンドに --dir と --debounce フラグを追加
Some("watch") => {
    let mut dirs: Vec<&str> = Vec::new();
    let mut debounce_ms: u64 = 80;
    let mut cmd_str = String::from("check");
    let mut file: Option<&str> = None;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--dir" => {
                i += 1;
                if let Some(d) = args.get(i) { dirs.push(d); }
            }
            "--debounce" => {
                i += 1;
                if let Some(ms) = args.get(i) {
                    debounce_ms = ms.parse().unwrap_or(80);
                }
            }
            "--cmd" => {
                i += 1;
                if let Some(c) = args.get(i) { cmd_str = c.to_string(); }
            }
            f if !f.starts_with("--") => file = Some(f),
            _ => {}
        }
        i += 1;
    }

    cmd_watch(file, &cmd_str, &dirs, debounce_ms);
}
```

---

## Phase 5 — テスト・ドキュメント

### テスト構造

```rust
// tests/task_tests.rs (新規)
#[test]
fn task_async_fn_returns_task_type() { ... }
#[test]
fn task_bind_unwraps_task() { ... }
#[test]
fn task_run_executes_immediately() { ... }
#[test]
fn task_map_transforms_value() { ... }

// tests/type_alias_tests.rs (新規)
#[test]
fn type_alias_simple() { ... }
#[test]
fn type_alias_compatible_with_target() { ... }
#[test]
fn type_alias_generic() { ... }
#[test]
fn type_alias_e059_unknown_target() { ... }
#[test]
fn type_alias_e060_circular() { ... }

// driver.rs の既存テストに追加
#[test]
fn coverage_tracks_executed_lines() { ... }
#[test]
fn coverage_excludes_unexecuted_branches() { ... }
#[test]
fn coverage_report_format() { ... }
#[test]
fn watch_collect_paths_from_dirs() { ... }
#[test]
fn watch_collect_paths_multiple_dirs() { ... }
```

### example ファイル

```fav
// examples/async_demo.fav
async fn fetch_greeting(name: String) -> String !Io {
    IO.println($"Fetching for {name}...");
    $"Hello, {name}!"
}

fn main() -> Unit !Io {
    bind msg <- fetch_greeting("Alice");
    IO.println(msg)
}
```

```fav
// examples/type_alias_demo.fav
type UserId = Int
type UserName = String
type UserScore = Float

type UserRecord = { id: UserId, name: UserName, score: UserScore }

fn format_user(u: UserRecord) -> String {
    $"[{Int.show.show(u.id)}] {u.name}: {Float.show.show(u.score)}"
}

fn main() -> Unit !Io {
    bind u = UserRecord { id: 1, name: "Alice", score: 98.5 };
    IO.println(format_user(u))
}
```
