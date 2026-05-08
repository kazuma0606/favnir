# Favnir v1.8.0 実装プラン

作成日: 2026-05-09

---

## Phase 0 — バージョン更新

```toml
# Cargo.toml
version = "1.8.0"
```

```rust
// main.rs
const HELP: &str = "fav v1.8.0 ...";
```

---

## Phase 1 — `Task` 並列 API

### 1-1. `checker.rs` — ビルトイン登録拡張

`register_builtins` の `"Task"` ネームスペースに以下を追加:

```
Task.all       : List<Task<T>> -> Task<List<T>>
Task.race      : List<Task<T>> -> Task<T>
Task.timeout   : Task<T> -> Int -> Task<Option<T>>
```

型チェック戦略:
- `Task.all(list)` の `list` の型が `List<Task<T>>` であることを確認し、戻り型 `Task<List<T>>` を返す。
- `Task.race(list)` も同様、戻り型 `Task<T>`。
- `Task.timeout(task, ms)` は `Task<Option<T>>` を返す。
- 型変数 `T` は呼び出し時に単相化（unify で解決）。

E061: リストが空（`List.length == 0` は動的チェック）→ vm.rs で実行時エラー。

### 1-2. `vm.rs` — ビルトイン実装

`vm_call_builtin` に追加:

```rust
"Task.all" => {
    // args[0]: VMValue::List(tasks)
    // tasks の各要素を実行（v1.8.0 では同期順次実行）
    // 結果を List に収集して返す
    let tasks = match &args[0] { VMValue::List(v) => v.clone(), _ => runtime_error("E061") };
    if tasks.is_empty() { runtime_error("E061: Task.all received empty list") }
    let results: Vec<VMValue> = tasks.into_iter().map(|t| {
        // Task は v1.8.0 では即値（透明）なので that unwrap
        t
    }).collect();
    VMValue::List(results)
}

"Task.race" => {
    // args[0]: VMValue::List(tasks) — 先頭要素を返す
    let tasks = match &args[0] { VMValue::List(v) => v.clone(), _ => runtime_error("E061") };
    if tasks.is_empty() { runtime_error("E061: Task.race received empty list") }
    tasks.into_iter().next().unwrap()
}

"Task.timeout" => {
    // args[0]: task value, args[1]: ms (Int)
    // v1.8.0 では常に Some(value) を返す
    VMValue::Variant("some".into(), Some(Box::new(args[0].clone())))
}
```

---

## Phase 2 — `async fn main()`

### 2-1. `checker.rs` — エントリポイント検証の拡張

`ensure_valid_main`（または該当する main 型チェック箇所）を以下に変更:

```rust
// 現在: main の型が () -> Unit !Io のみ許可
// 変更後: () -> Task<Unit> !Io も許可
fn is_valid_main_type(ty: &Type) -> bool {
    match ty {
        // 既存: () -> Unit !Io
        Type::Fn(params, ret, effs) if params.is_empty() => {
            matches!(ret.as_ref(), Type::Unit) || matches!(ret.as_ref(), Type::Task(t) if matches!(t.as_ref(), Type::Unit))
        }
        _ => false,
    }
}
```

### 2-2. `driver.rs` — `exec_artifact_main` 拡張

```rust
// main の戻り値が Task<Unit> の場合、自動的に Task.run する
pub fn exec_artifact_main(artifact: &FvcArtifact, db: Option<&str>) -> Result<Value, String> {
    let result = /* VM 実行 */;
    // Task<Unit> の自動解除
    // v1.8.0 の Task は透明なので特別処理不要（値がそのまま Unit）
    // ただし VMValue::Task が将来実装された場合のために分岐を確認
    Ok(result)
}
```

---

## Phase 3 — `chain` + `Task<T>` 統合

### 3-1. `checker.rs` — `check_chain_stmt` の拡張

```rust
fn check_chain_stmt(&mut self, stmt: &ChainStmt) -> Type {
    let rhs_ty = self.check_expr(&stmt.rhs);

    // Task ラッパーを剥がす
    let inner_ty = match rhs_ty {
        Type::Task(inner) => *inner,
        other => other,
    };

    // 既存の chain チェック（Result / Option の伝播）
    match inner_ty {
        Type::Result(ok, _err) => {
            self.env.define(&stmt.name, *ok);
        }
        Type::Option(inner) => {
            self.env.define(&stmt.name, *inner);
        }
        other => {
            self.report_error(E063, ...);
            other
        }
    }
}
```

### 3-2. `vm.rs` — chain ハンドラの変更

chain 実行時に Task ラッパーの透明解除を確認:

```rust
// ChainResult の評価で VMValue::Task が来た場合、即値として扱う
// v1.8.0 の Task は透明値なので変更不要（Task.run と同等の動作は既に実装済み）
```

---

## Phase 4 — Coverage 強化

### 4-1. `driver.rs` — 関数単位レポート

```rust
// 関数名 → カバーされた行のマップを追跡
// IRFnDef.name を TrackLine に紐付ける

// 既存の format_coverage_report を拡張
pub fn format_coverage_report_by_fn(
    file_path: &str,
    source: &str,
    executed: &HashSet<u32>,
    fn_line_ranges: &HashMap<String, (u32, u32)>,  // fn_name -> (start_line, end_line)
) -> String {
    // ファイル全体サマリ（既存）
    // + 関数ごとのブレークダウン
}

// fn_line_ranges を IRProgram から収集する補助関数
fn collect_fn_line_ranges(program: &IRProgram) -> HashMap<String, (u32, u32)> {
    program.functions.iter().map(|f| {
        let lines: Vec<u32> = f.body.iter()
            .filter_map(|s| if let IRStmt::TrackLine(l) = s { Some(*l) } else { None })
            .collect();
        let min = lines.iter().copied().min().unwrap_or(0);
        let max = lines.iter().copied().max().unwrap_or(0);
        (f.name.clone(), (min, max))
    }).collect()
}
```

### 4-2. `driver.rs` — `--coverage-report <dir>` フラグ

```rust
pub fn cmd_test(
    file: Option<&str>,
    filter: Option<&str>,
    no_capture: bool,
    coverage: bool,
    coverage_report_dir: Option<&str>,   // 追加
) {
    // ... 既存ロジック ...
    if let Some(dir) = coverage_report_dir {
        std::fs::create_dir_all(dir).ok();
        let report_path = std::path::Path::new(dir).join("coverage.txt");
        std::fs::write(&report_path, &report_str).ok();
        println!("coverage report written to: {}", report_path.display());
    }
}
```

### 4-3. `main.rs`

```rust
// test コマンドに --coverage-report フラグを追加
"--coverage-report" => {
    i += 1;
    coverage_report_dir = args.get(i).map(|s| s.as_str());
}
```

---

## Phase 5 — `fav bench`

### 5-1. `ast.rs` — `BenchDef` と `Item::BenchDef`

```rust
pub struct BenchDef {
    pub description: String,
    pub body: Expr,
    pub span: Span,
}

pub enum Item {
    // ... 既存 ...
    BenchDef(BenchDef),   // 追加
}
```

### 5-2. `lexer.rs` — `Bench` トークン

```rust
TokenKind::Bench,   // "bench" キーワード
"bench" => TokenKind::Bench,
```

### 5-3. `parser.rs` — `parse_bench_def`

```rust
fn parse_bench_def(&mut self) -> Result<BenchDef, ParseError> {
    self.expect(TokenKind::Bench)?;
    let description = self.expect_string()?;   // 文字列リテラル
    let body = self.parse_block_expr()?;
    Ok(BenchDef { description, body, span: ... })
}

// parse_item に TokenKind::Bench => parse_bench_def() を追加
```

### 5-4. `checker.rs` — `check_bench_def`

```rust
fn check_bench_def(&mut self, bd: &BenchDef) {
    // 本体を型チェック（戻り型は何でも可）
    // !File / !Db / !Network は E064（Io のみ許可）
    let body_ty = self.check_expr(&bd.body);
    // エフェクトチェック
}
```

### 5-5. `driver.rs` — `cmd_bench`

```rust
pub fn cmd_bench(
    file: Option<&str>,
    filter: Option<&str>,
    iters: u64,
) {
    let (program, source) = load_and_check_program(file);
    let bench_cases = collect_bench_cases(&program);

    if bench_cases.is_empty() {
        println!("no bench blocks found");
        return;
    }

    println!("running {} benchmarks", bench_cases.len());
    let artifact = build_artifact(&program);

    for bc in &bench_cases {
        if let Some(f) = filter {
            if !bc.description.contains(f) {
                println!("  skip   {}", bc.description);
                continue;
            }
        }

        let mut total_ns: u128 = 0;
        let mut min_ns = u128::MAX;
        let mut max_ns: u128 = 0;

        for _ in 0..iters {
            let start = std::time::Instant::now();
            let _ = exec_bench_case(&artifact, bc);
            let elapsed = start.elapsed().as_nanos();
            total_ns += elapsed;
            if elapsed < min_ns { min_ns = elapsed; }
            if elapsed > max_ns { max_ns = elapsed; }
        }

        let avg_ns = total_ns / iters as u128;
        let avg_us = avg_ns as f64 / 1000.0;
        println!("  bench  {:<40} {:.2} µs/iter  ({} iters)", bc.description, avg_us, iters);
    }
}

fn collect_bench_cases(program: &Program) -> Vec<&BenchDef> {
    program.items.iter().filter_map(|item| {
        if let Item::BenchDef(bd) = item { Some(bd) } else { None }
    }).collect()
}
```

### 5-6. `main.rs` — `bench` コマンド

```rust
Some("bench") => {
    let mut file: Option<&str> = None;
    let mut filter: Option<&str> = None;
    let mut iters: u64 = 100;
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--filter" => { i += 1; filter = args.get(i).map(|s| s.as_str()); }
            "--iters"  => { i += 1; iters = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(100); }
            f if !f.starts_with("--") => file = Some(f),
            _ => {}
        }
        i += 1;
    }
    cmd_bench(file, filter, iters);
}
```

---

## Phase 6 — テスト・ドキュメント

### テスト配置

- `checker.rs` の `mod tests` に Task 並列 API・chain+Task テストを追加
- `driver.rs` の `mod tests` に coverage-by-fn・bench テストを追加
- `frontend/parser.rs` の `mod tests` に bench パーサーテストを追加

### example ファイル

```favnir
// examples/task_parallel_demo.fav
async fn fetch_a() -> String !Io { "result_a" }
async fn fetch_b() -> String !Io { "result_b" }

public fn main() -> Unit !Io {
    bind results <- Task.all([fetch_a(), fetch_b()])
    IO.println(List.join(results, ", "));
    bind first <- Task.race([fetch_a(), fetch_b()])
    IO.println(first)
}
```

```favnir
// examples/async_main_demo.fav
async fn setup() -> String !Io {
    "initialized"
}

async fn main() -> Unit !Io {
    bind msg <- setup()
    IO.println(msg)
}
```

```favnir
// examples/math.bench.fav
fn add(a: Int, b: Int) -> Int { a + b }

bench "add two numbers" {
    add(100, 200)
}

bench "list fold 1000 items" {
    List.range(0, 1000) |> List.fold(0, |acc, x| acc + x)
}
```
