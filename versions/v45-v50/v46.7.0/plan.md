# Plan: v46.7.0 — `fav explain --lineage` 2.0

Date: 2026-07-17

---

## ステップ

### Step 1 — `lineage.rs`: `is_dead` + `has_early_return` + `render_lineage_mermaid_with_opts`

**`LineageEntry` に `is_dead: bool` 追加** (line 16 付近):

```rust
pub struct LineageEntry {
    pub name: String,
    pub kind: String,
    pub capability: Option<String>,
    pub effects: Vec<String>,
    pub sources: Vec<String>,
    pub sinks: Vec<String>,
    pub is_dead: bool,  // v46.7.0
}
```

**`has_early_return`** (`collect_sql_literals` 等のヘルパー近辺に追加):

```rust
/// fn/stage ボディのトップレベル stmts に Stmt::Return が存在するかを判定。
/// Phase 1 スコープ: ネストした if/match/for 内は対象外。
fn has_early_return(stmts: &[ast::Stmt]) -> bool {
    stmts.iter().any(|s| matches!(s, ast::Stmt::Return(_)))
}
```

**`TrfDef` ブランチ更新** (line 838 の `transformations.push`):

```rust
transformations.push(LineageEntry {
    name: trf.name.clone(),
    kind: cap_kind,
    capability: cap_name,
    effects,
    sources,
    sinks,
    is_dead: has_early_return(&trf.body.stmts),  // v46.7.0
});
```

**`FnDef` ブランチ更新** (line 904 の `transformations.push`):

```rust
transformations.push(LineageEntry {
    name: fndef.name.clone(),
    kind: cap_kind,
    capability: cap_name,
    effects,
    sources,
    sinks,
    is_dead: has_early_return(&fndef.body.stmts),  // v46.7.0
});
```

**`render_lineage_mermaid` を委譲に変更 + `render_lineage_mermaid_with_opts` 追加**:

`sanitize_mermaid_id` は `lineage.rs` 内（line 1230 付近）に既存定義あり — 同モジュール内で直接使用可。

```rust
pub fn render_lineage_mermaid(report: &LineageReport) -> String {
    render_lineage_mermaid_with_opts(report, false)
}

pub fn render_lineage_mermaid_with_opts(report: &LineageReport, show_dead: bool) -> String {
    let mut out = String::from("flowchart LR\n");
    if show_dead {
        out.push_str("    classDef deadEntry stroke-dasharray:5 5\n");
    }

    for entry in &report.transformations {
        let effects = if entry.effects.is_empty() {
            "Pure".to_string()
        } else {
            entry.effects.iter()
                .map(|e| format!("!{}", e.trim_start_matches('!')))
                .collect::<Vec<_>>()
                .join("+")
        };
        let id = sanitize_mermaid_id(&entry.name);
        out.push_str(&format!("  {}[\"{}<br/>{}\"]\n", id, entry.name, effects));
        if show_dead && entry.is_dead {
            out.push_str(&format!("  class {} deadEntry\n", id));
        }
    }

    for pipeline in &report.pipelines {
        let steps = &pipeline.steps;
        for i in 0..steps.len().saturating_sub(1) {
            let from = sanitize_mermaid_id(&steps[i]);
            let to   = sanitize_mermaid_id(&steps[i + 1]);
            out.push_str(&format!("  {} --> {}\n", from, to));
        }
    }

    out
}
```

---

### Step 2 — `driver.rs`: 既存 `LineageEntry` リテラル修正 + `pub use` + `cmd_explain_lineage`

**既存 `LineageEntry` 構造体リテラルへの `is_dead: false` 追加** (5 箇所):
- line 35159 付近 (`render_lineage_mermaid_basic` テスト、2 件)
- line 35207 付近 (`render_lineage_d2_basic` テスト、1 件)
- line 44066, 44074 付近 (`v37600_tests::make_report()`、2 件)

各箇所に `is_dead: false,` を追加。

**`pub use` ブロック更新** (line 22907 付近):

```rust
pub use crate::lineage::{
    extract_tables_from_sql, lineage_analysis,
    render_lineage_json, render_lineage_text,
    render_lineage_mermaid, render_lineage_mermaid_with_opts,
    render_lineage_d2, render_lineage_dot, render_lineage_svg,
};
```

**`cmd_explain_lineage` シグネチャ変更**:

```rust
pub fn cmd_explain_lineage(file: Option<&str>, format: &str, show_dead: bool) {
    ...
    match format {
        "mermaid" if show_dead => print!("{}", render_lineage_mermaid_with_opts(&report, true)),
        "mermaid"              => print!("{}", render_lineage_mermaid(&report)),
        ...
    }
}
```

---

### Step 3 — `main.rs`: `--show-dead` フラグ追加

`main.rs` line 782 の match アームに追加:

```rust
"--show-dead" => { show_dead = true; i += 1; }
```

`let mut show_dead = false;` の宣言を `format` 宣言の近辺に追加。
`cmd_explain_lineage(file, &format)` → `cmd_explain_lineage(file, &format, show_dead)` に更新。
呼び出し元は `main.rs:800` の 1 箇所のみ。

---

### Step 4 — `driver.rs`: `v467000_tests`

`v466000_tests` の後に追加:

```rust
mod v467000_tests {
    use crate::frontend::parser::Parser;
    use crate::lineage::{lineage_analysis, render_lineage_mermaid_with_opts};

    #[test]
    fn lineage_return_path_is_dead() {
        // has_ctx_param = true (LoadCtx) → LineageEntry が生成される
        // Stmt::Return が存在 → is_dead = true
        let src = "fn Validate(ctx: LoadCtx, rows: List<String>) -> List<String> { return rows }";
        let program = Parser::parse_str(src, "test.fav").expect("parse");
        let report = lineage_analysis(&program);
        assert_eq!(report.transformations.len(), 1, "expected 1 transformation");
        assert_eq!(report.transformations[0].name, "Validate");
        assert!(
            report.transformations[0].is_dead,
            "expected is_dead=true for fn with early return"
        );
    }

    #[test]
    fn lineage_happy_path_active() {
        // early return なし → is_dead = false
        let src = "fn Transform(ctx: WriteCtx, rows: List<String>) -> List<String> { rows }";
        let program = Parser::parse_str(src, "test.fav").expect("parse");
        let report = lineage_analysis(&program);
        assert_eq!(report.transformations.len(), 1, "expected 1 transformation");
        assert!(!report.transformations[0].is_dead, "expected is_dead=false for normal fn");
        // show_dead=true でも Transform ノードに deadEntry クラスが付与されないことを確認
        let rendered = render_lineage_mermaid_with_opts(&report, true);
        assert!(
            !rendered.contains("class Transform deadEntry"),
            "expected no deadEntry class for non-dead fn, got:\n{}",
            rendered
        );
    }
}
```

---

### Step 5 — バージョン更新

- `fav/Cargo.toml`: `46.7.0`
- `CHANGELOG.md`: v46.7.0 エントリ
- `versions/current.md`: v46.7.0（3007 tests）
