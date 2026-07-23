# Plan: v46.4.0 — LSP inlay hints 強化

Date: 2026-07-17

---

## 実装手順

### Step 1 — `checker.rs`: `Stmt::Bind` に `remember_type` 追加

**ファイル**: `fav/src/middle/checker.rs`

`Stmt::Bind` ハンドラの `if let Pattern::Bind(name, span) = &b.pattern {` ブロック（line ≈ 3969）に
`remember_type` 呼び出しを追加する。

変更前（line ≈ 3969-3977）:
```rust
if let Pattern::Bind(name, span) = &b.pattern {
    if effective_ty == Type::Unknown {
        self.type_warning(
            "W001",
            format!("type of `{}` could not be resolved (Unknown)", name),
            span,
        );
    }
}
```

変更後:
```rust
if let Pattern::Bind(name, span) = &b.pattern {
    // v46.4.0: bind 変数の型を type_at に記録（LSP inlay hints 用）
    self.remember_type(span, &effective_ty);
    if effective_ty == Type::Unknown {
        self.type_warning(
            "W001",
            format!("type of `{}` could not be resolved (Unknown)", name),
            span,
        );
    }
}
```

---

### Step 2 — `inlay_hints.rs`: `collect_stage_hints` 追加 + `handle_inlay_hints` 更新

**ファイル**: `fav/src/lsp/inlay_hints.rs`

`collect_bind_hints` の後ろに `collect_stage_hints` と `find_stage_prefix` を追加する。
`handle_inlay_hints` を更新して両方の結果を結合して返す。

変更後の `handle_inlay_hints`:
```rust
pub fn handle_inlay_hints(store: &DocumentStore, uri: &str) -> Vec<InlayHint> {
    let doc = match store.get(uri) {
        Some(d) => d,
        None => return vec![],
    };
    let mut hints = collect_bind_hints(&doc.source, &doc.type_at);
    hints.extend(collect_stage_hints(&doc.source, &doc.type_at));
    hints
}
```

追加する `collect_stage_hints`:
```rust
pub(crate) fn collect_stage_hints(
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
            let prefix_len = line.len() - rest.len();
            let name_start = byte_offset + prefix_len;
            let name_end_offset = name_start + name_end;
            if let Some(ty) = find_type_at(type_at, name_start, name_end_offset) {
                let col = (prefix_len + name_end) as u32;
                hints.push(InlayHint {
                    position: Position {
                        line: line_idx as u32,
                        character: col,
                    },
                    label: format!(": {}", ty.display()),
                    kind: 1,
                });
            }
        }
        byte_offset += line.len() + 1;
    }
    hints
}

fn find_stage_prefix(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    trimmed.strip_prefix("stage ").map(|r| r.trim_start())
}
```

---

### Step 3 — `driver.rs`: `v464000_tests` 追加

**ファイル**: `fav/src/driver.rs`

`v463000_tests` の直後に追加:

```rust
// -- v464000_tests (v46.4.0) -- LSP inlay hints 強化 --
#[cfg(test)]
mod v464000_tests {
    #[test]
    fn lsp_inlay_hints_type_annotation() {
        use super::LspServer;
        // bind x <- 42 を含むドキュメントを didOpen し inlayHint を要求
        // §1 の checker 修正で type_at に型が記録され ": Int" ヒントが返ることを確認
    }

    #[test]
    fn lsp_inlay_hints_pipeline() {
        use crate::lsp::inlay_hints::collect_stage_hints;
        use crate::frontend::lexer::Span;
        use crate::middle::checker::Type;
        use std::collections::HashMap;
        // collect_stage_hints が "stage " プレフィックスを検出してヒントを返すことを確認
    }
}
```

---

### Step 4 — バージョン・ドキュメント更新

1. `fav/Cargo.toml`: `version = "46.4.0"`
2. `CHANGELOG.md`: v46.4.0 エントリ追加
3. `versions/current.md`: v46.4.0（3001 tests）に更新

---

## 注意事項

- `check_pattern_bindings` の `Pattern::Bind` は変更しない（match arm 等の内部パターンにも適用されてしまう）
- `Stmt::Bind` ハンドラのみ修正してトップレベル bind 文の変数のみを記録する
- `find_type_at` は既存の実装をそのまま使う（`inlay_hints.rs` の `pub(crate)` でない関数）
  → `find_type_at` は `pub(crate)` 化不要（`collect_stage_hints` は同じファイル内）
