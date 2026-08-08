# v63.7.0 Spec — パイプライン DAG 最適化（dead stage elimination + pure stage fusion）

Version: 63.7.0
Status: 未着手

---

## 概要

`driver.rs` に `cmd_opt_stats(src: &str) -> String` を追加し、パイプライン最適化の静的解析を実装する。
2 種類の最適化候補を検出してレポートする:

1. **Dead stage elimination**: `PipelineDef` のどのステップにも参照されていない `TrfDef` を「未使用（dead）」として報告する。
2. **Pure stage fusion**: エフェクトのない（エフェクトフル名前空間への呼び出しを含まない）連続する `TrfDef` ペアを「融合可能（fusable）」として報告する。

出力例:
```
[optimizer] stage `Unused` has no downstream consumers — eliminated
[optimizer] stages `Normalize -> Trim` fused (all pure) — 1 stage emitted
optimizer: 1 eliminated, 1 fused
```

---

## 前提確認（T0 で実施）

- `cargo test -j 8 -- --test-threads=8` でベース 3421 tests passed, 0 failed を確認
- `driver.rs` に `v63600_tests` が存在することを確認（`v63700_tests` の挿入位置確認）
- `driver.rs` に `cmd_opt_stats` が存在しないことを確認（新規追加）
- `ast.rs` の `PipelineDef.steps[*].seq_name` が stage 名を保持していることを確認（dead stage 検出の根拠）
- `ast.rs` の `TrfDef` に `effects` フィールドが存在しないことを確認（v35.6.0 で削除済み。pure 判定は body の AST 解析で行う）

**ロードマップとの差異（重要）**:
- ロードマップ原案は `compiler.rs` への DAG パス追加と `petgraph` 活用を記載しているが、本バージョンでは `driver.rs` への `cmd_opt_stats` として実装する（分析のみ、実際のコンパイル変更なし）。
- `petgraph` 活用・`compiler.rs` 統合・`fav run --opt-stats` CLI フラグは非スコープ（後送り）。
- ロードマップのベーステスト数（3418）は v63.6.0 code-reviewer 対応 +3 件により 3421 に変更。

---

## 実装スコープ

### 1. `driver.rs` — `cmd_opt_stats` + ヘルパー関数

#### ① `cmd_opt_stats(src: &str) -> String`（公開関数）

```rust
pub fn cmd_opt_stats(src: &str) -> String {
    use crate::frontend::parser::Parser;
    let program = match Parser::parse_str(src, "<opt>") {
        Ok(p) => p,
        Err(e) => return format!("parse error: {e}"),
    };

    // TrfDef 名の収集
    let trf_names: Vec<String> = program.items.iter()
        .filter_map(|item| if let crate::ast::Item::TrfDef(td) = item { Some(td.name.clone()) } else { None })
        .collect();

    // PipelineDef で参照されている stage 名の収集
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

    // Dead stage elimination
    for name in &trf_names {
        if !pipeline_refs.contains(name) {
            lines.push(format!(
                "[optimizer] stage `{}` has no downstream consumers — eliminated", name
            ));
            eliminated += 1;
        }
    }

    // Pure stage fusion（ソース順で連続する純粋 TrfDef ペアを検出）
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

#### ② ヘルパー関数 3 件（`cmd_opt_stats` の直前に配置）

```rust
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
```

### 2. `driver.rs` — `v63700_tests` 追加

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

---

## 完了条件

- `cargo build` エラーなし
- `cargo test --bin fav v63700_tests` で 2 件 PASS
  - `optimizer_dead_stage_eliminated` PASS（Unused が eliminated、LoadCsv が eliminated に含まれない）
  - `optimizer_pure_stages_fused` PASS（Normalize -> Trim が fused）
- `cargo test -j 8 -- --test-threads=8` で 3423 tests passed, 0 failed

---

## 非スコープ

- `compiler.rs` への DAG パス統合（実際のコンパイル最適化）
- `petgraph` を使った本格的な DAG 解析
- `fav run --opt-stats` CLI フラグ追加（`main.rs` 更新）
- エフェクト検出の網羅性向上（現在は主要名前空間のみ）
- W042（バックプレッシャー実行時警告）— v63.6.0 の非スコープ継続

---

## 技術ノート

### TrfDef に effects フィールドがない件

v35.6.0 で `Effect` enum と `effects` フィールドが AST から削除されている。
「純粋ステージ」の判定は `TrfDef.body` の AST を走査し、エフェクトフル名前空間（`Io.*` / `Http.*` 等）への `FieldAccess` + `Apply` パターンを検出することで行う。

ヘルパー関数名は `opt_` プレフィックスを付けて `driver.rs` 内の既存ヘルパーとの命名の混乱を防ぐ。
（`lint.rs` 関数は別モジュールのため Rust の名前解決上の衝突はない。）

### `TrfDef.body` は Block（Lambda ラップなし）

`parse_trf_def`（`parser.rs` 2162〜2205行）は `= |params| { body }` を以下のように分解する:
- `|params|` → `TrfDef.params: Vec<Param>`（`parse_closure_params_typed()` で解析）
- `{ body }` → `TrfDef.body: Block`（`parse_block()` で直接解析）

すなわち `public stage Name: T -> T = |x| { x + 1 }` の body は `Block { expr: BinOp(+, Ident("x"), Lit(1)), stmts: [] }` となる。
`Expr::Lambda` でラップされることはないため、`opt_block_has_effect_call` が正しく body を走査できる。

### dead stage の定義

`PipelineDef.steps[*].seq_name` に含まれない `TrfDef` が "dead"。
`PipelineDef` が存在しないプログラム（関数のみ、ステージ定義のみ）では全 `TrfDef` が dead 扱いになる。
これは意図的な設計：パイプラインに組み込まれていない stage は実行経路がない。

### `optimizer_dead_stage_eliminated` テストの負の assertion

テスト内の `` !out.contains("`LoadCsv` has no downstream") `` はフォーマット文字列の部分文字列を使った検証であり、フォーマットが変わるとサイレントに通過するリスクがある。
このトレードオフは許容範囲の設計判断だが、将来のフォーマット変更時には同時にテストを更新すること。

### pure stage fusion の定義

ソース順で隣接する `TrfDef` アイテムが 2 件以上連続して pure である場合に fusion 候補として報告。
「純粋」= body 中に既知エフェクトフル名前空間への `FieldAccess` が存在しない。
