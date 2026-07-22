# Plan: v50.4.0 — LSP インレイヒント Phase 1（変数・関数戻り型）

Date: 2026-07-19

---

## 実装方針

### Step 1: 現状確認

```bash
# inlay_hints.rs の現行実装を確認
grep -n "pub fn\|pub(crate) fn" fav/src/lsp/inlay_hints.rs

# handle_inlay_hints の wiring を確認
grep -n "inlay" fav/src/lsp/mod.rs

# collect_fn_return_hints が未実装であることを確認
grep -n "fn_return" fav/src/lsp/inlay_hints.rs
```

**確認済み事項（事前調査）:**
- `inlay_hints.rs`: `collect_bind_hints` / `collect_stage_hints` 実装済み
- `mod.rs`: `textDocument/inlayHint` ハンドラ・`"inlayHintProvider": true` 実装済み
- `collect_fn_return_hints` は未存在 → v50.4.0 で追加

### Step 2: `lsp/inlay_hints.rs` — `collect_fn_return_hints` 追加

`collect_stage_hints` の直後（現在の末尾付近）に追加する。

```rust
/// v50.4.0: Collect inlay hints for fn return types (no explicit `->` annotation).
pub(crate) fn collect_fn_return_hints(
    source: &str,
    type_at: &HashMap<Span, Type>,
) -> Vec<InlayHint> {
    let mut hints = Vec::new();
    let mut byte_offset: usize = 0;
    for (line_idx, line) in source.lines().enumerate() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("fn ") || line.contains("->") {
            byte_offset += line.len() + 1;
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("fn ") {
            let name_end = rest
                .find(|c: char| !c.is_alphanumeric() && c != '_')
                .unwrap_or(rest.len());
            if name_end == 0 {
                byte_offset += line.len() + 1;
                continue;
            }
            let indent_len = line.len() - trimmed.len();
            let name_start = byte_offset + indent_len + 3; // 3 = "fn ".len()
            let name_end_offset = name_start + name_end;
            if let Some(paren_pos) = line.rfind(')') {
                if let Some(ty) = find_type_at(type_at, name_start, name_end_offset) {
                    hints.push(InlayHint {
                        position: Position {
                            line: line_idx as u32,
                            character: paren_pos as u32 + 1,
                        },
                        label: format!(" -> {}", ty.display()),
                        kind: 1,
                    });
                }
            }
        }
        byte_offset += line.len() + 1;
    }
    hints
}
```

### Step 3: `handle_inlay_hints` に `collect_fn_return_hints` を組み込み

`handle_inlay_hints` 関数を以下に更新する:

```rust
pub fn handle_inlay_hints(store: &DocumentStore, uri: &str) -> Vec<InlayHint> {
    let doc = match store.get(uri) {
        Some(d) => d,
        None => return vec![],
    };
    let mut hints = collect_bind_hints(&doc.source, &doc.type_at);
    hints.extend(collect_stage_hints(&doc.source, &doc.type_at));
    // v50.4.0: fn return type hints for definitions without explicit `->`
    hints.extend(collect_fn_return_hints(&doc.source, &doc.type_at));
    hints
}
```

### Step 4: `v504000_tests` モジュール追加

`driver.rs` の `v503000_tests` 直前に挿入する。

```rust
// -- v504000_tests (v50.4.0) -- LSP インレイヒント Phase 1 --
#[cfg(test)]
mod v504000_tests {
    #[test]
    fn cargo_toml_version_is_50_4_0() {
        let content = include_str!("../Cargo.toml");
        assert!(content.contains("version = \"50.4.0\""),
            "Cargo.toml version should be 50.4.0");
    }

    #[test]
    fn lsp_inlay_hint_let_binding() {
        use crate::frontend::lexer::Span;
        use crate::lsp::inlay_hints::collect_bind_hints;
        use crate::middle::checker::Type;
        use std::collections::HashMap;

        let source = "bind count <- 42";
        // "bind " = 5 bytes; "count" spans bytes 5..10
        let span = Span { file: "t".to_string(), start: 5, end: 10, line: 1, col: 6 };
        let mut type_at = HashMap::new();
        type_at.insert(span, Type::Int);

        let hints = collect_bind_hints(source, &type_at);
        assert_eq!(hints.len(), 1, "expected one hint for bind binding");
        assert!(hints[0].label.contains("Int"),
            "hint label should show inferred type, got: {}", hints[0].label);
    }

    #[test]
    fn lsp_inlay_hint_fn_return() {
        use crate::frontend::lexer::Span;
        use crate::lsp::inlay_hints::collect_fn_return_hints;
        use crate::middle::checker::Type;
        use std::collections::HashMap;

        let source = "fn double(x: Int) { x * 2 }";
        // "fn " = 3 bytes; "double" spans bytes 3..9
        let span = Span { file: "t".to_string(), start: 3, end: 9, line: 1, col: 4 };
        let mut type_at = HashMap::new();
        type_at.insert(span, Type::Int);

        let hints = collect_fn_return_hints(source, &type_at);
        assert_eq!(hints.len(), 1, "expected one hint for fn without return type");
        assert!(hints[0].label.contains("Int"),
            "hint should show inferred return type, got: {}", hints[0].label);
    }
}
```

### Step 5: バージョン更新・完了

1. `fav/Cargo.toml` version → `"50.4.0"`
2. `v503000_tests::cargo_toml_version_is_50_3_0` を削除
3. `cargo test` 通過確認（3099）
4. `cargo clippy -- -D warnings` クリーン確認
5. `CHANGELOG.md` に v50.4.0 エントリ追加
6. `versions/current.md` 更新
7. `versions/roadmap/roadmap-v50.1-v51.0.md` の v50.4.0 実績を記入

---

## 注意事項

- `find_type_at` は `pub(crate)` ではなく `fn`（モジュール内プライベート）のため
  `collect_fn_return_hints` は同一ファイル（`inlay_hints.rs`）内に置くこと。
- `line.contains("->")` は `fn foo() -> Int` を正しく除外するが、
  `->` をパラメータ型に含む関数（例: `fn f(cb: Fn() -> Int)`）も誤って除外する。
  v50.4.0 スコープでは許容する（既存の `collect_stage_hints` と同じ方針）。
- `line.rfind(')')` はパラメータ内の `)` より後ろを優先するため、
  `fn f(a: Option<(Int, Int)>)` のような入れ子括弧では誤った位置になる可能性がある。
  v50.4.0 スコープでは許容する。
- self-hosted ファイル（`compiler.fav` / `checker.fav`）への変更なし。
