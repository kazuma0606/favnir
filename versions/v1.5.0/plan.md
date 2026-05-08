# Favnir v1.5.0 実装計画 — CI/CD 統合 + 静的解析強化

作成日: 2026-05-08

> **実装順序**: Phase 0 → 3 → 4 → 1 → 2 → 5
> Phase 3（ユーザー定義エフェクト）は AST/パーサー変更を伴うため先に実施する。
> Phase 1/2 は driver.rs の追加のみで依存関係が少ない。

---

## Phase 0 — バージョン更新

```toml
# Cargo.toml
version = "1.5.0"
```

```rust
// main.rs の HELP 定数
const HELP: &str = r#"
Favnir Compiler v1.5.0

Usage: fav <command> [options] [file]

Commands:
  run [file]                  Run a Favnir source file
  build [-o <file>] [file]    Compile to .fvc artifact
  exec [--db path] [--info] <file.fvc>
                              Execute a compiled artifact
  check [file]                Type-check without running
  explain [--schema] [--format <text|json>] [--focus <all|fns|trfs|flws|types>] [file]
                              Show type signatures and effects
  explain diff [--format <text|json>] <from> <to>
                              Compare two explain outputs
  bundle [-o <file>] [--entry <name>] [--manifest] [--explain] <file>
                              Build minimal reachable artifact
  graph [--format <text|mermaid>] [--focus <flw|fn>] [--entry <name>] [--depth <n>] <file>
                              Show flow/function dependency graph
  test [file]                 Run .test.fav files
  fmt [--check] [file]        Format Favnir source files
  lint [--warn-only] [file]   Lint Favnir source files
  install                     Install dependencies from fav.toml
  publish                     Publish rune to registry
  lsp                         Start Language Server (JSON-RPC on stdin/stdout)
"#;
```

---

## Phase 3 — ユーザー定義エフェクト（AST/Parser/Checker 変更）

### 3-1. `ast.rs` の変更

```rust
// ast.rs に追加
#[derive(Debug, Clone)]
pub struct EffectDef {
    pub visibility: Option<Visibility>,
    pub name: String,
    pub span: Span,
}

// Item に追加
pub enum Item {
    // ... 既存バリアント ...
    EffectDef(EffectDef),
}

// Item::span() に追加
Item::EffectDef(e) => &e.span,
```

### 3-2. `lexer.rs` の変更

```rust
// TokenKind に追加
Effect,

// keyword map に追加
"effect" => TokenKind::Effect,

// test_keywords に追加（existing test 関数内）
("effect", TokenKind::Effect),
```

### 3-3. `parser.rs` の変更

```rust
// parse_item の冒頭に追加（Abstract の前後）
Some(TokenKind::Effect) => {
    self.advance();
    Ok(Item::EffectDef(self.parse_effect_def(visibility)?))
}

// 新規関数
fn parse_effect_def(&mut self, visibility: Option<Visibility>) -> Result<EffectDef, ParseError> {
    let start = self.current_span();
    let name = self.expect_ident("effect name")?;
    let end = self.prev_span();
    Ok(EffectDef {
        visibility,
        name,
        span: start.merge(end),
    })
}
```

### 3-4. `checker.rs` の変更

```rust
// Checker 構造体に追加
pub effect_registry: HashSet<String>,

// new() / new_with_resolver() で初期化
effect_registry: HashSet::new(),

// first_pass に追加
Item::EffectDef(e) => {
    self.effect_registry.insert(e.name.clone());
}

// 組み込みエフェクト定数
const BUILTIN_EFFECTS: &[&str] = &["Io", "Db", "File", "Trace", "Emit"];

// 新規: エフェクト検証関数
fn check_effects_declared(&mut self, effects: &[Effect], span: Span) {
    for eff in effects {
        match eff {
            Effect::Unknown(name) => {
                if !BUILTIN_EFFECTS.contains(&name.as_str())
                    && !self.effect_registry.contains(name)
                {
                    self.errors.push(CheckError {
                        code: "E052".into(),
                        message: format!(
                            "undeclared effect `{}` — declare with `effect {}` at top level",
                            name, name
                        ),
                        span,
                    });
                }
            }
            _ => {} // 組み込みエフェクトは常に OK
        }
    }
}

// check_fn_def / check_trf_def / check_flw_def / check_abstract_trf_def 内で呼ぶ:
self.check_effects_declared(&fn_def.effects, fn_def.span);
```

### 3-5. `driver.rs` の変更（explain JSON への追加）

```rust
// ExplainPrinter::render_json 内の serde_json::json! ブロックに追加:
"custom_effects": program.items.iter()
    .filter_map(|item| {
        if let ast::Item::EffectDef(e) = item {
            Some(serde_json::json!({
                "name": e.name,
                "public": e.visibility.is_some(),
            }))
        } else {
            None
        }
    })
    .collect::<Vec<_>>(),
```

---

## Phase 4 — `fav lint` 強化

### 4-1. `lint.rs` の変更

```rust
// collect_trf_flw_uses: 使用されている trf/flw 名を収集
fn collect_trf_flw_uses(program: &ast::Program) -> HashSet<String> {
    let mut used = HashSet::new();
    for item in &program.items {
        match item {
            // FlwDef のステップ名
            ast::Item::FlwDef(f) => {
                for step in &f.steps {
                    used.insert(step.clone());
                }
            }
            // FlwBindingDef: template 名とスロット実装名
            ast::Item::FlwBindingDef(fb) => {
                used.insert(fb.template.clone());
                for (_, slot_impl) in &fb.bindings {
                    match slot_impl {
                        ast::SlotImpl::Global(name) | ast::SlotImpl::Local(name) => {
                            used.insert(name.clone());
                        }
                    }
                }
            }
            // fn/trf 本体内の呼び出し（AST レベルの簡易走査）
            ast::Item::FnDef(f) => collect_expr_calls(&f.body, &mut used),
            ast::Item::TrfDef(t) => collect_expr_calls(&t.body, &mut used),
            _ => {}
        }
    }
    used
}

// collect_expr_calls: AST 式から Ident 参照名を収集（再帰）
fn collect_expr_calls(expr: &ast::Expr, used: &mut HashSet<String>) {
    match expr {
        ast::Expr::Ident(name, _) => { used.insert(name.clone()); }
        ast::Expr::Apply(f, args, _) => {
            collect_expr_calls(f, used);
            for a in args { collect_expr_calls(a, used); }
        }
        ast::Expr::Block(stmts, tail, _) => {
            for stmt in stmts { collect_stmt_calls(stmt, used); }
            collect_expr_calls(tail, used);
        }
        // ... 他のバリアント（If/Match/FieldAccess 等）
        _ => {}
    }
}

// lint_program に L005/L006/L007 を追加
pub fn lint_program(program: &ast::Program, source: &str) -> Vec<LintWarning> {
    let mut warnings = vec![];
    let used = collect_trf_flw_uses(program);

    for item in &program.items {
        match item {
            // L002 (既存): 未使用 bind
            // L003 (既存): fn 名 snake_case
            // L004 (既存): type 名 PascalCase

            // L005: 未参照の private trf/flw
            ast::Item::TrfDef(t) if t.visibility.is_none() && !used.contains(&t.name) => {
                warnings.push(LintWarning {
                    code: "L005".into(),
                    message: format!("trf `{}` is defined but never referenced", t.name),
                    span: t.span,
                });
            }
            ast::Item::AbstractTrfDef(t) if t.visibility.is_none() && !used.contains(&t.name) => {
                warnings.push(LintWarning {
                    code: "L005".into(),
                    message: format!("abstract trf `{}` is defined but never referenced", t.name),
                    span: t.span,
                });
            }
            ast::Item::FlwDef(f) if f.visibility.is_none() && !used.contains(&f.name) => {
                warnings.push(LintWarning {
                    code: "L005".into(),
                    message: format!("flw `{}` is defined but never referenced", f.name),
                    span: f.span,
                });
            }
            ast::Item::AbstractFlwDef(f) if f.visibility.is_none() && !used.contains(&f.name) => {
                warnings.push(LintWarning {
                    code: "L005".into(),
                    message: format!("abstract flw `{}` is defined but never referenced", f.name),
                    span: f.span,
                });
            }

            // L006: trf 名が PascalCase でない
            ast::Item::TrfDef(t) if !is_pascal_case(&t.name) => {
                warnings.push(LintWarning {
                    code: "L006".into(),
                    message: format!(
                        "trf `{}` should be PascalCase (e.g. `{}`)",
                        t.name, to_pascal_case(&t.name)
                    ),
                    span: t.span,
                });
            }

            // L007: effect 名が PascalCase でない
            ast::Item::EffectDef(e) if !is_pascal_case(&e.name) => {
                warnings.push(LintWarning {
                    code: "L007".into(),
                    message: format!(
                        "effect `{}` should be PascalCase (e.g. `{}`)",
                        e.name, to_pascal_case(&e.name)
                    ),
                    span: e.span,
                });
            }

            _ => {}
        }
    }
    warnings
}
```

---

## Phase 1 — `fav explain diff`

### 1-1. データ構造

```rust
// driver.rs に追加

#[derive(Debug, Default)]
pub struct ExplainDiff {
    pub from_label: String,
    pub to_label:   String,
    pub fn_changes:       CategoryDiff,
    pub trf_changes:      CategoryDiff,
    pub flw_changes:      CategoryDiff,
    pub type_changes:     CategoryDiff,
    pub effects_added:    Vec<String>,
    pub effects_removed:  Vec<String>,
    pub breaking_changes: Vec<String>,
}

#[derive(Debug, Default)]
pub struct CategoryDiff {
    pub added:   Vec<serde_json::Value>,
    pub removed: Vec<serde_json::Value>,
    pub changed: Vec<ChangedEntry>,
}

#[derive(Debug)]
pub struct ChangedEntry {
    pub name:  String,
    pub diffs: Vec<String>,
}
```

### 1-2. 比較ロジック

```rust
fn diff_explain_json(
    from: &serde_json::Value,
    to: &serde_json::Value,
) -> ExplainDiff {
    let mut diff = ExplainDiff::default();

    // カテゴリごとに比較: fns, trfs, flws, types
    for key in &["fns", "trfs", "flws", "types"] {
        let from_map: HashMap<String, serde_json::Value> = value_array_to_map(from[key].as_array());
        let to_map:   HashMap<String, serde_json::Value> = value_array_to_map(to[key].as_array());
        let category_diff = diff_category(&from_map, &to_map);

        // 破壊的変更の判定
        let breaking = detect_breaking_changes(*key, &category_diff);
        diff.breaking_changes.extend(breaking);

        match *key {
            "fns"   => diff.fn_changes   = category_diff,
            "trfs"  => diff.trf_changes  = category_diff,
            "flws"  => diff.flw_changes  = category_diff,
            "types" => diff.type_changes = category_diff,
            _ => {}
        }
    }

    // effects_used の差分
    let from_fx: HashSet<String> = json_array_to_strings(&from["effects_used"]);
    let to_fx:   HashSet<String> = json_array_to_strings(&to["effects_used"]);
    diff.effects_added   = to_fx.difference(&from_fx).cloned().collect();
    diff.effects_removed = from_fx.difference(&to_fx).cloned().collect();

    diff
}

fn diff_category(
    from: &HashMap<String, serde_json::Value>,
    to:   &HashMap<String, serde_json::Value>,
) -> CategoryDiff {
    let mut cat = CategoryDiff::default();
    for (name, to_entry) in to {
        match from.get(name) {
            None => cat.added.push(to_entry.clone()),
            Some(from_entry) => {
                let diffs = diff_entry(from_entry, to_entry);
                if !diffs.is_empty() {
                    cat.changed.push(ChangedEntry { name: name.clone(), diffs });
                }
            }
        }
    }
    for (name, from_entry) in from {
        if !to.contains_key(name) {
            cat.removed.push(from_entry.clone());
        }
    }
    cat
}

// 1エントリの差分（return_type/effects/params を比較）
fn diff_entry(from: &serde_json::Value, to: &serde_json::Value) -> Vec<String> {
    let mut diffs = vec![];
    for field in &["return_type", "input_type", "output_type", "effects", "params"] {
        if from[field] != to[field] {
            diffs.push(format!(
                "{}: {} -> {}",
                field,
                json_to_display(&from[field]),
                json_to_display(&to[field])
            ));
        }
    }
    diffs
}

// 破壊的変更の判定
fn detect_breaking_changes(category: &str, diff: &CategoryDiff) -> Vec<String> {
    let mut breaking = vec![];
    for removed in &diff.removed {
        if let Some(name) = removed["name"].as_str() {
            breaking.push(format!("{} `{}` removed", &category[..category.len()-1], name));
        }
    }
    for changed in &diff.changed {
        let breaking_fields = ["return_type", "input_type", "output_type", "effects", "params"];
        for diff_str in &changed.diffs {
            if breaking_fields.iter().any(|f| diff_str.starts_with(f)) {
                breaking.push(format!("`{}` signature changed: {}", changed.name, diff_str));
            }
        }
    }
    breaking
}
```

### 1-3. テキスト/JSON レンダリング

```rust
fn render_diff_text(diff: &ExplainDiff) -> String {
    let mut out = String::new();
    out.push_str(&format!("--- {}\n+++ {}\n\n", diff.from_label, diff.to_label));

    let total_added   = diff.fn_changes.added.len() + diff.trf_changes.added.len()
                      + diff.flw_changes.added.len() + diff.type_changes.added.len();
    let total_removed = diff.fn_changes.removed.len() + diff.trf_changes.removed.len()
                      + diff.flw_changes.removed.len() + diff.type_changes.removed.len();
    let total_changed = diff.fn_changes.changed.len() + diff.trf_changes.changed.len()
                      + diff.flw_changes.changed.len() + diff.type_changes.changed.len();

    if total_added == 0 && total_removed == 0 && total_changed == 0 {
        out.push_str("No changes detected.\n");
        return out;
    }

    for (label, cat) in &[
        ("[fns]",   &diff.fn_changes),
        ("[trfs]",  &diff.trf_changes),
        ("[flws]",  &diff.flw_changes),
        ("[types]", &diff.type_changes),
    ] {
        if cat.added.is_empty() && cat.removed.is_empty() && cat.changed.is_empty() { continue; }
        out.push_str(&format!("{}\n", label));
        for a in &cat.added   { out.push_str(&format!("+ {}\n", entry_signature(a))); }
        for r in &cat.removed { out.push_str(&format!("- {}\n", entry_signature(r))); }
        for c in &cat.changed {
            for d in &c.diffs { out.push_str(&format!("~ {}   {}\n", c.name, d)); }
        }
        out.push('\n');
    }

    out.push_str(&format!(
        "[summary]\n  added: {}, removed: {}, changed: {}\n",
        total_added, total_removed, total_changed
    ));
    if !diff.breaking_changes.is_empty() {
        out.push_str(&format!(
            "  breaking changes: {}\n",
            diff.breaking_changes.join(", ")
        ));
    }
    out
}

fn render_diff_json(diff: &ExplainDiff) -> String {
    serde_json::to_string_pretty(&serde_json::json!({
        "schema_version": "1.0",
        "from": diff.from_label,
        "to":   diff.to_label,
        "changes": {
            "fns":   category_diff_to_json(&diff.fn_changes),
            "trfs":  category_diff_to_json(&diff.trf_changes),
            "flws":  category_diff_to_json(&diff.flw_changes),
            "types": category_diff_to_json(&diff.type_changes),
            "effects_used": {
                "added":   diff.effects_added,
                "removed": diff.effects_removed,
            }
        },
        "summary": {
            "total_added":   diff.fn_changes.added.len() + diff.trf_changes.added.len()
                             + diff.flw_changes.added.len() + diff.type_changes.added.len(),
            "total_removed": diff.fn_changes.removed.len() + diff.trf_changes.removed.len()
                             + diff.flw_changes.removed.len() + diff.type_changes.removed.len(),
            "total_changed": diff.fn_changes.changed.len() + diff.trf_changes.changed.len()
                             + diff.flw_changes.changed.len() + diff.type_changes.changed.len(),
            "breaking_changes": diff.breaking_changes,
        }
    })).unwrap_or_default()
}
```

### 1-4. `cmd_explain_diff` の実装

```rust
pub fn cmd_explain_diff(from_path: &str, to_path: &str, format: &str) {
    let from_json = load_explain_json(from_path);
    let to_json   = load_explain_json(to_path);
    let mut diff  = diff_explain_json(&from_json, &to_json);
    diff.from_label = from_path.to_string();
    diff.to_label   = to_path.to_string();

    let output = match format {
        "json" => render_diff_json(&diff),
        _      => render_diff_text(&diff),
    };
    print!("{output}");
}

// 入力パスの種別を判定して explain JSON を取得
fn load_explain_json(path: &str) -> serde_json::Value {
    if path.ends_with(".json") {
        let content = std::fs::read_to_string(path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or(serde_json::Value::Object(Default::default()))
    } else if path.ends_with(".fvc") {
        let artifact = read_artifact_from_path(Path::new(path)).unwrap_or_else(|e| {
            eprintln!("{e}"); process::exit(1);
        });
        match explain_json_from_artifact(&artifact) {
            Ok(json_str) => serde_json::from_str(json_str).unwrap_or_default(),
            Err(e) => { eprintln!("{e}"); process::exit(1); }
        }
    } else {
        // .fav ソースファイル
        let (program, _) = load_and_check_program(Some(path));
        let ir = compile_program(&program);
        let reachability = reachability_analysis("main", &ir);
        let json_str = ExplainPrinter::new().render_json(&program, Some(&ir), false, "all", Some(&reachability));
        serde_json::from_str(&json_str).unwrap_or_default()
    }
}
```

### 1-5. `main.rs` のルーティング

```rust
// main.rs: explain コマンド内に "diff" サブコマンドを追加
Some("explain") => {
    if args.get(1).map(|s| s.as_str()) == Some("diff") {
        let from = args.get(2).map(String::as_str).unwrap_or_else(|| {
            eprintln!("error: explain diff requires <from> <to> paths");
            process::exit(1);
        });
        let to = args.get(3).map(String::as_str).unwrap_or_else(|| {
            eprintln!("error: explain diff requires <from> <to> paths");
            process::exit(1);
        });
        let mut format = "text";
        let mut i = 4;
        while i < args.len() {
            if args[i] == "--format" {
                format = args.get(i+1).map(String::as_str).unwrap_or("text");
                i += 2;
            } else { i += 1; }
        }
        cmd_explain_diff(from, to, format);
    } else {
        // 既存の explain ルーティング
        // ...
    }
}
```

---

## Phase 2 — `fav graph --focus fn`

### 2-1. `reachability.rs` への追加

```rust
// fn 呼び出し依存グラフを構築: fn名 → 呼び出し先 fn名リスト
pub fn collect_fn_calls_from_ir(ir: &IRProgram) -> HashMap<String, Vec<String>> {
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();
    for fn_def in &ir.fns {
        let calls = collect_calls_in_ir(fn_def, &ir.globals);
        graph.insert(fn_def.name.clone(), calls);
    }
    graph
}
```

### 2-2. `driver.rs` のグラフレンダラー拡張

```rust
fn render_graph_text(program: &ast::Program, focus: &str) -> String {
    if focus == "fn" {
        // fn 依存ツリーを描画
        return render_fn_graph_text(program);
    }
    // 既存 flw グラフ描画 (変更なし)
    // ...
}

fn render_fn_graph_text_with_opts(
    program: &ast::Program,
    entry: &str,
    max_depth: usize,
) -> String {
    // IR コンパイルして fn 呼び出しグラフを取得
    let ir = compile_program(program);
    let call_graph = crate::middle::reachability::collect_fn_calls_from_ir(&ir);

    let mut out = format!("fn dependencies from: {}\n", entry);
    let mut visited = HashSet::new();
    render_fn_tree(&call_graph, entry, 0, max_depth, &mut visited, &mut out, &ir);
    out
}

fn render_fn_tree(
    graph: &HashMap<String, Vec<String>>,
    name: &str,
    depth: usize,
    max_depth: usize,
    visited: &mut HashSet<String>,
    out: &mut String,
    ir: &IRProgram,
) {
    let indent = "  ".repeat(depth);
    let effects = fn_effects_str(ir, name);
    let suffix = if effects.is_empty() { String::new() } else { format!("    {}", effects) };

    if visited.contains(name) {
        out.push_str(&format!("{}[cycle: {}]{}\n", indent, name, suffix));
        return;
    }
    out.push_str(&format!("{}{}{}\n", indent, name, suffix));
    visited.insert(name.to_string());

    if depth >= max_depth { return; }
    if let Some(callees) = graph.get(name) {
        for callee in callees {
            render_fn_tree(graph, callee, depth + 1, max_depth, out, visited, ir);
        }
    }
    visited.remove(name);
}

fn render_graph_mermaid(program: &ast::Program, focus: &str) -> String {
    if focus == "fn" {
        return render_fn_graph_mermaid(program);
    }
    // 既存 flw mermaid (変更なし)
    // ...
}

fn render_fn_graph_mermaid_with_opts(
    program: &ast::Program,
    entry: &str,
    max_depth: usize,
) -> String {
    let ir = compile_program(program);
    let call_graph = crate::middle::reachability::collect_fn_calls_from_ir(&ir);

    let mut out = String::from("flowchart LR\n");
    let mut visited = HashSet::new();
    let mut edges = vec![];
    let mut effect_nodes = vec![];  // エフェクトを持つノード

    collect_mermaid_edges(
        &call_graph, entry, 0, max_depth,
        &mut visited, &mut edges, &mut effect_nodes, &ir
    );

    for edge in &edges {
        out.push_str(&format!("  {}\n", edge));
    }
    for node in &effect_nodes {
        out.push_str(&format!("  style {} fill:#f8d7da\n", node));
    }
    out
}
```

### 2-3. `cmd_graph` のオプション拡張

```rust
pub fn cmd_graph(file: &str, format: &str, focus: Option<&str>, entry: Option<&str>, depth: Option<usize>) {
    let focus = focus.unwrap_or("flw");
    let entry = entry.unwrap_or("main");
    let max_depth = depth.unwrap_or(usize::MAX);

    let source = load_file(file);
    let program = Parser::parse_str(&source, file).unwrap_or_else(|e| {
        eprintln!("{e}"); process::exit(1);
    });

    let rendered = match format {
        "mermaid" => render_graph_mermaid_with_opts(&program, focus, entry, max_depth),
        _         => render_graph_text_with_opts(&program, focus, entry, max_depth),
    };
    print!("{rendered}");
}
```

### 2-4. `main.rs` の変更

```rust
Some("graph") => {
    // 既存フラグに追加
    let mut entry: Option<String> = None;
    let mut depth: Option<usize> = None;
    // ...
    "--entry" => entry = Some(args[i+1].clone()),
    "--depth" => depth = args[i+1].parse().ok(),
    // ...
    cmd_graph(file, &format, focus.as_deref(), entry.as_deref(), depth);
}
```

---

## Phase 5 — テスト・ドキュメント

### テストの実装例

```rust
// driver.rs のテストモジュール内

#[test]
fn explain_diff_no_changes() {
    let source = r#"
public fn main() -> Int { 42 }
"#;
    let program = Parser::parse_str(source, "test.fav").expect("parse");
    let ir = compile_program(&program);
    let reach = reachability_analysis("main", &ir);
    let json_str = ExplainPrinter::new().render_json(&program, Some(&ir), false, "all", Some(&reach));
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let diff = diff_explain_json(&json, &json);
    let text = render_diff_text(&diff);
    assert!(text.contains("No changes detected."));
}

#[test]
fn explain_diff_fn_added_shows_in_diff() {
    let source_old = r#"public fn main() -> Int { 42 }"#;
    let source_new = r#"
public fn helper() -> Int { 7 }
public fn main() -> Int { helper() }
"#;
    let from_json = source_to_explain_json(source_old);
    let to_json   = source_to_explain_json(source_new);
    let diff = diff_explain_json(&from_json, &to_json);
    assert_eq!(diff.fn_changes.added.len(), 1);
    assert_eq!(diff.fn_changes.added[0]["name"], "helper");
    assert!(diff.breaking_changes.is_empty());
}

#[test]
fn explain_diff_fn_removed_is_breaking() {
    let source_old = r#"
public fn helper() -> Int { 7 }
public fn main() -> Int { helper() }
"#;
    let source_new = r#"public fn main() -> Int { 42 }"#;
    let from_json = source_to_explain_json(source_old);
    let to_json   = source_to_explain_json(source_new);
    let diff = diff_explain_json(&from_json, &to_json);
    assert_eq!(diff.fn_changes.removed.len(), 1);
    assert!(!diff.breaking_changes.is_empty());
    assert!(diff.breaking_changes.iter().any(|b| b.contains("helper")));
}

#[test]
fn graph_fn_text_shows_calls() {
    let source = r#"
fn helper() -> Int { 7 }
public fn main() -> Int { helper() }
"#;
    let program = Parser::parse_str(source, "test.fav").expect("parse");
    let rendered = render_graph_text_with_opts(&program, "fn", "main", usize::MAX);
    assert!(rendered.contains("fn dependencies from: main"));
    assert!(rendered.contains("helper"));
}

#[test]
fn graph_fn_cycle_safe() {
    // 循環参照があっても無限ループしない
    // (チェッカーエラーが出るケースだが、グラフ描画はクラッシュしない)
    let source = r#"
fn a() -> Int { b() }
fn b() -> Int { a() }
public fn main() -> Int { a() }
"#;
    let program = Parser::parse_str(source, "cycle.fav").expect("parse");
    let rendered = render_graph_text_with_opts(&program, "fn", "main", usize::MAX);
    assert!(rendered.contains("[cycle:") || rendered.contains("a") && rendered.contains("b"));
}

#[test]
fn effect_def_parses() {
    let program = Parser::parse_str("effect Payment\n", "test.fav").expect("parse");
    let has_effect = program.items.iter().any(|i| matches!(i, ast::Item::EffectDef(e) if e.name == "Payment"));
    assert!(has_effect);
}

#[test]
fn effect_unknown_e052() {
    let source = r#"
trf ChargeUser: Int -> Int !Payment = |x| x
"#;
    let errors = Checker::check_program_str(source);
    assert!(errors.iter().any(|e| e.code == "E052"), "expected E052, got: {:?}", errors);
}

#[test]
fn effect_declared_no_error() {
    let source = r#"
effect Payment
trf ChargeUser: Int -> Int !Payment = |x| x
"#;
    let errors = Checker::check_program_str(source);
    assert!(errors.iter().all(|e| e.code != "E052"), "unexpected E052");
}

#[test]
fn lint_l005_unused_trf() {
    let source = r#"
trf UnusedTransform: Int -> Int = |x| x
public fn main() -> Int { 42 }
"#;
    let program = Parser::parse_str(source, "test.fav").expect("parse");
    let warnings = lint_program(&program, source);
    assert!(warnings.iter().any(|w| w.code == "L005"), "expected L005");
}

#[test]
fn lint_l005_public_trf_ignored() {
    let source = r#"
public trf Exported: Int -> Int = |x| x
public fn main() -> Int { 42 }
"#;
    let program = Parser::parse_str(source, "test.fav").expect("parse");
    let warnings = lint_program(&program, source);
    assert!(!warnings.iter().any(|w| w.code == "L005"), "unexpected L005 for public trf");
}

#[test]
fn lint_l006_trf_not_pascal() {
    let source = r#"
trf parse_user: Int -> Int = |x| x
"#;
    let program = Parser::parse_str(source, "test.fav").expect("parse");
    let warnings = lint_program(&program, source);
    assert!(warnings.iter().any(|w| w.code == "L006"));
}

#[test]
fn lint_l007_effect_not_pascal() {
    let source = r#"
effect payment_effect
"#;
    let program = Parser::parse_str(source, "test.fav").expect("parse");
    let warnings = lint_program(&program, source);
    assert!(warnings.iter().any(|w| w.code == "L007"));
}
```

### example ファイル

```
// examples/custom_effects.fav
public effect Payment
public effect Notification

type Receipt = {
    amount: Int
    status: String
}

trf ChargeCard: Int -> Receipt !Payment = |amount| {
    Receipt { amount: amount  status: "charged" }
}

trf NotifyUser: String -> Bool !Notification = |msg| {
    true
}

public fn main() -> Bool !Payment !Notification {
    bind receipt <- ChargeCard(100);
    NotifyUser("Payment complete")
}
```

```
// examples/diff_demo/old.fav
public fn greet(name: String) -> String {
    "Hello"
}

public fn main() -> String !Io {
    IO.println(greet("World"));
    "ok"
}
```

```
// examples/diff_demo/new.fav
public fn greet(name: String) -> String !Io {
    IO.println("greeting");
    "Hello"
}

public fn farewell(name: String) -> String {
    "Goodbye"
}

public fn main() -> String !Io {
    IO.println(greet("World"));
    "ok"
}
```

---

## 実装順序まとめ

```
Phase 0: Cargo.toml + main.rs HELP
Phase 3: ast.rs(EffectDef) → lexer.rs(effect) → parser.rs(parse_effect_def) → checker.rs(E052)
Phase 4: lint.rs(L005/L006/L007)
Phase 1: driver.rs(ExplainDiff + diff_explain_json + cmd_explain_diff) → main.rs(explain diff)
Phase 2: reachability.rs(collect_fn_calls_from_ir) → driver.rs(render_fn_graph_*) → main.rs(graph --focus fn)
Phase 5: tests + examples + langspec.md + README.md
```

各フェーズは独立しており、並行実装可能。
Phase 3 のみ AST 変更を伴うため、他フェーズより先に完成させるのが安全。
