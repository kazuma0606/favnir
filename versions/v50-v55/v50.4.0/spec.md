# Spec: v50.4.0 — LSP インレイヒント Phase 1（変数・関数戻り型）

Date: 2026-07-19
Status: Draft

---

## 概要

`fn` 定義の戻り型注釈が省略されている場合に、推論された戻り型を ` -> Type` 形式で
インレイヒントとして表示する `collect_fn_return_hints` 関数を `lsp/inlay_hints.rs` に追加する。

> **テスト件数の注記**: ロードマップ v50.4.0 は機能テスト 2 件（`lsp_inlay_hint_let_binding`・
> `lsp_inlay_hint_fn_return`）を完了条件として記載している。
> 本バージョンではこれに加えてバージョン確認テスト 1 件（`cargo_toml_version_is_50_4_0`）を
> 追加するため、`v504000_tests` モジュールは合計 3 件となる。テスト総数は 3099。

---

## 背景

### 現状（v46.4.0 時点で実装済み）

| 機能 | 実装状況 | 備考 |
|---|---|---|
| `textDocument/inlayHint` ハンドラ | **実装済み** | `mod.rs` 175 行 |
| `"inlayHintProvider": true` | **実装済み** | `mod.rs` capabilities |
| `bind` 束縛の型ヒント | **実装済み** | `collect_bind_hints` |
| `stage` 名の型ヒント | **実装済み** | `collect_stage_hints`（v46.4.0） |
| `fn` 戻り型ヒント | **未実装** | 本バージョンで追加 |

v50.4.0 で追加するのは `collect_fn_return_hints` のみ。LSP ハンドラ・capabilities は変更不要。

---

## 仕様

### 変更 1: `lsp/inlay_hints.rs` — `collect_fn_return_hints` 追加

`fn <name>(...)` 行に `->` が含まれない場合（明示的な戻り型なし）に
推論型を ` -> Type` 形式でインレイヒント表示する。

> **制約**: `find_type_at` は `inlay_hints.rs` 内のプライベート関数（`fn find_type_at`）のため、
> `collect_fn_return_hints` は必ず同一ファイル（`inlay_hints.rs`）内に追加すること。

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
        // Only process `fn` definitions without explicit return type
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
            // Place hint at closing `)` of parameter list
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

### 変更 2: `handle_inlay_hints` に `collect_fn_return_hints` を追加

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

### テスト仕様

`v504000_tests` モジュールを `driver.rs` の `v503000_tests` 直前に追加（3 件）。

テスト総数: 3097（ベース）− 1（`cargo_toml_version_is_50_3_0` 削除）+ 3（v504000_tests 追加）= **3099**。

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

        // "bind count <- 42": "bind " = 5 bytes, "count" at bytes 5..10
        // Span.line is 1-indexed in Favnir lexer; find_type_at uses start/end only
        let source = "bind count <- 42";
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

        // "fn double(x: Int) { x * 2 }": "fn " = 3 bytes, "double" at bytes 3..9
        // Span.line is 1-indexed in Favnir lexer; find_type_at uses start/end only
        let source = "fn double(x: Int) { x * 2 }";
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

---

## 完了条件

- `cargo test` 3099 passed, 0 failed
- `lsp/inlay_hints.rs`: `collect_fn_return_hints` 追加、`handle_inlay_hints` に組み込み
- `collect_fn_return_hints` は `pub(crate)` で `driver.rs` テストからアクセス可能
- `cargo clippy -- -D warnings` クリーン
- `CHANGELOG.md` に v50.4.0 エントリ追加
- `versions/current.md` を v50.4.0 に更新
- `versions/roadmap/roadmap-v50.1-v51.0.md` の v50.4.0 実績を記入
