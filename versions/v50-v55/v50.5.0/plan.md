# Plan: v50.5.0 — LSP インレイヒント Phase 2（パイプライン stage 型）

Date: 2026-07-19

---

## 実装方針

### Step 1: 現状確認

```bash
# collect_pipeline_type_hints が未実装であることを確認
grep -n "pipeline_type" fav/src/lsp/inlay_hints.rs

# collect_stage_hints の実装・find_stage_prefix を確認
grep -n "find_stage_prefix\|collect_stage_hints" fav/src/lsp/inlay_hints.rs

# Type::Arrow の存在を確認
grep -n "Arrow\|Trf" fav/src/middle/checker.rs | head -10
```

**確認済み事項（事前調査）:**
- `collect_stage_hints` / `find_stage_prefix` 実装済み
- `Type::Arrow(Box<Type>, Box<Type>)` → `display()` で `"In -> Out"`
- `Type::Trf(Box<Type>, Box<Type>)` → `display()` で `"Trf<In, Out>"`
- `collect_pipeline_type_hints` は未存在 → v50.5.0 で追加

### Step 2: `lsp/inlay_hints.rs` — `collect_pipeline_type_hints` 追加

`collect_fn_return_hints` の直後（`find_type_at` の直前）に追加する。
`find_stage_prefix` を再利用し、`Type::Arrow` / `Type::Trf` の stage のみヒントを生成。

```rust
/// v50.5.0: Collect inlay hints for pipeline stage IO types.
/// Only emits hints for stages whose type is `Type::Arrow` or `Type::Trf`
/// (i.e. has a meaningful In -> Out structure).
/// Shares the same text-scan approach and known limitations as `collect_stage_hints`.
pub(crate) fn collect_pipeline_type_hints(
    source: &str,
    type_at: &HashMap<Span, Type>,
) -> Vec<InlayHint> {
    let mut hints = Vec::new();
    let mut byte_offset: usize = 0;
    for (line_idx, line) in source.lines().enumerate() {
        if let Some(rest) = find_stage_prefix(line) {
            let name_end = rest
                .find(|c: char| !c.is_alphanumeric() && c != '_')
                .unwrap_or(rest.len());
            if name_end == 0 {
                byte_offset += line.len() + 1;
                continue;
            }
            let name = &rest[..name_end];
            if name == "_" {
                byte_offset += line.len() + 1;
                continue;
            }
            let prefix_len = line.len() - rest.len();
            let name_start = byte_offset + prefix_len;
            let name_end_offset = name_start + name_end;
            if let Some(ty) = find_type_at(type_at, name_start, name_end_offset) {
                let label = match ty {
                    Type::Arrow(_, _) | Type::Trf(_, _) => {
                        Some(format!(": {}", ty.display()))
                    }
                    _ => None,
                };
                if let Some(label) = label {
                    let col = (prefix_len + name_end) as u32;
                    hints.push(InlayHint {
                        position: Position {
                            line: line_idx as u32,
                            character: col,
                        },
                        label,
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

### Step 3: `handle_inlay_hints` に `collect_pipeline_type_hints` を追加

```rust
pub fn handle_inlay_hints(store: &DocumentStore, uri: &str) -> Vec<InlayHint> {
    let doc = match store.get(uri) {
        Some(d) => d,
        None => return vec![],
    };
    let mut hints = collect_bind_hints(&doc.source, &doc.type_at);
    hints.extend(collect_stage_hints(&doc.source, &doc.type_at));
    hints.extend(collect_fn_return_hints(&doc.source, &doc.type_at));
    // v50.5.0: pipeline stage IO type hints (Arrow / Trf types only)
    hints.extend(collect_pipeline_type_hints(&doc.source, &doc.type_at));
    hints
}
```

### Step 4: `v505000_tests` モジュール追加

`driver.rs` の `v504000_tests` 直前に挿入する。

```rust
// -- v505000_tests (v50.5.0) -- LSP インレイヒント Phase 2 --
#[cfg(test)]
mod v505000_tests {
    #[test]
    fn cargo_toml_version_is_50_5_0() {
        let content = include_str!("../Cargo.toml");
        assert!(content.contains("version = \"50.5.0\""),
            "Cargo.toml version should be 50.5.0");
    }

    #[test]
    fn lsp_inlay_hint_stage_type() {
        use crate::frontend::lexer::Span;
        use crate::lsp::inlay_hints::collect_stage_hints;
        use crate::middle::checker::Type;
        use std::collections::HashMap;

        // "  stage Parse\n": indent=2, "stage "=6 → "Parse" at bytes 8..13
        // Span.line is 1-indexed; find_type_at uses start/end only
        let source = "  stage Parse\n";
        let span = Span { file: "t".to_string(), start: 8, end: 13, line: 1, col: 9 };
        let mut type_at = HashMap::new();
        type_at.insert(span, Type::Arrow(
            Box::new(Type::Named("RawOrder".to_string(), vec![])),
            Box::new(Type::Named("Order".to_string(), vec![])),
        ));

        let hints = collect_stage_hints(source, &type_at);
        assert_eq!(hints.len(), 1, "expected one hint for stage");
        assert!(hints[0].label.contains("RawOrder"),
            "hint should show input type, got: {}", hints[0].label);
        assert!(hints[0].label.contains("Order"),
            "hint should show output type, got: {}", hints[0].label);
    }

    #[test]
    fn lsp_inlay_hint_pipeline_type() {
        use crate::frontend::lexer::Span;
        use crate::lsp::inlay_hints::collect_pipeline_type_hints;
        use crate::middle::checker::Type;
        use std::collections::HashMap;

        // "  stage Parse\n  stage Validate\n"
        // Line 1: "  stage Parse"   (13 bytes) → "Parse"   at bytes 8..13
        // Line 2: "  stage Validate" (16 bytes) → offset 14, "Validate" at bytes 22..30
        // Span.line is 1-indexed; find_type_at uses start/end only
        let source = "  stage Parse\n  stage Validate\n";
        let mut type_at = HashMap::new();
        type_at.insert(
            Span { file: "t".to_string(), start: 8, end: 13, line: 1, col: 9 },
            Type::Arrow(
                Box::new(Type::Named("RawOrder".to_string(), vec![])),
                Box::new(Type::Named("Order".to_string(), vec![])),
            ),
        );
        type_at.insert(
            Span { file: "t".to_string(), start: 22, end: 30, line: 2, col: 9 },
            Type::Arrow(
                Box::new(Type::Named("Order".to_string(), vec![])),
                Box::new(Type::Named("ValidOrder".to_string(), vec![])),
            ),
        );

        let hints = collect_pipeline_type_hints(source, &type_at);
        assert_eq!(hints.len(), 2, "expected hints for both stages");
        let labels: Vec<_> = hints.iter().map(|h| &h.label).collect();
        assert!(labels.iter().any(|l| l.contains("RawOrder")),
            "first stage hint should show RawOrder, got: {:?}", labels);
        assert!(labels.iter().any(|l| l.contains("ValidOrder")),
            "second stage hint should show ValidOrder, got: {:?}", labels);
    }
}
```

### Step 5: バージョン更新・完了

1. `fav/Cargo.toml` version → `"50.5.0"`
2. `v504000_tests::cargo_toml_version_is_50_4_0` を削除
3. `cargo test` 通過確認（3101）
4. `cargo clippy -- -D warnings` クリーン確認
5. `CHANGELOG.md` に v50.5.0 エントリ追加
6. `versions/current.md` 更新
7. `versions/roadmap/roadmap-v50.1-v51.0.md` の v50.5.0 実績を記入

---

## 注意事項

- `collect_pipeline_type_hints` は `collect_stage_hints` と同じ `stage ` テキストスキャンを使う。
  `type_at` に `Type::Arrow` / `Type::Trf` が記録されている stage に対してのみヒントを生成するため、
  同じ stage に `collect_stage_hints` と `collect_pipeline_type_hints` が重複してヒントを出す
  可能性がある。重複は現在の `handle_inlay_hints` の設計上許容する（LSP クライアントがフィルタリング）。
- `find_stage_prefix` は `inlay_hints.rs` 内のプライベート関数のため、`collect_pipeline_type_hints`
  は同一ファイル内に置くこと。
- self-hosted ファイル（`compiler.fav` / `checker.fav`）への変更なし。
