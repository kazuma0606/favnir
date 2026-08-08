# v63.7.0 Plan — パイプライン DAG 最適化（dead stage elimination + pure stage fusion）

Version: 63.7.0
Status: 未着手

---

## 実装順序

### Step 1: `driver.rs` — ヘルパー関数 3 件 + `cmd_opt_stats` 追加

`cmd_parallel_stats` の直後（または `cmd_build_aot_validate` の直前）に以下を追加:

```rust
// ── Optimizer helpers (v63.7.0) ────────────────────────────────────────────────

fn opt_is_pure_stage(td: &crate::ast::TrfDef) -> bool {
    !opt_block_has_effect_call(&td.body)
}

fn opt_block_has_effect_call(block: &crate::ast::Block) -> bool {
    if opt_expr_has_effect_call(&block.expr) {
        return true;
    }
    block.stmts.iter().any(|s| match s {
        crate::ast::Stmt::Expr(e) => opt_expr_has_effect_call(e),
        crate::ast::Stmt::Bind(b) => opt_expr_has_effect_call(&b.expr),
        _ => false,
    })
}

fn opt_expr_has_effect_call(expr: &crate::ast::Expr) -> bool {
    match expr {
        crate::ast::Expr::FieldAccess(obj, _, _) => {
            if let crate::ast::Expr::Ident(ns, _) = obj.as_ref() {
                matches!(
                    ns.as_str(),
                    "Io" | "Http" | "Db" | "Kafka" | "S3" | "Sqs"
                        | "Slack" | "Email" | "Llm" | "Snowflake" | "Postgres"
                )
            } else {
                opt_expr_has_effect_call(obj)
            }
        }
        crate::ast::Expr::Apply(callee, args, _) => {
            opt_expr_has_effect_call(callee)
                || args.iter().any(|a| opt_expr_has_effect_call(a))
        }
        crate::ast::Expr::Pipeline(steps, _) => {
            steps.iter().any(|s| opt_expr_has_effect_call(s))
        }
        _ => false,
    }
}

/// v63.7.0: パイプライン DAG 最適化の静的解析。
/// dead stage（PipelineDef 未参照 TrfDef）と pure stage fusion 候補を報告する。
pub fn cmd_opt_stats(src: &str) -> String {
    use crate::frontend::parser::Parser;
    let program = match Parser::parse_str(src, "<opt>") {
        Ok(p) => p,
        Err(e) => return format!("parse error: {e}"),
    };

    let trf_names: Vec<String> = program.items.iter()
        .filter_map(|item| {
            if let crate::ast::Item::TrfDef(td) = item { Some(td.name.clone()) } else { None }
        })
        .collect();

    let mut pipeline_refs: std::collections::HashSet<String> = std::collections::HashSet::new();
    for item in &program.items {
        if let crate::ast::Item::PipelineDef(pd) = item {
            for step in &pd.steps {
                pipeline_refs.insert(step.seq_name.clone());
            }
        }
    }

    let mut lines: Vec<String> = Vec::new();
    let mut eliminated = 0usize;

    for name in &trf_names {
        if !pipeline_refs.contains(name) {
            lines.push(format!(
                "[optimizer] stage `{}` has no downstream consumers — eliminated", name
            ));
            eliminated += 1;
        }
    }

    let trf_defs: Vec<&crate::ast::TrfDef> = program.items.iter()
        .filter_map(|item| if let crate::ast::Item::TrfDef(td) = item { Some(td) } else { None })
        .collect();

    let mut fused = 0usize;
    let mut pure_run: Vec<&str> = Vec::new();
    for td in &trf_defs {
        if opt_is_pure_stage(td) {
            pure_run.push(&td.name);
        } else {
            if pure_run.len() >= 2 {
                lines.push(format!(
                    "[optimizer] stages `{}` fused (all pure) — 1 stage emitted",
                    pure_run.join(" -> ")
                ));
                fused += 1;
            }
            pure_run.clear();
        }
    }
    if pure_run.len() >= 2 {
        lines.push(format!(
            "[optimizer] stages `{}` fused (all pure) — 1 stage emitted",
            pure_run.join(" -> ")
        ));
        fused += 1;
    }

    lines.push(format!("optimizer: {} eliminated, {} fused", eliminated, fused));
    lines.join("\n")
}
```

`cargo build` でエラーなしを確認。

### Step 2: `driver.rs` — `v63700_tests` 追加

`v63600_tests` の直前に挿入:

```rust
// -- v63700_tests (v63.7.0) -- パイプライン DAG 最適化（dead stage elimination + pure stage fusion）--
#[cfg(test)]
mod v63700_tests {
    #[test]
    fn optimizer_dead_stage_eliminated() {
        let src = concat!(
            "public stage LoadCsv: Int -> Int = |x| { x }\n",
            "public stage Unused: Int -> Int = |x| { x + 1 }\n",
            "pipeline Main {\n",
            "    step \"load\" = seq LoadCsv\n",
            "}"
        );
        let out = crate::driver::cmd_opt_stats(src);
        assert!(
            out.contains("Unused"),
            "dead stage Unused should be reported: {}", out
        );
        assert!(
            out.contains("eliminated"),
            "output should mention 'eliminated': {}", out
        );
        assert!(
            !out.contains("`LoadCsv` has no downstream"),
            "LoadCsv is in pipeline — should NOT be eliminated: {}", out
        );
    }

    #[test]
    fn optimizer_pure_stages_fused() {
        let src = concat!(
            "public stage Normalize: Int -> Int = |x| { x + 1 }\n",
            "public stage Trim: Int -> Int = |x| { x - 1 }\n",
            "pipeline P {\n",
            "    step \"n\" = seq Normalize\n",
            "    step \"t\" = seq Trim after \"n\"\n",
            "}"
        );
        let out = crate::driver::cmd_opt_stats(src);
        assert!(
            out.contains("fused"),
            "consecutive pure stages should be reported as fusable: {}", out
        );
        assert!(
            out.contains("Normalize"),
            "output should mention Normalize: {}", out
        );
    }
}
```

`cargo build` でエラーなしを確認。

### Step 3: テスト・ドキュメント更新

```bash
cargo test --bin fav v63700_tests   # 2 件 PASS を確認
cargo test -j 8 -- --test-threads=8  # 3423 tests passed, 0 failed を確認
```

1. `CHANGELOG.md` 先頭に v63.7.0 エントリを追加
2. `versions/roadmap/roadmap-v63.1-v64.0.md` v63.7.0 セクションに実績追記
3. `versions/current.md` の「進行中」を v63.7.0（3423 tests）に更新
4. `tasks.md` を COMPLETE に更新

---

## 設計メモ

### 変更対象ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `opt_is_pure_stage` / `opt_block_has_effect_call` / `opt_expr_has_effect_call` ヘルパー + `cmd_opt_stats` 追加 + `v63700_tests` 追加 |

### `cmd_parallel_stats` の挿入位置

`cmd_parallel_stats` は `cmd_run_with_cache` の直後に存在する（v63.4.0 追加）。
`cmd_opt_stats` はその直後に追加する（`cmd_build_aot_validate` の直前）。

### `opt_` プレフィックス

ヘルパー関数に `opt_` プレフィックスを付けることで、`driver.rs` 内の既存ヘルパーとの命名の混乱を防ぐ。
（`lint.rs` の関数は別モジュールにあるため Rust の名前解決上の衝突はないが、`driver.rs` 内部での意図の明確化のために付与する。）

### ベーステスト数の差異

ロードマップ記載: base=3418、target=3420。
実際: v63.6.0 code-reviewer 対応で +3 件追加されたため base=3421、target=3423。
