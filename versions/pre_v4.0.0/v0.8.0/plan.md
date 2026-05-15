# Favnir v0.8.0 実装計画

更新日: 2026-04-30（Codex レビュー反映）

---

## フェーズ構成と依存関係

```
Phase 0-A: バージョン文字列統一    ← 即実施（5分）
Phase 0-B: 警告解消               ← 即実施
Phase 0-C: value.rs 切り出し      ← eval.rs 廃止の前提
Phase 0-D: eval.rs 廃止（段階的） ← 0-C 完了後。切替→枯らす→削除
    │
    ├── Phase 1: fav test          ← AST 追加が必要
    ├── Phase 2: fav fmt           ← 並行可（AST 依存のみ）
    ├── Phase 3: エラーメッセージ   ← 警告解消後（Span 活用）
    ├── Phase 4: fav explain 強化  ← IR ベース DEPS
    └── Phase 5: fav lint (MVP)    ← 最後（AST ウォーカー）
```

---

## Phase 0-A: バージョン文字列統一

```toml
# Cargo.toml
version = "0.8.0"
```

```rust
// src/main.rs 内の HELP 定数
fav - Favnir language toolchain v0.8.0
```

---

## Phase 0-B: 警告解消

### 削除（本当に不要）

| 対象 | ファイル |
|---|---|
| `fn compose_effects` | `middle/checker.rs` |
| `fn merge_effect` | `middle/checker.rs` |
| `fn instantiate` | `middle/checker.rs` |
| フィールド `type_params`（Checker 構造体） | `middle/checker.rs` |
| フィールド `subst`（Checker 構造体） | `middle/checker.rs` |

削除前に grep で呼び出し箇所がないことを確認する。

### `#[allow(dead_code)]` を追加

`ast.rs` 先頭に `#![allow(dead_code)]` を追加。対象：
- `TypeExpr` の Span フィールド群（Phase 3 で使う予定）
- `Field.span`, `Variant` の Span フィールド
- `EmitUnion` variant
- `NamespaceDecl`, `UseDecl` variants

`eval::run`, `eval_item` は Phase 0-D で解消されるため、今は手を付けない。

---

## Phase 0-C: value.rs 切り出し

### 手順

1. `src/value.rs` 新規作成
2. `eval.rs` の `pub enum Value { ... }` と全 `impl Value` を `value.rs` に移動
3. `eval.rs` に `pub use crate::value::Value;` を追加（driver.rs 等の既存参照を守る）
4. `backend/vm.rs`: `use crate::eval::Value` → `use crate::value::Value`
5. `src/main.rs` に `mod value;` を追加
6. `cargo test` 全通過確認

---

## Phase 0-D: eval.rs の廃止（段階的）

### Step 1: fav run を VM 経路に切り替え

**`driver.rs` の `cmd_run` を書き換える**

```rust
pub fn cmd_run(file: Option<&str>, db_url: Option<&str>) {
    let (program, _path) = load_and_check_program(file);
    let artifact = build_artifact(&program);
    let mut vm = VM::new();
    vm.set_db_url(db_url.unwrap_or(":memory:"));
    if let Err(e) = vm.exec_main(&artifact) {
        eprintln!("{}", e);
        process::exit(1);
    }
}
```

- `Interpreter::run_with_db` の呼び出しを削除
- `driver.rs` の `use crate::eval::Interpreter` を削除
- `cargo test` 全通過確認（出力の一致を確認）

### Step 2: eval.rs の枯らしと削除

- `src/eval.rs` からの参照がゼロになったことを確認（`use crate::eval` が driver.rs / main.rs にないこと）
- `src/eval.rs` を削除
- `src/main.rs`: `mod eval;` を削除
- eval.rs の `#[cfg(test)]` テストを vm.rs のテストに移植（等価なカバレッジ）
- `cargo test` 全通過確認

---

## Phase 1: fav test

### Step 1: レキサー + パーサ

**`frontend/lexer.rs`**
```rust
"test" => TokenKind::Test,
```

**`ast.rs`**
```rust
pub struct TestDef {
    pub name: String,   // テスト説明文字列
    pub body: Block,
    pub span: Span,
}
// Item::TestDef(TestDef) を追加
```

**`frontend/parser.rs`**
```rust
fn parse_test_item(&mut self) -> Result<TestDef, ParseError> {
    // expect(Test) → expect(Str) → parse_block()
}
```

### Step 2: 型検査

**`middle/checker.rs`**
```rust
fn check_test_def(&mut self, test: &TestDef) -> Vec<TypeError> {
    // body を Unit 期待で型検査
    // assert / assert_eq / assert_ne を組み込みとして型 env に追加
    // エフェクト: !Io !File を許可
}
```

アサーション型登録（check 開始時に env に追加）：
```
assert    : Bool -> Unit
assert_eq : T -> T -> Unit  （Eq cap が必要; MVP では制約なしで実装）
assert_ne : T -> T -> Unit
```

### Step 3: VM 組み込みアサーション

**`backend/vm.rs`** の `vm_call_builtin` に追加：
```rust
("", "assert") => {
    match args[0] {
        Value::Bool(true) => Ok(Value::Unit),
        Value::Bool(false) => Err(VMError::TestFailure("assert failed".into())),
        _ => Err(...)
    }
}
("", "assert_eq") => { /* 値比較 */ }
("", "assert_ne") => { /* 値比較 */ }
```

`VMError::TestFailure(String)` を追加。

### Step 4: driver.rs

```rust
pub fn cmd_test(file: Option<&str>, filter: Option<&str>, fail_fast: bool) {
    // 1. parse + typecheck
    // 2. program.items から TestDef を収集
    // 3. filter でフィルタリング
    // 4. 各 test を compile → vm.exec_test_body() で実行
    // 5. pass / FAILED を表示
    // 6. fail_fast なら最初の失敗で break
}
```

`vm.exec_test_body(artifact, test_fn_idx)` — test item の Block を
単独関数として コンパイルして実行する。

### Step 5: main.rs

```rust
Some("test") => {
    // --filter, --fail-fast, --trace のパース
    cmd_test(file, filter, fail_fast);
}
```

---

## Phase 2: fav fmt（MVP）

### src/fmt.rs の設計

```rust
pub struct Formatter {
    out: String,
    indent: usize,
}

impl Formatter {
    pub fn format_program(prog: &Program) -> String
    fn emit(&mut self, s: &str)
    fn emit_line(&mut self, s: &str)    // indent + s + \n
    fn indent(&mut self)
    fn dedent(&mut self)

    fn format_item(&mut self, item: &Item)
    fn format_fn_def(&mut self, f: &FnDef)
    fn format_trf_def(&mut self, t: &TrfDef)
    fn format_flw_def(&mut self, f: &FlwDef)
    fn format_type_def(&mut self, t: &TypeDef)
    fn format_block(&mut self, b: &Block)
    fn format_stmt(&mut self, s: &Stmt)
    fn format_expr(&mut self, e: &Expr) -> String
    fn format_pattern(&mut self, p: &Pattern) -> String
    fn format_type_expr(&self, te: &TypeExpr) -> String
    fn format_effects(&self, effects: &[Effect]) -> String
}
```

**MVP 対象**: `fn`, `trf`, `flw`, `type`, `bind`, `chain`, `match`, `if`, 全 `Expr`
**MVP 対象外**: `cap`, `impl`, `test`（存在するが整形はパススルー）

### driver.rs

```rust
pub fn cmd_fmt(file: Option<&str>, check: bool) {
    // 1. 対象ファイルを列挙
    // 2. 各ファイル: parse → Formatter::format_program → formatted string
    // 3. check: 差分比較 → 差分あり exit 1
    // 4. 通常: 上書き
}
```

---

## Phase 3: エラーメッセージ改善

### format_diagnostic ヘルパー

**`driver.rs`** に追加：

```rust
fn format_diagnostic(source: &str, span: &Span, label: &str) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let line_no = span.line as usize;
    let col = span.col as usize;
    let src_line = lines.get(line_no.saturating_sub(1)).unwrap_or(&"");
    let underline = " ".repeat(col.saturating_sub(1))
        + &"^".repeat((span.end - span.start).max(1));
    format!(
        "   |\n{:>4} | {}\n   | {}\n   = {}\n",
        line_no, src_line, underline, label
    )
}
```

### エラー出力の更新

`cmd_check`, `cmd_run` の `TypeError` 出力で `format_diagnostic` を呼び出す。
エラーコードは `error[EXXX]:` 形式に統一。

---

## Phase 4: fav explain 強化

### IR ベース DEPS 収集

```rust
// middle/ir.rs に追加
pub fn collect_deps(fn_def: &IRFnDef) -> Vec<String> {
    let mut deps = vec![];
    collect_deps_expr(&fn_def.body, &mut deps);
    deps.sort();
    deps.dedup();
    deps
}

fn collect_deps_expr(expr: &IRExpr, out: &mut Vec<String>) {
    match expr {
        IRExpr::Builtin { namespace, name } => {
            out.push(format!("{}.{}", namespace, name));
        }
        IRExpr::GlobalRef(name) => out.push(name.clone()),
        // ... 再帰
    }
}
```

### cmd_explain の更新

IR コンパイルをかけてから DEPS を表示：

```rust
// compile_program を呼んで IRProgram を取得 → collect_deps で一覧
```

---

## Phase 5: fav lint（MVP）

### src/lint.rs の設計

```rust
pub struct LintError {
    pub code: &'static str,    // "L001" 等
    pub message: String,
    pub span: Span,
}

pub fn lint_program(program: &Program) -> Vec<LintError>
```

### L001-L004 の実装

```rust
// L001: pub fn が戻り値型を持たない
fn check_return_type(fn_def: &FnDef) -> Option<LintError>

// L002: 未使用の bind 束縛
fn check_unused_binds(block: &Block) -> Vec<LintError>

// L003: fn 名がスネークケースでない
fn check_fn_name_case(fn_def: &FnDef) -> Option<LintError>

// L004: type 名がパスカルケースでない
fn check_type_name_case(type_def: &TypeDef) -> Option<LintError>
```

### driver.rs

```rust
pub fn cmd_lint(file: Option<&str>, warn_only: bool) {
    // parse → lint_program → エラー出力
    // warn_only=false: exit 1 on errors
}
```

---

## 実施順序

```
0-A → 0-B → 0-C → 0-D（Step1: 切替）→ テスト確認 → 0-D（Step2: 削除）
    → Phase 1（fav test）
    → Phase 2（fav fmt）     // Phase 1 と並行可
    → Phase 3（エラー改善）
    → Phase 4（explain）
    → Phase 5（lint）
```

各フェーズで `cargo test` 全通過を確認してから次へ。
