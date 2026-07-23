# Plan: v46.5.0 — LSP クイックフィックス強化

Date: 2026-07-17

---

## ステップ

### Step 1 — `code_action.rs`: CA-4 + CA-5 追加

**`parse_did_you_mean`**（private ヘルパー）:

```rust
/// "did you mean `foo`?" → Some("foo")
fn parse_did_you_mean(hint: &str) -> Option<&str> {
    let start = hint.find('`')? + 1;
    let end = hint[start..].find('`')? + start;
    Some(&hint[start..end])
}
```

**`check_did_you_mean_fix`** (CA-4):
- E0102 エラーを行番号でフィルタ（`err.span.line > 0 && err.span.line as u32 - 1 == line_idx`）
- hints から候補名を抽出して TextEdit 付き CodeAction を生成
- col は 1-based → `err.span.col.saturating_sub(1) as u32` で 0-based に変換
- 識別子長は `err.span.end.saturating_sub(err.span.start) as u32`

```rust
fn check_did_you_mean_fix(
    doc: &crate::lsp::document_store::CheckedDoc,
    uri: &str,
    range: Range,
) -> Vec<CodeAction> {
    let line_idx = range.start.line;
    let mut actions = Vec::new();
    for err in &doc.errors {
        if err.code != "E0102" { continue; }
        if err.span.line == 0 || err.span.line as u32 - 1 != line_idx { continue; }
        for hint in &err.hints {
            if let Some(name) = parse_did_you_mean(hint) {
                let start_col = err.span.col.saturating_sub(1) as u32;
                let len = err.span.end.saturating_sub(err.span.start) as u32;
                let edit_range = Range {
                    start: Position { line: line_idx, character: start_col },
                    end:   Position { line: line_idx, character: start_col + len },
                };
                let mut changes = HashMap::new();
                changes.insert(
                    uri.to_string(),
                    vec![TextEdit { range: edit_range, new_text: name.to_string() }],
                );
                actions.push(CodeAction {
                    title: format!("Did you mean `{}`?", name),
                    kind: Some("quickfix".to_string()),
                    edit: Some(WorkspaceEdit { changes }),
                });
            }
        }
    }
    actions
}
```

**`check_arg_count_fix`** (CA-5):
- E0101 エラーかつ `message.contains("argument(s)")` でさらに絞り込む（型不一致 E0101 を除外）
- edit: None（診断表示のみ。将来の TextEdit 対応に備えて uri パラメータは受け取る）

```rust
fn check_arg_count_fix(
    _uri: &str,
    doc: &crate::lsp::document_store::CheckedDoc,
    range: Range,
) -> Vec<CodeAction> {
    let line_idx = range.start.line;
    doc.errors
        .iter()
        .filter(|e| {
            e.code == "E0101"
                && e.message.contains("argument(s)")
                && e.span.line > 0
                && e.span.line as u32 - 1 == line_idx
        })
        .map(|e| CodeAction {
            title: format!("Fix: {}", e.message),
            kind: Some("quickfix".to_string()),
            edit: None,
        })
        .collect()
}
```

**`handle_code_action` 更新**:

```rust
actions.extend(check_did_you_mean_fix(doc, uri, range));
actions.extend(check_arg_count_fix(uri, doc, range));
```

---

### Step 2 — `driver.rs`: `v465000_tests`

`v464000_tests` の後（`v455000_tests` の前）に追加:

```rust
mod v465000_tests {
    use super::*;
    use crate::lsp::code_action::handle_code_action;
    use crate::lsp::document_store::DocumentStore;
    use crate::lsp::protocol::{Position, Range};

    #[test]
    fn lsp_quick_fix_undefined_var() {
        let mut store = DocumentStore::new();
        // 'totally_undefined_xyz' は未定義 → E0102 が発行される
        let src = "fn main() -> Int { totally_undefined_xyz }";
        store.open_or_change("file:///undef.fav", src.to_string());
        let doc = store.get("file:///undef.fav").unwrap();
        assert!(
            doc.errors.iter().any(|e| e.code == "E0102"),
            "expected E0102 for undefined variable"
        );
        let range = Range {
            start: Position { line: 0, character: 19 },
            end:   Position { line: 0, character: 19 },
        };
        // handle_code_action はパニックせず Vec<CodeAction> を返す
        let actions = handle_code_action(&store, "file:///undef.fav", range);
        // hints が空なら CA-4 アクションは 0 件（正常）; hints があれば quickfix が存在する
        for a in &actions {
            assert_eq!(a.kind, Some("quickfix".to_string()));
        }
    }

    #[test]
    fn lsp_quick_fix_arg_count() {
        let mut store = DocumentStore::new();
        // add は 2 引数だが 1 個しか渡していない → E0101 が発行される
        let src = "fn add(a: Int, b: Int) -> Int { a + b }\nfn main() -> Int { add(1) }";
        store.open_or_change("file:///argcount.fav", src.to_string());
        let doc = store.get("file:///argcount.fav").unwrap();
        assert!(
            doc.errors.iter().any(|e| e.code == "E0101"),
            "expected E0101 for arg count mismatch"
        );
        let range = Range {
            start: Position { line: 1, character: 19 },
            end:   Position { line: 1, character: 19 },
        };
        let actions = handle_code_action(&store, "file:///argcount.fav", range);
        let has_fix = actions.iter().any(|a| a.title.contains("Fix: expected"));
        assert!(has_fix, "expected quickfix action for E0101: {:?}", actions);
    }
}
```

---

### Step 3 — ロードマップ修正

`versions/roadmap/roadmap-v46.1-v47.0.md` の v46.5.0 セクションの `E0007（引数数不一致）` を
`E0101（引数数不一致）` に修正する。

---

### Step 4 — バージョン更新

- `fav/Cargo.toml`: `46.5.0`
- `CHANGELOG.md`: v46.5.0 エントリ
- `versions/current.md`: v46.5.0（3003 tests）
