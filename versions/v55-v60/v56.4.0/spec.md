# Spec — v56.4.0 — エフェクト推論 LSP 統合（inlay hints 表示）

## 概要

関数ボディ内で呼び出されているエフェクト系ネームスペース（`IO`, `Snowflake`, `Http` 等）を
静的解析で収集し、LSP の `textDocument/inlayHint` にインライン表示する。
また `infer_fn_effects` ヘルパー（`--show-types` / テスト向け）を追加する。

---

## ロードマップ参照

- `versions/roadmap/roadmap-v56.1-v57.0.md` — v56.4.0 セクション
- `versions/roadmap/roadmap-v55.1-v60.0.md` — v56.4.0 行
- ベーステスト数: **3233**（v56.3.0 完了時点の実績値）
- 目標テスト数: **3235**（+2）

---

## 既存実装との関係

| 要素 | バージョン | 状態 |
|------|-----------|------|
| `infer_effects_fn` / `Effect enum` | v32.9 実装 → **v35.5.0 削除** | 削除済み（本バージョンでは不使用） |
| `propagate_transitive_effects` | v35.x で no-op 化 | no-op（本バージョンでは不使用） |
| `AMBIENT_NAMESPACES` / `collect_ambient_in_expr` | v13.1.0 | 実装済み（本バージョンで活用） |
| `check_ambient_effects` / `check_ambient_errors` | v13.1.0 | 実装済み（本バージョンで参照） |
| `handle_inlay_hints` + 4 collectors | v46.4.0 〜 v50.5.0 | 実装済み（本バージョンで拡張） |
| `collect_inference_annotations` | v43.9.0 | 実装済み（本バージョンで参照） |

---

## スコープの明確化

### `infer_effects_fn` を使わない根拠

ロードマップには「`infer_effects_fn` の結果を LSP に統合」と記載されているが、
`infer_effects_fn` は v35.5.0 に `Effect enum` と共に削除されている。
したがって本バージョンでは `infer_effects_fn` を復元せず、
**既存の `AMBIENT_NAMESPACES` + AST ウォーク**で同等の効果収集を実現する。

- `AMBIENT_NAMESPACES` = `["IO", "Postgres", "AWS", "Snowflake", "Http", "Grpc", "Llm", "Queue", "Cache", "Slack", "Email"]`
- `AMBIENT_GEN_FNS` = `["uuid_raw", "uuid_v7_raw", "nano_id"]`
- `lint.rs` の `collect_ambient_in_expr` と同様の AST ウォークで「どの NS が呼ばれているか」を収集する

完全な型レベルエフェクト推論（Effect union type 等）は将来スプリントで対応する。

---

## 実装内容

### 1. `fav/Cargo.toml` バージョン更新

```toml
version = "56.4.0"
```

---

### 2. `fav/src/lint.rs` — `collect_used_namespaces` 追加

`check_ambient_errors` の直後（L737 付近）に追加する。

```rust
/// v56.4.0: Collect effectful namespace names called in a fn/block body.
/// Uses the same AMBIENT_NAMESPACES / AMBIENT_GEN_FNS lists as W008/E0023.
/// Returns a sorted, deduplicated list of namespace names.
pub fn collect_used_namespaces(block: &crate::ast::Block) -> Vec<String> {
    let mut found = std::collections::BTreeSet::new();
    collect_ns_in_block(block, &mut found);
    found.into_iter().collect()
}
```

また非公開ヘルパー `collect_ns_in_block` / `collect_ns_in_expr` を追加する。
これらは `collect_ambient_in_block` / `collect_ambient_in_expr` の構造を踏襲し、
エラーを emit する代わりに NS 名を `BTreeSet<String>` に追加する。

`collect_ns_in_expr` の構造は以下の通り:

```rust
fn collect_ns_in_expr(expr: &Expr, found: &mut std::collections::BTreeSet<String>) {
    match expr {
        Expr::Apply(func, args, _) => {
            if let Expr::FieldAccess(base, method, _) = func.as_ref() {
                if let Expr::Ident(ns, _) = base.as_ref() {
                    let is_ambient = AMBIENT_NAMESPACES.contains(&ns.as_str())
                        || (ns == "Gen" && AMBIENT_GEN_FNS.contains(&method.as_str()));
                    if is_ambient {
                        found.insert(ns.clone());
                    }
                }
            }
            collect_ns_in_expr(func, found);
            for a in args { collect_ns_in_expr(a, found); }
        }
        Expr::Block(b) => collect_ns_in_block(b, found),
        Expr::If(cond, then, else_, _) => {
            collect_ns_in_expr(cond, found);
            collect_ns_in_block(then, found);
            if let Some(eb) = else_ { collect_ns_in_block(eb, found); }
        }
        Expr::Match(scrutinee, arms, _) => {
            collect_ns_in_expr(scrutinee, found);
            for arm in arms { collect_ns_in_expr(&arm.body, found); }
        }
        Expr::Pipeline(steps, _) => {
            for s in steps { collect_ns_in_expr(s, found); }
        }
        Expr::FieldAccess(obj, _, _) => collect_ns_in_expr(obj, found),
        Expr::BinOp(_, l, r, _) => {
            collect_ns_in_expr(l, found);
            collect_ns_in_expr(r, found);
        }
        Expr::Closure(_, body, _) => collect_ns_in_expr(body, found),
        Expr::Collect(b, _) => collect_ns_in_block(b, found),
        Expr::EmitExpr(inner, _) => collect_ns_in_expr(inner, found),
        Expr::Question(inner, _) => collect_ns_in_expr(inner, found),
        Expr::AssertMatches(e, _, _) => collect_ns_in_expr(e, found),
        Expr::AssertSchema { arg, .. } => collect_ns_in_expr(arg, found),
        Expr::TypeApply(f, _, _) => collect_ns_in_expr(f, found),
        Expr::RecordConstruct(_, fields, _) => {
            for (_, v) in fields { collect_ns_in_expr(v, found); }
        }
        Expr::RecordSpread(base, updates, _) => {
            collect_ns_in_expr(base, found);
            for (_, v) in updates { collect_ns_in_expr(v, found); }
        }
        Expr::FString(parts, _) => {
            for part in parts {
                if let FStringPart::Expr(e) = part { collect_ns_in_expr(e, found); }
            }
        }
        Expr::ListComp { expr, clauses, .. } | Expr::ResultComp { expr, clauses, .. } => {
            collect_ns_in_expr(expr, found);
            for c in clauses {
                match c {
                    CompClause::For { src, .. } => collect_ns_in_expr(src, found),
                    CompClause::Guard(g) => collect_ns_in_expr(g, found),
                }
            }
        }
        Expr::Lit(..) | Expr::Ident(..) => {}
    }
}
```

---

### 3. `fav/src/lsp/inlay_hints.rs` — `collect_fn_effect_hints` 追加

`collect_pipeline_type_hints` の直後（v50.5.0 セクション末尾）に追加。

```rust
// v56.4.0: fn エフェクト inlay hints ─────────────────────────────────────────

/// v56.4.0: Collect inlay hints showing inferred effect namespaces for fn definitions.
///
/// For each `fn` definition whose body calls at least one effectful namespace
/// (IO, Snowflake, Http, etc.), emits an InlayHint with label `" /* !NS1 !NS2 */"`
/// placed at the end of the fn definition line.
///
/// Parses the source internally (does not require type_at map).
/// Known limitation: uses fd.span.line (1-based) to find the hint line;
/// multi-line fn signatures may show the hint on the wrong line.
pub(crate) fn collect_fn_effect_hints(source: &str) -> Vec<InlayHint> {
    use crate::ast::Item;
    use crate::frontend::parser::Parser;

    let program = match Parser::parse_str(source, "<effect-hints>") {
        Ok(p) => p,
        Err(_) => return vec![],
    };

    let mut hints = Vec::new();
    for item in &program.items {
        if let Item::FnDef(fd) = item {
            let namespaces = crate::lint::collect_used_namespaces(&fd.body);
            if namespaces.is_empty() {
                continue;
            }
            let labels: Vec<String> = namespaces.iter().map(|ns| format!("!{}", ns)).collect();
            let label = format!(" /* {} */", labels.join(" "));
            // fd.span.line は 1-based; source.lines() は 0-based index
            let line_idx = fd.span.line.saturating_sub(1) as usize;
            if let Some(line_str) = source.lines().nth(line_idx) {
                hints.push(InlayHint {
                    position: Position {
                        line: line_idx as u32,
                        character: line_str.len() as u32,
                    },
                    label,
                    kind: 1,
                });
            }
        }
    }
    hints
}
```

`handle_inlay_hints` に以下を追加（`collect_pipeline_type_hints` の直後）:

```rust
// v56.4.0: fn エフェクト inlay hints
hints.extend(collect_fn_effect_hints(&doc.source));
```

---

### 4. `fav/src/driver.rs` — `infer_fn_effects` 追加

`collect_inference_annotations` の直後に追加。

```rust
/// v56.4.0: Collect effectful namespaces per fn for --show-types output and tests.
/// Does NOT require the checker to pass (parse-only).
/// Returns vec of (fn_name, [namespace1, ...]) for all FnDef items.
pub fn infer_fn_effects(src: &str) -> Vec<(String, Vec<String>)> {
    use crate::ast::Item;

    let program = match crate::frontend::parser::Parser::parse_str(src, "<effects>") {
        Ok(p) => p,
        Err(_) => return vec![],
    };
    program
        .items
        .iter()
        .filter_map(|item| {
            if let Item::FnDef(fd) = item {
                let effects = crate::lint::collect_used_namespaces(&fd.body);
                Some((fd.name.clone(), effects))
            } else {
                None
            }
        })
        .collect()
}
```

---

### 5. `fav/src/driver.rs` — `v56400_tests` 追加 + v56300_tests 更新

#### 5a. `v56300_tests::cargo_toml_version_is_56_3_0` を削除

Cargo.toml が `56.4.0` に更新されるため削除。

#### 5b. `v56400_tests` モジュールを `v56300_tests` の直前に挿入

```rust
// -- v56400_tests (v56.4.0) -- エフェクト推論 LSP 統合 --
#[cfg(test)]
mod v56400_tests {
    #[test]
    fn cargo_toml_version_is_56_4_0() {
        let cargo_toml = include_str!("../Cargo.toml");
        assert!(
            cargo_toml.contains("version = \"56.4.0\""),
            "Cargo.toml version should be 56.4.0, got: {}",
            cargo_toml.lines().find(|l| l.contains("version")).unwrap_or("")
        );
    }

    #[test]
    fn effect_inference_inlay_hint() {
        // IO.println を呼ぶ fn に対して "IO" を含む inlay hint が生成されることを確認
        use crate::lsp::inlay_hints::collect_fn_effect_hints;
        let source = r#"
fn print_hello() {
    IO.println("hello")
}
"#;
        let hints = collect_fn_effect_hints(source);
        assert!(
            hints.iter().any(|h| h.label.contains("IO")),
            "Expected inlay hint containing 'IO', got: {:?}",
            hints
        );
    }

    #[test]
    fn effect_inference_check_output() {
        // infer_fn_effects が IO.println を持つ fn の effects に "IO" を含むことを確認
        use crate::driver::infer_fn_effects;
        let source = r#"
fn print_hello() {
    IO.println("hello")
}
fn pure_add(a: Int, b: Int) -> Int {
    a + b
}
"#;
        let result = infer_fn_effects(source);
        let print_hello = result.iter().find(|(name, _)| name == "print_hello");
        assert!(print_hello.is_some(), "Expected 'print_hello' in results, got: {:?}", result);
        let (_, effects) = print_hello.unwrap();
        assert!(
            effects.contains(&"IO".to_string()),
            "Expected 'IO' in effects of print_hello, got: {:?}",
            effects
        );
        // pure_add はエフェクトなし
        let pure_add = result.iter().find(|(name, _)| name == "pure_add");
        assert!(pure_add.is_some(), "Expected 'pure_add' in results");
        let (_, pure_effects) = pure_add.unwrap();
        assert!(
            pure_effects.is_empty(),
            "Expected no effects for pure_add, got: {:?}",
            pure_effects
        );
    }
}
```

---

## テスト仕様

| テスト名 | 検証内容 |
|---------|---------|
| `cargo_toml_version_is_56_4_0` | Cargo.toml が `56.4.0` を反映 |
| `effect_inference_inlay_hint` | `collect_fn_effect_hints` が `IO.println` を持つ fn に対し "IO" を含む hint を生成 |
| `effect_inference_check_output` | `infer_fn_effects` が `IO.println` を持つ fn の effects に "IO" を返し、純粋関数は effects 空 |

---

## 完了条件

- `cargo build` コンパイルエラーなし
- `cargo test` 全通過（**3235 tests passed, 0 failed**）
- `cargo clippy -- -D warnings` クリーン
- `cargo_toml_version_is_56_4_0` pass
- `effect_inference_inlay_hint` pass（"IO" を含む InlayHint が生成される）
- `effect_inference_check_output` pass（`infer_fn_effects` が "IO" を返し純粋関数は空）
- `lint.rs` に `collect_used_namespaces` が追加されている（pub）
- `lsp/inlay_hints.rs` に `collect_fn_effect_hints` が追加されている（pub(crate)）
- `handle_inlay_hints` に `collect_fn_effect_hints` の呼び出しが追加されている
- `driver.rs` に `infer_fn_effects` が追加されている（pub）
- `v56300_tests::cargo_toml_version_is_56_3_0` が削除されている
- `CHANGELOG.md` に v56.4.0 エントリが追加されている（version: `56.3.0 → 56.4.0`）
- `versions/current.md` が v56.4.0 / 3235 tests を反映
- 両ロードマップの v56.4.0 実績を COMPLETE に更新

---

## 備考

- **`infer_effects_fn` 非使用の根拠**: v35.5.0 で `Effect enum` と共に削除済み。
  `AMBIENT_NAMESPACES` + AST ウォークによる静的 NS 収集で代替する。
- **`collect_used_namespaces` は `collect_ambient_in_expr` の収集専用バリアント**:
  `lint.rs` 既存のエラー emit ロジックに手を加えず、新しい関数として並置する。
  将来 `collect_ambient_in_block` をリファクタリングで統合してもよいが v56.4 スコープ外。
- **inlay hint 位置の制限**: `fd.span.line` は fn キーワードの行（1-based）。
  マルチライン fn シグネチャでは hint が fn 先頭行の末尾に付く。
  既存 `collect_fn_return_hints` と同様の Known Limitation として許容する。
- **`infer_fn_effects` は checker 非依存**: パース成功のみで動作するため、
  型エラーがあるソースに対しても効果収集を返す。`collect_inference_annotations`（checker 依存）
  とは明示的に分離する。
- **`effect_inference_check_output` テストで `infer_fn_effects` を使う理由**:
  `collect_inference_annotations` は checker.fav が通らないと空を返すため、
  テストの安定性を優先して checker 非依存の `infer_fn_effects` を直接テストする。
- **ロードマップのテスト数**: `3233 + 2 = 3235` は正確（-1 削除 +3 追加 = net +2）。
