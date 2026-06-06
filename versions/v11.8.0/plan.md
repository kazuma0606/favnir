# Favnir v11.8.0 実装計画

作成日: 2026-06-06

---

## 実装順序

```
Phase A: emit_python.rs — lineage_comments フィールド + emit_python_with_lineage API
    ↓
Phase B: emit_python.rs — emit_fn_def / emit_trf_def にコメント挿入
    ↓
Phase C: driver.rs — build_lineage_comments ヘルパー追加
    ↓
Phase D: driver.rs — cmd_transpile に --no-check / --lineage フラグ + 型チェック統合
    ↓
Phase E: テスト（v11800_tests）
    ↓
Phase F: バージョン更新・コミット
```

---

## Phase A — emit_python.rs: lineage_comments フィールド + 新 API

### A-1: `use` 追加

```rust
use std::collections::HashMap;
```

### A-2: `Emitter` 構造体にフィールド追加

`type_names: Vec<String>` の直前に追加:

```rust
lineage_comments: HashMap<String, String>,
```

### A-3: `Emitter::new()` 初期化

```rust
lineage_comments: HashMap::new(),
```

### A-4: `emit_python_with_lineage` 関数追加（`emit_python_str` の直後）

```rust
pub fn emit_python_with_lineage(
    prog: &Program,
    source_path: &str,
    comments: HashMap<String, String>,
) -> String {
    let mut e = Emitter::new();
    e.lineage_comments = comments;
    e.emit_program(prog, source_path)
}
```

---

## Phase B — emit_fn_def / emit_trf_def コメント挿入

### B-1: `emit_fn_def` 先頭にコメント挿入

`emit_fn_def` の先頭（`self.blank()` の前）に追加:

```rust
fn emit_fn_def(&mut self, fd: &FnDef) {
    // lineage コメント（--lineage 指定時のみ存在）
    let fn_name = fd.name.clone();
    if let Some(comment) = self.lineage_comments.get(&fn_name).cloned() {
        self.line(&comment);
    }
    // ...既存コード（self.blank() など）...
}
```

### B-2: `emit_trf_def`（stage）先頭にコメント挿入

同様に `emit_trf_def` の先頭に追加:

```rust
fn emit_trf_def(&mut self, td: &TrfDef) {
    let stage_name = td.name.clone();
    if let Some(comment) = self.lineage_comments.get(&stage_name).cloned() {
        self.line(&comment);
    }
    // ...既存コード...
}
```

---

## Phase C — driver.rs: build_lineage_comments ヘルパー

`build_readme_content` の直後に追加:

```rust
fn build_lineage_comments(
    report: &crate::lineage::LineageReport,
) -> std::collections::HashMap<String, String> {
    report.transformations.iter().map(|entry| {
        let effects = if entry.effects.is_empty() {
            "Pure".to_string()
        } else {
            entry.effects.join(", ")
        };
        let sources = if entry.sources.is_empty() {
            "-".to_string()
        } else {
            entry.sources.join(", ")
        };
        let sinks = if entry.sinks.is_empty() {
            "-".to_string()
        } else {
            entry.sinks.join(", ")
        };
        let comment = format!(
            "# [lineage] effects: {} | sources: {} | sinks: {}",
            effects, sources, sinks
        );
        (entry.name.clone(), comment)
    }).collect()
}
```

---

## Phase D — cmd_transpile: --no-check / --lineage + 型チェック統合

### D-1: 引数追加

```rust
let mut do_no_check = false;
let mut do_lineage  = false;
```

### D-2: パーサに追加

```rust
"--no-check" => { do_no_check = true; }
"--lineage"  => { do_lineage = true; }
```

### D-3: 型チェック統合（parse 後、emit_python 前）

```rust
if !do_no_check {
    let errors = check_source_str(&src);
    if !errors.is_empty() {
        for e in &errors {
            eprintln!("{}", format_diagnostic(&src, e));
        }
        eprintln!("error: {} type error(s); Python generation blocked", errors.len());
        std::process::exit(1);
    }
}
```

`check_source_str` は既に `driver.rs` 内に定義済み（`fn check_source_str`）。
ただし現在は `fn`（非 pub）のため、同一ファイル内から直接呼べる。

### D-4: `--lineage` 時の emit_python 切り替え

```rust
let py_src = if do_lineage {
    let report = crate::lineage::lineage_analysis(&prog);
    let comments = build_lineage_comments(&report);
    crate::emit_python::emit_python_with_lineage(&prog, &input, comments)
} else {
    crate::emit_python::emit_python(&prog, &input)
};
```

---

## Phase E — テスト（v11800_tests）

`driver.rs` 末尾の `v11700_tests` の後に追加。

```rust
#[cfg(test)]
mod v11800_tests {
    use crate::emit_python::{emit_python_str, emit_python_with_lineage};
    use crate::frontend::parser::Parser;
    use crate::lineage::lineage_analysis;
    use std::collections::HashMap;
}
```

### テスト一覧

```rust
#[test]
fn transpile_blocks_on_type_error() {
    // !Postgres なし → E0315 → check_source_str がエラーを返す
    let src = "fn run(sql: String) -> Result<String, String> {\n\
               Postgres.query_raw(sql, \"[]\")\n}";
    let errors = crate::driver::check_source_str_pub(src);
    assert!(!errors.is_empty(), "expected type errors");
}

#[test]
fn transpile_type_check_passes_valid() {
    let src = "fn greet(name: String) -> String {\n  name\n}";
    let errors = crate::driver::check_source_str_pub(src);
    assert!(errors.is_empty(), "expected no errors: {:?}", errors);
}

#[test]
fn transpile_lineage_comment_effects() {
    let src = "fn fetch(sql: String) -> Result<String, String> !Postgres {\n\
               Postgres.query_raw(sql, \"[]\")\n}";
    let prog = Parser::parse_str(src, "<test>").unwrap();
    let report = lineage_analysis(&prog);
    let mut comments = HashMap::new();
    for entry in &report.transformations {
        let effects = if entry.effects.is_empty() { "Pure".to_string() }
                      else { entry.effects.join(", ") };
        comments.insert(entry.name.clone(),
            format!("# [lineage] effects: {} | sources: - | sinks: -", effects));
    }
    let out = emit_python_with_lineage(&prog, "<test>", comments);
    assert!(out.contains("# [lineage] effects:"), "lineage comment:\n{}", out);
}

#[test]
fn transpile_lineage_comment_pure_fn() {
    let src = "fn add(a: Int, b: Int) -> Int {\n  a\n}";
    let prog = Parser::parse_str(src, "<test>").unwrap();
    let report = lineage_analysis(&prog);
    let mut comments = HashMap::new();
    for entry in &report.transformations {
        let effects = if entry.effects.is_empty() { "Pure".to_string() }
                      else { entry.effects.join(", ") };
        comments.insert(entry.name.clone(),
            format!("# [lineage] effects: {} | sources: - | sinks: -", effects));
    }
    let out = emit_python_with_lineage(&prog, "<test>", comments);
    assert!(out.contains("# [lineage] effects: Pure"), "pure fn comment:\n{}", out);
}

#[test]
fn transpile_no_check_skips_error() {
    // --no-check 相当: check_source_str を呼ばずに emit_python_str が動作する
    // (型エラーコードでも Python 生成が成功する)
    let src = "fn run(sql: String) -> Result<String, String> {\n\
               Postgres.query_raw(sql, \"[]\")\n}";
    let out = emit_python_str(src);
    assert!(out.contains("def run("), "python generated:\n{}", out);
}

#[test]
fn transpile_lineage_postgres_fn() {
    let src = "fn insert(sql: String) -> Result<String, String> !Postgres {\n\
               Postgres.execute_raw(sql, \"[]\")\n}";
    let prog = Parser::parse_str(src, "<test>").unwrap();
    let report = lineage_analysis(&prog);
    let mut comments = HashMap::new();
    for entry in &report.transformations {
        let effects = if entry.effects.is_empty() { "Pure".to_string() }
                      else { entry.effects.join(", ") };
        comments.insert(entry.name.clone(),
            format!("# [lineage] effects: {} | sources: - | sinks: -", effects));
    }
    let out = emit_python_with_lineage(&prog, "<test>", comments);
    assert!(out.contains("!Postgres"), "Postgres in lineage comment:\n{}", out);
}
```

---

## Phase F — バージョン更新・コミット

- `fav/Cargo.toml`: `version = "11.8.0"`
- `cargo build` で `Cargo.lock` 更新
- `git commit & push` — CI 確認
