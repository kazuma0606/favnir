# Plan — v56.4.0 — エフェクト推論 LSP 統合（inlay hints 表示）

## ゴール

- `lint.rs` に `collect_used_namespaces` を追加（`AMBIENT_NAMESPACES` ベース AST ウォーク）
- `lsp/inlay_hints.rs` に `collect_fn_effect_hints` を追加し `handle_inlay_hints` に組み込む
- `driver.rs` に `infer_fn_effects` を追加（checker 非依存、テスト用）
- 3233 → 3235 tests

---

## 実装ステップ

### Phase 1: Cargo.toml バージョン更新

`56.3.0` → `56.4.0`

---

### Phase 2: `lint.rs` — `collect_used_namespaces` 追加

`check_ambient_errors` 直後（L737 付近）に挿入。

```rust
pub fn collect_used_namespaces(block: &crate::ast::Block) -> Vec<String> {
    let mut found = std::collections::BTreeSet::new();
    collect_ns_in_block(block, &mut found);
    found.into_iter().collect()
}

fn collect_ns_in_block(block: &Block, found: &mut std::collections::BTreeSet<String>) { ... }
fn collect_ns_in_expr(expr: &Expr, found: &mut std::collections::BTreeSet<String>) { ... }
```

`collect_ns_in_expr` は `collect_ambient_in_expr` と同じ AST 分岐を持ち、
エラー emit の代わりに `found.insert(ns.clone())` を呼ぶ。

---

### Phase 3: `lsp/inlay_hints.rs` — `collect_fn_effect_hints` 追加 + handle_inlay_hints 更新

`collect_pipeline_type_hints` の直後に追加。

```rust
pub(crate) fn collect_fn_effect_hints(source: &str) -> Vec<InlayHint> {
    // 1. Parse source (Err → return vec![])
    // 2. For each FnDef: collect_used_namespaces(&fd.body)
    // 3. If non-empty: emit InlayHint { label: " /* !IO !Snowflake */", ... }
    //    Position: line = fd.span.line - 1, character = source.lines().nth(line_idx).len()
}
```

`handle_inlay_hints` に追加:
```rust
hints.extend(collect_fn_effect_hints(&doc.source)); // v56.4.0
```

---

### Phase 4: `driver.rs` — `infer_fn_effects` 追加

`collect_inference_annotations` の直後に追加。

```rust
pub fn infer_fn_effects(src: &str) -> Vec<(String, Vec<String>)> {
    // Parse source; for each FnDef: collect_used_namespaces(&fd.body)
    // Returns [(fn_name, [ns1, ns2, ...]), ...]
    // Does NOT call checker — parse-only
}
```

---

### Phase 5: `driver.rs` — v56400_tests 追加 + v56300_tests 更新

1. `v56300_tests::cargo_toml_version_is_56_3_0` を削除
2. `v56400_tests` を `v56300_tests` の直前に挿入:
   - `cargo_toml_version_is_56_4_0`
   - `effect_inference_inlay_hint`（`collect_fn_effect_hints` テスト）
   - `effect_inference_check_output`（`infer_fn_effects` テスト）

---

### Phase 6: ポスト処理

- `CHANGELOG.md` に v56.4.0 エントリを追加
- `versions/current.md` を v56.4.0 / 3235 tests に更新
- `roadmap-v56.1-v57.0.md` の v56.4.0 実績を COMPLETE に更新
- `roadmap-v55.1-v60.0.md` の v56.4.0 実績を COMPLETE に更新

---

## リスク管理

| リスク | 対策 |
|--------|------|
| `collect_ns_in_expr` の分岐漏れ | `collect_ambient_in_expr` と同じ分岐を持つよう spec.md の完全コードをそのままコピー |
| `fd.span.line` が 0-based/1-based で混乱 | spec に明示: `fd.span.line` は 1-based → `saturating_sub(1)` で 0-based に変換 |
| `IO.println("hello")` が Favnir でパース失敗 | `Expr::Apply(FieldAccess(Ident("IO"), "println"), [Lit("hello")])` — パース可能（既存 W008 テストで確認済み） |
| `pub(crate) fn collect_fn_effect_hints` が driver.rs から使えない | `pub(crate)` は同一 crate 内で参照可能 — OK |
| `collect_fn_effect_hints` を `handle_inlay_hints` に追加し既存テストが壊れる | 既存テスト（`v464000_tests`）は bind/stage hints をテストしており、effect hints は追加のみ — 問題なし |
