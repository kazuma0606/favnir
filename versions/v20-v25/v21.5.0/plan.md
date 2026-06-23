# v21.5.0 実装計画 — LSP コードアクション強化

## タスク順序

```
T0: document_store.rs program フィールド追加（T2/T3/T4 が依存）
T1: protocol.rs 拡張（TextEdit / WorkspaceEdit / CodeAction）
T2: references.rs 新規作成（collect_symbol_occurrences + handle_references）
T3: rename.rs 新規作成（handle_rename）
T4: code_action.rs 新規作成（CA-1〜CA-3）
T5: mod.rs 更新（3 ハンドラ + capabilities）
T6: Cargo.toml バージョン更新
T7: driver.rs v215000_tests 追加
T8: CHANGELOG + lsp.mdx
```

T0 → T1 → T2, T3, T4 は並列可（T0 の program フィールド + T1 の型定義が必要）
T5 は T2, T3, T4 完了後

---

## T0: `fav/src/lsp/document_store.rs` — program フィールド追加

**事前確認**: `grep -n "struct CheckedDoc\|pub program\|pub source" fav/src/lsp/document_store.rs | head -15`

現在の `CheckedDoc` には `program` フィールドがない。references / rename / code_action が AST を必要とするため追加する。

```rust
// CheckedDoc に追加:
pub program: Option<Program>,
```

`open_or_change` の `Parser::parse_str` 成功時に `program` を格納:
```rust
let program = Parser::parse_str(&source, uri).ok();
// ... existing checker logic ...
CheckedDoc {
    source,
    program,  // 追加
    errors,
    // ... rest unchanged ...
}
```

注意: `Program` 型は `crate::frontend::ast::Program`。`use crate::frontend::ast::Program;` を追加。

---

## T1: `fav/src/lsp/protocol.rs` — 新規型追加

既存の `Range` / `Location` / `Position` 型はそのまま使う。

追加する型:
```rust
#[derive(Debug, Clone, Serialize)]
pub struct TextEdit {
    pub range: Range,
    #[serde(rename = "newText")]
    pub new_text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceEdit {
    pub changes: std::collections::HashMap<String, Vec<TextEdit>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CodeAction {
    pub title: String,
    // LSP 仕様: kind は省略可能な文字列
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edit: Option<WorkspaceEdit>,
}
```

---

## T2: `fav/src/lsp/references.rs` — 新規作成

### `collect_symbol_occurrences`

```rust
pub fn collect_symbol_occurrences(program: &Program, name: &str) -> Vec<Span>
```

走査対象:
- `program.items` の各 `Item` でシンボル宣言スパンを収集
- `FnDef(fd)` if `fd.name == name` → `fd.span`
- `TrfDef(td)` if `td.name == name` → `td.span`
- `TypeDef(td)` if `td.name == name` → `td.span`
- 式を再帰走査して `Expr::Ident(n, span)` where `n == name` を収集
- `TypeExpr::Named(n, _, span)` where `n == name` を収集

ヘルパー:
```rust
fn collect_in_item(item: &Item, name: &str, spans: &mut Vec<Span>)
fn collect_in_expr(expr: &Expr, name: &str, spans: &mut Vec<Span>)
fn collect_in_block(block: &Block, name: &str, spans: &mut Vec<Span>)
fn collect_in_type_expr(ty: &TypeExpr, name: &str, spans: &mut Vec<Span>)
```

### `handle_references`

```rust
pub fn handle_references(
    store: &DocumentStore,
    uri: &str,
    pos: Position,
) -> Vec<Location>
```

実装:
1. `store.get(uri)` → `doc`
2. `position_to_char_offset(&doc.source, pos)` → `offset`
3. カーソル位置のシンボル名を `word_at_offset(&doc.source, offset)` で取得
4. `doc.program.as_ref()` が `Some(program)` なら `collect_symbol_occurrences` を呼ぶ
5. 各 Span を `span_to_range(&doc.source, span)` で Range に変換
6. `Location { uri: uri.to_string(), range }` のリストを返す

### span → Range 変換

`Span` は `{ line: u32, col: u32, start: usize, end: usize }` のバイトオフセット情報を持つ。

```rust
pub fn span_to_range(span: &Span) -> Range
```

- `start_line = span.line - 1`（0-based）
- `start_char = span.col - 1`（0-based）
- `end_char = start_char + (span.end - span.start)` — ASCII 識別子はバイト長 = 文字長のため正確
- `end_line = start_line`（同一行を仮定、識別子は改行をまたがない）

`src` 引数は不要（`span.start/end` でバイト長が直接計算できるため）。
`Span { start: usize, end: usize }` のバイトオフセットが存在するため、`name.len()` による近似は不要。

### カーソル位置のシンボル名取得

```rust
pub fn word_at_offset(src: &str, offset: usize) -> Option<String>
```

- offset を中心に前後のアルファベット / `_` をスキャン
- 空なら `None`

---

## T3: `fav/src/lsp/rename.rs` — 新規作成

```rust
pub fn handle_rename(
    store: &DocumentStore,
    uri: &str,
    pos: Position,
    new_name: &str,
) -> Option<WorkspaceEdit>
```

実装:
1. `word_at_offset` でカーソル下のシンボル名を取得 → `name`
2. `name` が空、または Favnir キーワード（`"fn"`, `"stage"`, `"type"`, `"bind"` 等）なら `None`
3. `collect_symbol_occurrences(program, &name)` → `spans`
4. `spans` が空なら `None`
5. 各 span を TextEdit に変換（`new_text: new_name.to_string()`）
6. `WorkspaceEdit { changes: { uri: edits } }` を返す

### Favnir キーワードリスト（ローカル定数）

```rust
const FAVNIR_KEYWORDS: &[&str] = &[
    "fn", "stage", "type", "bind", "chain", "seq", "pub",
    "public", "match", "if", "else", "true", "false",
    "use", "import", "test", "forall", "par", "abstract",
];
```

**注**: 実装では v9.12.0〜v9.13.0 で導入された `"interface"`, `"impl"`, `"cap"`, `"bench"`, `"namespace"` を追加し計 25 語。

---

## T4: `fav/src/lsp/code_action.rs` — 新規作成

```rust
pub fn handle_code_action(
    store: &DocumentStore,
    uri: &str,
    range: Range,
) -> Vec<CodeAction>
```

実装:
1. `store.get(uri)` → `doc`
2. `range.start.line` の行テキストを取得（ソースを改行分割）
3. 3 つのアクションを順にチェックし、該当するものを `actions` に追加

### CA-1: addMissingImport

```rust
fn check_add_missing_import(
    doc: &CheckedDoc,
    uri: &str,
    line_text: &str,
) -> Option<CodeAction>
```

- `line_text` から英大小文字 + `_` の連続 + `.` パターンを文字列スキャンで抽出 → `ns`
- `ns` が `completion::KNOWN_RUNES` に含まれるか確認（`use crate::lsp::completion::KNOWN_RUNES;`）
- `doc.program.as_ref()` の `uses: Vec<Vec<String>>` を検索:
  `!program.uses.iter().any(|path| path.first().map(|s| s == ns).unwrap_or(false))`
- WorkspaceEdit: `Range { start: Position{0,0}, end: Position{0,0} }` に `"use <ns>\n"` を挿入
- `title: format!("Add missing import: use {}", ns)`、`kind: Some("quickfix".to_string())`

注意: `program.uses` は `Vec<Vec<String>>`（`UseDecl` 型は存在しない）。

### CA-2: convertToFstring

```rust
fn check_convert_to_fstring(line_text: &str) -> Option<CodeAction>
```

- `line_text.contains("String.concat(")` なら
- `CodeAction { title: "Convert to f-string".into(), kind: Some("refactor.rewrite".into()), edit: None }`

### CA-3: inlineBinding

```rust
fn check_inline_binding(line_text: &str) -> Option<CodeAction>
```

- `line_text.trim_start().starts_with("bind ")` なら
- `bind` の後の名前を抽出（`bind <name> <-`）
- `CodeAction { title: format!("Inline binding `{}`", name), kind: Some("refactor.inline".into()), edit: None }`

---

## T5: `fav/src/lsp/mod.rs` — 統合

### capabilities に追加

```rust
"codeActionProvider": true,
"renameProvider": true,
"referencesProvider": true,
```

### 新規ハンドラ

```rust
"textDocument/codeAction" => {
    let result = extract_code_action_params(&request.params)
        .map(|(uri, range)| handle_code_action(&self.store, &uri, range))
        .and_then(|actions| serde_json::to_value(actions).ok())
        .unwrap_or_else(|| serde_json::json!([]));
    self.write_response(...)?;
    Ok(false)
}
"textDocument/rename" => {
    let result = extract_rename_params(&request.params)
        .and_then(|(uri, pos, new_name)| handle_rename(&self.store, &uri, pos, &new_name))
        .and_then(|edit| serde_json::to_value(edit).ok())
        .unwrap_or(serde_json::Value::Null);
    self.write_response(...)?;
    Ok(false)
}
"textDocument/references" => {
    let result = extract_hover_target(&request.params)
        .map(|(uri, pos)| handle_references(&self.store, &uri, pos))
        .and_then(|locs| serde_json::to_value(locs).ok())
        .unwrap_or_else(|| serde_json::json!([]));
    self.write_response(...)?;
    Ok(false)
}
```

### params 抽出ヘルパー

```rust
fn extract_code_action_params(params: &serde_json::Value) -> Option<(String, Range)>
fn extract_rename_params(params: &serde_json::Value) -> Option<(String, Position, String)>
```

---

## T6: Cargo.toml バージョン更新

```
version = "21.4.0" → "21.5.0"
```

v214000_tests::version_is_21_4_0 に `#[ignore]` を追加。

---

## T7: `driver.rs` — v215000_tests 追加

```rust
#[cfg(test)]
mod v215000_tests {
    use super::*;
    use crate::lsp::{LspServer, protocol::RpcRequest};

    fn make_request(method: &str, id: i64, params: serde_json::Value) -> RpcRequest { ... }

    #[test] fn version_is_21_5_0() { ... }
    #[test] fn lsp_capabilities_code_action() { ... }
    #[test] fn lsp_capabilities_rename() { ... }
    #[test] fn lsp_capabilities_references() { ... }
    #[test] fn lsp_code_action_returns_array() { ... }
    #[test] fn lsp_code_action_convert_to_fstring() { ... }
    #[test] fn lsp_code_action_add_missing_import() { ... }
    #[test] fn lsp_references_returns_call_sites() { ... }
    #[test] fn lsp_references_empty_for_unknown() { ... }
    #[test] fn lsp_rename_fn_updates_occurrences() { ... }
    #[test] fn lsp_rename_returns_null_for_keyword() { ... }
    #[test] fn lsp_rename_response_contains_changes_key() { ... }
}
```

### テストコード例

```rust
// capabilities
fn lsp_capabilities_code_action() {
    let mut out = Vec::new();
    let mut server = LspServer::new(&mut out);
    server.handle(RpcRequest { id: Some(json!(1)), method: "initialize".into(), params: json!({}) }).unwrap();
    let text = String::from_utf8(out).unwrap();
    assert!(text.contains("\"codeActionProvider\":true"));
}

// rename
fn lsp_rename_fn_updates_occurrences() {
    let src = "fn double(n: Int) -> Int = n * 2\nfn main() -> Int = double(21)";
    let mut server = open_doc(&src, "file:///test.fav");
    let out = send_rename(&mut server, "file:///test.fav", Position { line: 0, character: 3 }, "multiply");
    assert!(out.contains("\"changes\""));
    assert!(out.contains("\"multiply\""));
}

// references
fn lsp_references_returns_call_sites() {
    let src = "fn double(n: Int) -> Int = n * 2\nfn main() -> Int = double(21)";
    let locs = send_references(&src, "file:///test.fav", Position { line: 1, character: 20 });
    // double is referenced at line 1
    assert!(locs.contains("\"line\":1") || locs.contains("\"line\":0"));
}
```

---

## T8: CHANGELOG + lsp.mdx

CHANGELOG.md 先頭に v21.5.0 エントリ追加:
```
## [v21.5.0] — 2026-06-20 — LSP コードアクション強化

### Added
- `textDocument/codeAction` — 3 種のコードアクション（addMissingImport / convertToFstring / inlineBinding）
- `textDocument/rename` — fn / type / stage の同一ファイル内一括リネーム（WorkspaceEdit）
- `textDocument/references` — シンボルの全参照箇所を Location リストで返す
- protocol.rs に TextEdit / WorkspaceEdit / CodeAction 型追加
- site/content/docs/tools/lsp.mdx 更新
```

`site/content/docs/tools/lsp.mdx` を新規作成または更新:
- Code Action セクション（CA-1〜CA-3 の説明・使い方）
- Rename セクション
- References セクション

---

## 実装上の注意事項

### DocumentStore の program フィールド（確認済み）

現在の `CheckedDoc` には `program` フィールドがない。**T0 で必ず追加すること**。
`definition.rs` は `doc.symbols` を参照しているため、既存フィールドには影響しない。
`program: Option<Program>` を追加し、`open_or_change` で格納する。

### `program.uses` の型（確認済み）

`ast.rs` の `Program.uses` は `Vec<Vec<String>>`（パスの各セグメントのリスト）。
`UseDecl` 構造体は存在しない。
NS チェックは `program.uses.iter().any(|path| path.first().map(|s| s == ns).unwrap_or(false))` を使う。

### Span の end 情報（確認済み）

`Span { file: String, start: usize, end: usize, line: u32, col: u32 }` — `start`/`end` バイトオフセットが存在する。
rename の TextEdit の end 文字位置は `span.col - 1 + (span.end - span.start)` で正確に計算できる。
`name_len` での近似は不要。

### KNOWN_RUNES の参照先

`code_action.rs` は `completion::KNOWN_RUNES` を `use crate::lsp::completion::KNOWN_RUNES;` で参照する。
独立したコピーを作らない（乖離防止）。
