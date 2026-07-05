# v31.5.0 実装計画 — LSP Inlay Hints

## 前提

- `fav/Cargo.toml` version = `31.4.0`
- `cargo test` — 2437 passed（0 failures）
- v31.4.0 が COMPLETE であること

---

## 実装ステップ

### Step 1: バージョンバンプ

**`fav/Cargo.toml`**
- `version = "31.4.0"` → `version = "31.5.0"`

### Step 2: driver.rs スタブ化

**`fav/src/driver.rs`** — `v314000_tests::cargo_toml_version_is_31_4_0` をスタブ化:

```rust
fn cargo_toml_version_is_31_4_0() {
    // Stubbed: version bumped to 31.5.0 in v31.5.0.
}
```

### Step 3: fav/src/lsp/inlay_hints.rs 新規作成

```rust
use serde::Serialize;
use std::collections::HashMap;
use crate::frontend::lexer::Span;
use crate::lsp::document_store::DocumentStore;
use crate::middle::checker::Type;

#[derive(Debug, Serialize)]
pub struct InlayHint {
    pub position: crate::lsp::protocol::Position,
    pub label: String,
    pub kind: u32, // 1 = Type
}

pub fn handle_inlay_hints(store: &DocumentStore, uri: &str) -> Vec<InlayHint> {
    let doc = match store.get(uri) {
        Some(d) => d,
        None => return vec![],
    };
    collect_bind_hints(&doc.source, &doc.type_at)
}

pub(crate) fn collect_bind_hints(
    source: &str,
    type_at: &HashMap<Span, Type>,
) -> Vec<InlayHint> {
    let mut hints = Vec::new();
    let mut byte_offset: usize = 0;
    for (line_idx, line) in source.lines().enumerate() {
        if let Some(rest) = find_bind_prefix(line) {
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
            let name_end_offset = name_start + name.len();
            if let Some(ty) = find_type_at(type_at, name_start, name_end_offset) {
                let col = (prefix_len + name.len()) as u32;
                hints.push(InlayHint {
                    position: crate::lsp::protocol::Position {
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

fn find_bind_prefix(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    trimmed.strip_prefix("bind ").map(|r| r.trim_start())
}

fn find_type_at<'a>(
    type_at: &'a HashMap<Span, Type>,
    name_start: usize,
    name_end: usize,
) -> Option<&'a Type> {
    // bind 変数名の Span に重なる型エントリを探す
    type_at.iter()
        .filter(|(span, _)| span.start <= name_end && span.end >= name_start)
        .min_by_key(|(span, _)| span.end.saturating_sub(span.start))
        .map(|(_, ty)| ty)
}
```

### Step 4: lsp/mod.rs に pub mod と initialize + ハンドラ追加

**`fav/src/lsp/mod.rs`** への変更（3 箇所）:

**①** ファイル先頭付近のモジュール宣言に追加:
```rust
pub mod inlay_hints;
```

**②** `initialize` 応答の capabilities に追加:
```rust
"inlayHintProvider": true,
```

**③** `"textDocument/references"` アームの直後に追加:
```rust
"textDocument/inlayHint" => {
    let result = request.params
        .get("textDocument")
        .and_then(|td| td.get("uri"))
        .and_then(|u| u.as_str())
        .map(|uri| inlay_hints::handle_inlay_hints(&self.store, uri))
        .and_then(|hints| serde_json::to_value(hints).ok())
        .unwrap_or_else(|| serde_json::json!([]));
    self.write_response(request.id.unwrap_or(serde_json::Value::Null), result)?;
    Ok(false)
}
```

### Step 5: VS Code package.json 更新

**`fav/editors/favnir-vscode/package.json`** の `"contributes"` オブジェクトに以下を追加:

```json
"capabilities": {
    "inlayHints": {
        "resolveProvider": false
    }
}
```

> package.json の既存の `"contributes"` キーを確認した上で追記すること。

### Step 6: v315000_tests 追加

`v314000_tests` の直前に追加:

```rust
// ── v31.5.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v315000_tests {
    use super::*;
    #[test]
    fn cargo_toml_version_is_31_5_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("version = \"31.5.0\""), "Cargo.toml must contain version = \"31.5.0\"");
    }
    #[test]
    fn benchmark_v31_5_0_exists() {
        let src = include_str!("../../benchmarks/v31.5.0.json");
        assert!(src.contains("31.5.0"), "benchmarks/v31.5.0.json must contain '31.5.0'");
    }
    #[test]
    fn lsp_inlay_hints_bind_variable() {
        use crate::lsp::inlay_hints::collect_bind_hints;
        use crate::frontend::lexer::Span;
        use crate::middle::checker::Type;
        use std::collections::HashMap;
        // "bind n <- List.length([1, 2, 3])" において
        // "n" は offset 5〜6（"bind " = 5 バイト）
        let source = "bind n <- List.length([1, 2, 3])\n";
        let mut type_at = HashMap::new();
        type_at.insert(Span::new("test", 5, 6, 1, 6), Type::Int);
        let hints = collect_bind_hints(source, &type_at);
        assert!(!hints.is_empty(), "should generate a hint for 'bind n'");
        assert!(hints[0].label.starts_with(": "),
            "hint label must start with ': ', got: {}", hints[0].label);
    }
}
```

### Step 7: CHANGELOG.md 追記

```markdown
## [v31.5.0] — 2026-07-02

### Added
- `lsp/inlay_hints.rs` — 新規作成: `handle_inlay_hints()` / `collect_bind_hints()` 実装
- LSP `initialize` 応答に `"inlayHintProvider": true` を追加
- LSP `textDocument/inlayHint` ハンドラを追加
- `editors/favnir-vscode/package.json` に `inlayHints` capability を追記
- `benchmarks/v31.5.0.json` 追加

### Changed
- `Cargo.toml` version: `31.4.0` → `31.5.0`
```

### Step 8: benchmarks/v31.5.0.json 作成

```json
{
  "version": "31.5.0",
  "date": "2026-07-02",
  "milestone": "Real-World Readiness",
  "tests_passed": 2440,
  "tests_failed": 0,
  "notes": "LSP inlayHint: bind variable type inline display"
}
```

> `tests_passed` は `cargo test` 実行後に実測値で更新する（+3 件 = 2440 想定）。
> **T13 で必ず実測値に書き換えること。** 上記の 2440 は暫定値。

### Step 9: versions/current.md 更新

- 「最新安定版」欄を v31.5.0 に更新
- 「次に切る版」を `v31.6.0 — TBD` に更新

---

## ファイル変更一覧

| ファイル | 種別 | 変更内容 |
|---|---|---|
| `fav/Cargo.toml` | 更新 | version `31.4.0` → `31.5.0` |
| `fav/src/driver.rs` | 更新 | v314000 スタブ化 + v315000_tests（3件）追加 |
| `fav/src/lsp/inlay_hints.rs` | 新規 | handle_inlay_hints / collect_bind_hints 実装 |
| `fav/src/lsp/mod.rs` | 更新 | pub mod + inlayHintProvider + textDocument/inlayHint ハンドラ |
| `fav/editors/favnir-vscode/package.json` | 更新 | inlayHints capability 追加 |
| `CHANGELOG.md` | 更新 | [v31.5.0] セクション追加 |
| `benchmarks/v31.5.0.json` | 新規 | ベンチマーク結果（T13 で tests_passed を実測値に更新すること）|
| `versions/current.md` | 更新 | v31.5.0 に更新 |

---

## 完了判定

- `cargo test v315000` — 3/3 PASS
- `cargo test` — 全件 PASS（0 failures）
