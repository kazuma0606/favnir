# v22.5.0 実装計画 — Pipeline Orchestration（DAG スケジューリング）

## 実装順序

```
T1（lexer.rs）         ← 最初（T2/T3 の依存元）
T2（ast.rs）           ← T1 完了後
T3（parser.rs）        ← T2 完了後
T4（driver.rs）        ← T3 完了後
T5（main.rs）          ← T4 完了後
T6（Cargo + doc）      ← T5 完了後
```

---

## T1: `fav/src/frontend/lexer.rs` — `TokenKind::Pipeline` 追加

### 事前確認コマンド

```bash
grep -n "Seq\|Bench\|Alias\|As," fav/src/frontend/lexer.rs | head -10
```

### 1-1: `TokenKind` enum に `Pipeline` を追加

`TokenKind::As` の直後（`// Effect keywords` の前）に追加:

```rust
    As,
    Pipeline,  // v22.5.0: `pipeline` block keyword

    // Effect keywords
```

### 1-2: キーワードマッチに追加

`"as" => TokenKind::As,` の直後に追加:

```rust
"as" => TokenKind::As,
"pipeline" => TokenKind::Pipeline,
```

### 確認

```bash
cargo check --bin fav
# pipeline が identifier として lexing されなくなる（既存テストに影響なし）
```

---

## T2: `fav/src/ast.rs` — `PipelineStep` / `PipelineDef` + `Item::PipelineDef`

### 事前確認コマンド

```bash
grep -n "TriggerAnnotation\|pub enum Item\|FlwDef(FlwDef)" fav/src/ast.rs | head -10
```

### 2-1: `PipelineStep` / `PipelineDef` struct 追加

`TriggerAnnotation` ブロックの直後（`// ── FnDef` の前）に追加:

```rust
// ── PipelineDef (v22.5.0) ─────────────────────────────────────────────────────

/// v22.5.0: A single step in a pipeline DAG.
#[derive(Debug, Clone)]
pub struct PipelineStep {
    /// Step ラベル（`step "load_raw"` の `"load_raw"`）
    pub name: String,
    /// 実行する seq 宣言名（`= seq LoadRaw` の `LoadRaw`）
    pub seq_name: String,
    /// このステップが依存するステップ名（`after "load_raw"` の `"load_raw"`）
    pub after: Vec<String>,
    pub span: Span,
}

/// v22.5.0: `pipeline Name { step ... }` ブロック。
#[derive(Debug, Clone)]
pub struct PipelineDef {
    pub name: String,
    pub steps: Vec<PipelineStep>,
    pub span: Span,
}
```

### 2-2: `Item::PipelineDef` を `Item` enum に追加

`Item::FlwDef(FlwDef),` の直後に追加:

```rust
    FlwDef(FlwDef),
    PipelineDef(PipelineDef),  // v22.5.0
    AbstractFlwDef(AbstractFlwDef),
```

### 2-3: `Item::span()` に `PipelineDef` アームを追加（**必須**）

`ast.rs` の `Item::span()` は `_` arm のない exhaustive match。`UseAlias { span, .. }` アームの直後に追加:

```rust
    Item::PipelineDef(pd) => &pd.span,
```

### 2-4: `fmt.rs` の `fmt_item` に `PipelineDef` アームを追加（**必須**）

`fav/src/fmt.rs` の `fmt_item` 末尾（`UseAlias` アームの直後）に追加:

```rust
    Item::PipelineDef(_) => None, // v22.5.0: fmt 未対応（スタブ）
```

### 2-5: `checker.rs` の `check_item` に `PipelineDef` アームを追加（**必須**）

`fav/src/middle/checker.rs` の `check_item` match 末尾に追加（`_` arm がない exhaustive match）:

```rust
    Item::PipelineDef(_) => {} // v22.5.0: 型チェック未対応（スタブ）
```

### 2-6: `compiler.rs` の Item ループに `PipelineDef` アームを追加（**必須**）

`fav/src/middle/compiler.rs` の Item ループ内に追加（`_` arm の有無を `cargo check` で確認してから対応）:

```rust
    Item::PipelineDef(_) => {} // v22.5.0: コンパイル未対応（スタブ）
```

### 確認

```bash
cargo check --bin fav
# 上記 2-3〜2-6 の対応後、コンパイルエラーが 0 になること
```

---

## T3: `fav/src/frontend/parser.rs` — `parse_pipeline_def` / `parse_pipeline_step` + `parse_item` 適用

### 事前確認コマンド

```bash
grep -n "parse_flw_def_or_binding\|TokenKind::Seq\b\|TokenKind::Bench\|fn parse_bench" fav/src/frontend/parser.rs | head -10
```

### 3-1: `parse_pipeline_step` メソッドを追加

`parse_flw_def` メソッドの直前に追加（`// ── FlwDef parsing` 付近）:

```rust
/// v22.5.0: parse `step "<name>" = seq <SeqName> [after "<dep>", ...]`
fn parse_pipeline_step(&mut self) -> Result<crate::ast::PipelineStep, ParseError> {
    let start = self.peek_span().clone();
    // step — soft keyword
    self.expect_ident_name("step")?;
    let name = self.expect_str()?;
    self.expect(&TokenKind::Eq)?;
    self.expect(&TokenKind::Seq)?;
    let (seq_name, _) = self.expect_ident()?;
    // optional: after "<dep1>", "<dep2>"
    let mut after = Vec::new();
    if self.peek_ident_text("after") {
        self.advance(); // consume "after"
        let dep = self.expect_str()?;
        after.push(dep);
        while self.peek() == &TokenKind::Comma {
            self.advance(); // ,
            if !matches!(self.peek(), TokenKind::Str(_)) { break; } // trailing comma
            after.push(self.expect_str()?);
        }
    }
    Ok(crate::ast::PipelineStep { name, seq_name, after, span: self.span_from(&start) })
}

/// v22.5.0: parse `pipeline <Name> { step ... }`
fn parse_pipeline_def(&mut self) -> Result<crate::ast::PipelineDef, ParseError> {
    let start = self.peek_span().clone();
    self.expect(&TokenKind::Pipeline)?;
    let (name, _) = self.expect_ident()?;
    self.expect(&TokenKind::LBrace)?;
    let mut steps = Vec::new();
    while self.peek() != &TokenKind::RBrace && !self.at_end() {
        steps.push(self.parse_pipeline_step()?);
    }
    self.expect(&TokenKind::RBrace)?;
    Ok(crate::ast::PipelineDef { name, steps, span: self.span_from(&start) })
}
```

### 3-2: `parse_item()` に `TokenKind::Pipeline` ブランチを追加

`TokenKind::Seq` ブランチの直前に追加:

```rust
            TokenKind::Pipeline => Ok(Item::PipelineDef(self.parse_pipeline_def()?)),
            TokenKind::Seq => {
```

### 確認

```bash
cargo check --bin fav
```

---

## T4: `fav/src/driver.rs` — `build_topo_order` + `cmd_orchestrate_*` + `v225000_tests`

### 事前確認コマンド

```bash
grep -n "// ── v22.4.0\|pub fn cmd_deploy_trigger\|v224000_tests" fav/src/driver.rs | head -5
```

### 4-1: `build_topo_order` + `cmd_orchestrate_run` / `cmd_orchestrate_status` / `cmd_orchestrate_retry` を追加

`build_trigger_config_json` / `cmd_deploy_trigger` ブロックの直後に追加:

```rust
// ── v22.5.0: Pipeline Orchestration ──────────────────────────────────────────

/// Kahn's algorithm でステップのトポロジカルソート順（インデックス配列）を返す。
/// 循環依存がある場合は `Err(循環を含むステップ名)` を返す。
pub(crate) fn build_topo_order(steps: &[crate::ast::PipelineStep]) -> Result<Vec<usize>, String> {
    use std::collections::HashMap;
    let n = steps.len();
    // name → index
    let idx: HashMap<&str, usize> = steps.iter().enumerate()
        .map(|(i, s)| (s.name.as_str(), i))
        .collect();
    // in-degree
    let mut in_deg = vec![0usize; n];
    // adjacency: edges[i] = i が完了したら unblock できる step インデックス
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (i, step) in steps.iter().enumerate() {
        for dep in &step.after {
            match idx.get(dep.as_str()) {
                Some(&j) => {
                    adj[j].push(i);
                    in_deg[i] += 1;
                }
                None => return Err(format!("step `{}` depends on unknown step `{}`", step.name, dep)),
            }
        }
    }
    // BFS (Kahn)
    let mut queue: std::collections::VecDeque<usize> = in_deg.iter().enumerate()
        .filter(|(_, &d)| d == 0)
        .map(|(i, _)| i)
        .collect();
    let mut order = Vec::with_capacity(n);
    while let Some(cur) = queue.pop_front() {
        order.push(cur);
        for &next in &adj[cur] {
            in_deg[next] -= 1;
            if in_deg[next] == 0 {
                queue.push_back(next);
            }
        }
    }
    if order.len() != n {
        // 処理されていないステップ = 循環
        let cycle_steps: Vec<&str> = steps.iter().enumerate()
            .filter(|(i, _)| !order.contains(i))
            .map(|(_, s)| s.name.as_str())
            .collect();
        return Err(format!("circular dependency detected among steps: {}", cycle_steps.join(", ")));
    }
    Ok(order)
}

fn find_pipeline_def<'a>(
    prog: &'a crate::ast::Program,
    name: &str,
) -> Option<&'a crate::ast::PipelineDef> {
    prog.items.iter().find_map(|item| {
        if let crate::ast::Item::PipelineDef(pd) = item {
            if pd.name == name { Some(pd) } else { None }
        } else {
            None
        }
    })
}

pub fn cmd_orchestrate_run(file: &str, pipeline_name: &str, dry_run: bool) {
    let src = load_file(file);
    let prog = Parser::parse_str(&src, file).unwrap_or_else(|e| {
        eprintln!("{}", e);
        process::exit(1);
    });
    let pd = find_pipeline_def(&prog, pipeline_name).unwrap_or_else(|| {
        eprintln!("error: pipeline `{}` not found in {}", pipeline_name, file);
        process::exit(1);
    });
    let order = build_topo_order(&pd.steps).unwrap_or_else(|e| {
        eprintln!("error: {}", e);
        process::exit(1);
    });
    println!("[orchestrate] pipeline: {}", pipeline_name);
    println!("[orchestrate] execution order ({} steps):", order.len());
    for (rank, &idx) in order.iter().enumerate() {
        let step = &pd.steps[idx];
        println!("  {}. \"{}\" → seq {}", rank + 1, step.name, step.seq_name);
    }
    if dry_run {
        println!("[orchestrate] dry-run mode — no steps executed");
        return;
    }
    // ステータス追跡（HashSet<String> でライフタイム問題を回避）
    let mut step_results: Vec<(String, String, String, u64)> = Vec::new(); // (name, seq, status, elapsed_ms)
    let mut failed: std::collections::HashSet<String> = std::collections::HashSet::new();
    let run_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    for &idx in &order {
        let step = &pd.steps[idx];
        // 依存 step が失敗していたら skip
        if step.after.iter().any(|dep| failed.contains(dep.as_str())) {
            println!("[orchestrate] skip \"{}\" (dependency failed)", step.name);
            step_results.push((step.name.clone(), step.seq_name.clone(), "skip".into(), 0));
            failed.insert(step.name.clone());
            continue;
        }
        println!("[orchestrate] run \"{}\" → seq {}", step.name, step.seq_name);
        let t0 = std::time::Instant::now();
        // seq を実行（ファイル内で seq <SeqName> を探して cmd_run に渡す形式はスタブ）
        // v22.5.0: ここでは seq 名を stdout に出力するのみ（実際の実行は v22.6+ で統合）
        let elapsed = t0.elapsed().as_millis() as u64;
        println!("[orchestrate] ok \"{}\" ({}ms)", step.name, elapsed);
        step_results.push((step.name.clone(), step.seq_name.clone(), "ok".into(), elapsed));
    }
    // ステータス JSON を .fav_orchestrate/ に保存
    let dir = ".fav_orchestrate";
    let _ = std::fs::create_dir_all(dir);
    let ts = chrono::Utc::now().format("%Y%m%d%H%M%S");
    let path = format!("{}/{}_{}.json", dir, pipeline_name, ts);
    let steps_json: Vec<String> = step_results.iter().map(|(n, s, st, e)| {
        format!(
            "    {{ \"name\": \"{}\", \"seq\": \"{}\", \"status\": \"{}\", \"elapsed_ms\": {} }}",
            n, s, st, e
        )
    }).collect();
    let json = format!(
        "{{\n  \"pipeline\": \"{}\",\n  \"run_at\": \"{}\",\n  \"steps\": [\n{}\n  ]\n}}",
        pipeline_name,
        run_at,
        steps_json.join(",\n")
    );
    let _ = std::fs::write(&path, &json);
    println!("[orchestrate] status saved → {}", path);
}

pub fn cmd_orchestrate_status(pipeline_name: &str) {
    let dir = ".fav_orchestrate";
    let prefix = format!("{}_", pipeline_name);
    let mut entries: Vec<std::path::PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|_| {
            eprintln!("error: .fav_orchestrate/ not found — run `fav orchestrate run` first");
            process::exit(1);
        })
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with(&prefix) && n.ends_with(".json"))
                .unwrap_or(false)
        })
        .collect();
    entries.sort();
    let latest = entries.last().unwrap_or_else(|| {
        eprintln!("error: no status found for pipeline `{}`", pipeline_name);
        process::exit(1);
    });
    let content = std::fs::read_to_string(latest).unwrap_or_else(|e| {
        eprintln!("error: {}", e);
        process::exit(1);
    });
    println!("{}", content);
}

pub fn cmd_orchestrate_retry(step_name: &str, file: &str, pipeline_name: &str) {
    let src = load_file(file);
    let prog = Parser::parse_str(&src, file).unwrap_or_else(|e| {
        eprintln!("{}", e);
        process::exit(1);
    });
    let pd = find_pipeline_def(&prog, pipeline_name).unwrap_or_else(|| {
        eprintln!("error: pipeline `{}` not found in {}", pipeline_name, file);
        process::exit(1);
    });
    let step = pd.steps.iter().find(|s| s.name == step_name).unwrap_or_else(|| {
        eprintln!("error: step `{}` not found in pipeline `{}`", step_name, pipeline_name);
        process::exit(1);
    });
    println!("[orchestrate] retry \"{}\" → seq {}", step.name, step.seq_name);
    // v22.5.0: スタブ — 実際の seq 実行は v22.6+ で統合
    println!("[orchestrate] ok (retry stub)");
}
```

**注意**: `chrono` はすでに `Cargo.toml` に含まれる（`cmd_deploy` で使用済み）。確認してから使うこと。

### 4-2: `v224000_tests::version_is_22_4_0` に `#[ignore]` を追加

### 4-3: `v225000_tests` モジュールを追加

```rust
// ── v225000_tests (v22.5.0) — Pipeline Orchestration ─────────────────────────
#[cfg(test)]
mod v225000_tests {
    use super::*;

    #[test]
    fn version_is_22_5_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("version = \"22.5.0\""), "Cargo.toml should have version 22.5.0");
    }

    #[test]
    fn pipeline_def_parsed() {
        let src = r#"
pipeline DailyETL {
  step "load" = seq Load
  step "transform" = seq Transform after "load"
}
"#;
        let tokens = crate::frontend::lexer::Lexer::new(src, "test.fav")
            .tokenize()
            .expect("lex failed");
        let prog = crate::frontend::parser::Parser::new(tokens)
            .parse_program()
            .expect("parse failed");
        assert_eq!(prog.items.len(), 1);
        if let crate::ast::Item::PipelineDef(pd) = &prog.items[0] {
            assert_eq!(pd.name, "DailyETL");
            assert_eq!(pd.steps.len(), 2);
            assert_eq!(pd.steps[0].name, "load");
            assert_eq!(pd.steps[0].seq_name, "Load");
            assert!(pd.steps[0].after.is_empty());
            assert_eq!(pd.steps[1].name, "transform");
            assert_eq!(pd.steps[1].seq_name, "Transform");
            assert_eq!(pd.steps[1].after, vec!["load"]);
        } else {
            panic!("expected PipelineDef item");
        }
    }

    #[test]
    fn pipeline_dag_topo_order() {
        use crate::ast::PipelineStep;
        let span = crate::frontend::lexer::Span::new("test", 0, 0, 1, 1);
        let steps = vec![
            PipelineStep { name: "a".into(), seq_name: "A".into(), after: vec![], span: span.clone() },
            PipelineStep { name: "b".into(), seq_name: "B".into(), after: vec!["a".into()], span: span.clone() },
            PipelineStep { name: "c".into(), seq_name: "C".into(), after: vec!["b".into()], span: span.clone() },
        ];
        let order = crate::driver::build_topo_order(&steps).expect("topo sort should succeed");
        // a=0 → b=1 → c=2 の順序であること
        assert_eq!(order, vec![0, 1, 2]);
    }

    #[test]
    fn pipeline_dag_cycle_detected() {
        use crate::ast::PipelineStep;
        let span = crate::frontend::lexer::Span::new("test", 0, 0, 1, 1);
        let steps = vec![
            PipelineStep { name: "a".into(), seq_name: "A".into(), after: vec!["b".into()], span: span.clone() },
            PipelineStep { name: "b".into(), seq_name: "B".into(), after: vec!["a".into()], span: span.clone() },
        ];
        let result = crate::driver::build_topo_order(&steps);
        assert!(result.is_err(), "cycle should be detected");
        let msg = result.unwrap_err();
        assert!(msg.contains("circular"), "error should mention circular: {}", msg);
    }

    #[test]
    fn changelog_has_v22_5_0() {
        let cl = include_str!("../../CHANGELOG.md");
        assert!(cl.contains("[v22.5.0]"), "CHANGELOG should have v22.5.0 entry");
    }
}
```

### 確認

```bash
cargo test v225000 --bin fav   # 5/5 PASS を確認
cargo test --bin fav           # リグレッションなし（1860 件以上）確認
```

---

## T5: `fav/src/main.rs` — `fav orchestrate` CLI 追加

### 事前確認コマンド

```bash
grep -n "Some(\"deploy\")\|Some(\"mcp\")\|Some(\"notebook\")" fav/src/main.rs | head -5
```

### 5-1: `fav orchestrate` サブコマンドを追加

`Some("deploy")` ブランチの直後（または適切な位置）に追加:

```rust
Some("orchestrate") => {
    match args.get(2).map(|s| s.as_str()) {
        Some("run") => {
            let pipeline_name = args.get(3).unwrap_or_else(|| {
                eprintln!("error: orchestrate run requires <PipelineName>");
                process::exit(1);
            });
            let file = args.get(4).unwrap_or_else(|| {
                eprintln!("error: orchestrate run requires <file>");
                process::exit(1);
            });
            let dry_run = args.iter().any(|a| a == "--dry-run");
            crate::driver::cmd_orchestrate_run(file, pipeline_name, dry_run);
        }
        Some("status") => {
            let pipeline_name = args.get(3).unwrap_or_else(|| {
                eprintln!("error: orchestrate status requires <PipelineName>");
                process::exit(1);
            });
            crate::driver::cmd_orchestrate_status(pipeline_name);
        }
        Some("retry") => {
            let step_name = args.get(3).unwrap_or_else(|| {
                eprintln!("error: orchestrate retry requires <StepName>");
                process::exit(1);
            });
            let pipeline_name = args.get(4).unwrap_or_else(|| {
                eprintln!("error: orchestrate retry requires <PipelineName>");
                process::exit(1);
            });
            let file = args.get(5).unwrap_or_else(|| {
                eprintln!("error: orchestrate retry requires <file>");
                process::exit(1);
            });
            crate::driver::cmd_orchestrate_retry(step_name, file, pipeline_name);
        }
        _ => {
            eprintln!("usage: fav orchestrate run <PipelineName> <file> [--dry-run]");
            eprintln!("       fav orchestrate status <PipelineName>");
            eprintln!("       fav orchestrate retry <StepName> <PipelineName> <file>");
            process::exit(1);
        }
    }
}
```

### 確認

```bash
cargo check --bin fav
```

---

## T6: `fav/Cargo.toml` + `CHANGELOG.md` + MDX

### 6-1: バージョン更新

```
version = "22.4.0" → "22.5.0"
```

### 6-2: CHANGELOG に v22.5.0 エントリを先頭に追加

```markdown
## [v22.5.0] — 2026-06-21 — Pipeline Orchestration（DAG スケジューリング）
```

### 6-3: `site/content/docs/cli/orchestrate.mdx` を新規作成

内容:
- `pipeline` ブロック構文と `step` / `after` の説明
- `fav orchestrate run/status/retry` の使用例
- ステータス JSON フォーマット
- DAG の依存解決（トポロジカルソート）の説明
- 将来の拡張（並列実行・リモートデプロイ）への言及

---

## 主要な落とし穴・注意事項

1. **`TokenKind::Pipeline` 追加の影響**: `"pipeline"` という識別子を変数名として使っているユーザーコードが壊れる可能性がある。ただし実際のユーザーコードへの影響は軽微。

2. **`Item::PipelineDef` の exhaustive match**: `Item` enum に新しいバリアントを追加するため、`Item` を `match` しているすべての箇所でコンパイルエラーが出る可能性がある。`cargo check` でリストアップして、各 `_` arm に委ねるか `PipelineDef` を明示処理する。

3. **`pipeline_dag_topo_order` テストの順序**: Kahn's アルゴリズムは in-degree 0 の node を BFS キューに追加する。入力が `[a, b, c]` で a→b→c の依存の場合、`a`（in-deg 0）がキューに最初に入るため `[0, 1, 2]` が返る。ただし、in-deg 0 が複数ある場合は BFS キューへの追加順序（index 順）に依存する。テストはシンプルな線形チェーンを使う。

4. **`chrono` の使用確認**: `cmd_orchestrate_run` 内で `chrono::Utc::now()` を使う。`cmd_deploy` で既に使用されているので依存関係は不要だが、import が必要な場合は `use chrono;` または `chrono::Utc::now()` を直接呼ぶ。

5. **`cmd_orchestrate_run` の seq 実行スタブ**: v22.5.0 では `seq <SeqName>` を実際には実行せず、stdout に表示するのみ。テストはトポロジカルソートのロジックのみを検証する（`pipeline_dag_topo_order`、`pipeline_dag_cycle_detected`）。

6. **`cmd_orchestrate_retry` の引数順序**: `fav orchestrate retry <StepName> <PipelineName> <file>` — ステップ名が最初に来る点に注意（args[3] = step_name, args[4] = pipeline_name, args[5] = file）。
