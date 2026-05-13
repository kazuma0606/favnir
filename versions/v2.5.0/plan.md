# Favnir v2.5.0 実装計画

作成日: 2026-05-13

---

## Phase 0 — バージョン更新

`Cargo.toml` を `version = "2.5.0"` に変更。
`src/main.rs` の HELP テキストを `v2.5.0` に更新。

---

## Phase 1 — protocol.rs に新型を追加

### `src/lsp/protocol.rs`

```rust
// 補完アイテム
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompletionItem {
    pub label: String,
    pub kind: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(rename = "insertText", skip_serializing_if = "Option::is_none")]
    pub insert_text: Option<String>,
    #[serde(rename = "insertTextFormat", skip_serializing_if = "Option::is_none")]
    pub insert_text_format: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<MarkupContent>,
}

// 定義ジャンプ結果
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Location {
    pub uri: String,
    pub range: Range,
}

// LSP CompletionItemKind 定数
pub mod completion_kind {
    pub const TEXT: u32 = 1;
    pub const FUNCTION: u32 = 3;
    pub const FIELD: u32 = 5;
    pub const KEYWORD: u32 = 14;
    pub const SNIPPET: u32 = 15;
    pub const CLASS: u32 = 7;
}
```

---

## Phase 2 — Checker に def_at とシンボル収集を追加

### `src/middle/checker.rs`

#### 2-1: `LspSymbol` 型と `def_at` フィールドの追加

```rust
#[derive(Debug, Clone)]
pub enum SymbolKind { Function, Type, Stage, Seq, Interface }

#[derive(Debug, Clone)]
pub struct LspSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub detail: String,    // 型シグネチャ等の表示文字列
    pub def_span: Span,
}

// Checker 構造体に追加
pub struct Checker {
    // ... 既存フィールド ...
    pub def_at: HashMap<Span, Span>,        // 使用箇所span → 定義箇所span
    pub symbol_index: Vec<LspSymbol>,       // グローバルシンボル一覧
    global_def_spans: HashMap<String, Span>, // 名前 → 定義span（内部用）
}
```

#### 2-2: first_pass で定義 span を収集

`check_program` の first_pass（関数・型・interface 等の収集ループ）で、
各アイテムの名前と定義 span を `global_def_spans` と `symbol_index` に登録する。

```rust
// FnDef 収集時
self.global_def_spans.insert(fn_def.name.clone(), fn_def.name_span.clone());
self.symbol_index.push(LspSymbol {
    name: fn_def.name.clone(),
    kind: SymbolKind::Function,
    detail: format!("fn {}: {} -> {}", fn_def.name, param_types_str, ret_ty_str),
    def_span: fn_def.name_span.clone(),
});
```

型定義・stage・seq・interface も同様に登録する。

#### 2-3: check_expr の Ident アームで def_at を記録

```rust
Expr::Ident(name, span) => {
    // 既存の型解決ロジック...
    let ty = /* resolve name */;
    self.record_type(span, &ty);

    // def_at に記録（グローバル名が解決できた場合）
    if let Some(def_span) = self.global_def_spans.get(name) {
        self.def_at.insert(span.clone(), def_span.clone());
    }
    ty
}
```

---

## Phase 3 — CheckedDoc の拡張

### `src/lsp/document_store.rs`

```rust
#[derive(Debug, Default)]
pub struct CheckedDoc {
    pub source: String,
    pub errors: Vec<TypeError>,
    pub type_at: HashMap<Span, Type>,
    pub symbols: Vec<LspSymbol>,          // 追加
    pub def_at: HashMap<Span, Span>,      // 追加
    pub doc_comments: HashMap<String, String>, // 追加
}
```

`open_or_change` で `checker.symbol_index` と `checker.def_at` を `CheckedDoc` に移す。
`doc_comments` は `extract_doc_comments(&source)` で収集する（Phase 5 で実装）。

```rust
CheckedDoc {
    source: source.clone(),
    errors,
    type_at: checker.type_at,
    symbols: checker.symbol_index,
    def_at: checker.def_at,
    doc_comments: extract_doc_comments(&source),
}
```

---

## Phase 4 — 補完ハンドラ

### `src/lsp/completion.rs` （新規ファイル）

```rust
use crate::lsp::document_store::DocumentStore;
use crate::lsp::protocol::{CompletionItem, Position, completion_kind};
use crate::middle::checker::Type;

pub fn handle_completion(
    store: &DocumentStore,
    uri: &str,
    pos: Position,
    trigger_char: Option<&str>,
) -> Vec<CompletionItem> {
    let Some(doc) = store.get(uri) else { return vec![] };

    if trigger_char == Some(".") {
        // フィールド補完
        return field_completions(doc, pos);
    }

    // グローバル + キーワード + スニペット
    let mut items = global_completions(doc);
    items.extend(keyword_completions());
    items.extend(snippet_completions());
    items
}

fn field_completions(doc: &CheckedDoc, pos: Position) -> Vec<CompletionItem> {
    // カーソル 1 文字前の span を type_at から検索
    // 型が Type::Record(fields) なら fields をアイテムに変換
    ...
}

fn global_completions(doc: &CheckedDoc) -> Vec<CompletionItem> {
    doc.symbols.iter().map(|sym| CompletionItem {
        label: sym.name.clone(),
        kind: match sym.kind {
            SymbolKind::Function | SymbolKind::Stage => completion_kind::FUNCTION,
            SymbolKind::Type | SymbolKind::Interface => completion_kind::CLASS,
            SymbolKind::Seq => completion_kind::FUNCTION,
        },
        detail: Some(sym.detail.clone()),
        insert_text: None,
        insert_text_format: None,
        documentation: None,
    }).collect()
}

fn keyword_completions() -> Vec<CompletionItem> {
    const KEYWORDS: &[&str] = &[
        "fn", "type", "stage", "seq", "interface", "impl", "match",
        "if", "else", "bind", "chain", "collect", "yield", "public",
        "async", "for", "in", "where", "bench", "test",
    ];
    KEYWORDS.iter().map(|kw| CompletionItem {
        label: kw.to_string(),
        kind: completion_kind::KEYWORD,
        detail: None,
        insert_text: None,
        insert_text_format: None,
        documentation: None,
    }).collect()
}

fn snippet_completions() -> Vec<CompletionItem> {
    vec![
        CompletionItem {
            label: "fn".to_string(),
            kind: completion_kind::SNIPPET,
            detail: Some("fn definition".to_string()),
            insert_text: Some(
                "fn ${1:name}(${2:param}: ${3:Type}) -> ${4:RetType} {\n    $0\n}".to_string()
            ),
            insert_text_format: Some(2),
            documentation: None,
        },
        // type / interface / match スニペットも同様
    ]
}
```

### `src/lsp/mod.rs` に補完ハンドラを追加

```rust
// mod.rs に追加
pub mod completion;

// capabilities を更新
"completionProvider": { "triggerCharacters": ["."] }

// ハンドラを追加
"textDocument/completion" => {
    let items = extract_completion_target(&request.params)
        .map(|(uri, pos, trigger)| completion::handle_completion(&self.store, &uri, pos, trigger.as_deref()))
        .unwrap_or_default();
    let result = serde_json::to_value(items).unwrap_or(serde_json::Value::Array(vec![]));
    self.write_response(request.id.unwrap_or(serde_json::Value::Null), result)?;
    Ok(false)
}
```

`extract_completion_target` はリクエストパラメータから `(uri, pos, triggerChar?)` を取り出す補助関数。

---

## Phase 5 — 定義ジャンプハンドラ

### `src/lsp/definition.rs` （新規ファイル）

```rust
use crate::lsp::document_store::DocumentStore;
use crate::lsp::protocol::{Location, Position, Range};

pub fn handle_definition(
    store: &DocumentStore,
    uri: &str,
    pos: Position,
) -> Option<Location> {
    let doc = store.get(uri)?;
    let offset = position_to_char_offset(&doc.source, pos)?;

    // usage span を検索
    let (_, def_span) = doc.def_at.iter()
        .find(|(usage_span, _)| span_contains(usage_span, offset))?;

    // def_span をLine/character に変換
    let range = span_to_range(&doc.source, def_span);
    Some(Location { uri: uri.to_string(), range })
}
```

### `src/lsp/mod.rs` の定義ジャンプを有効化

```rust
// capabilities を更新
"definitionProvider": true

// ハンドラを更新
"textDocument/definition" => {
    let result = extract_hover_target(&request.params)  // 同じ位置抽出関数を流用
        .and_then(|(uri, pos)| definition::handle_definition(&self.store, &uri, pos))
        .map(|loc| serde_json::to_value(loc).unwrap_or(serde_json::Value::Null))
        .unwrap_or(serde_json::Value::Null);
    self.write_response(request.id.unwrap_or(serde_json::Value::Null), result)?;
    Ok(false)
}
```

---

## Phase 6 — doc comment の hover 統合

### `src/lsp/doc_comment.rs` （新規ファイル）

ソースを行単位でスキャンし、`fn`/`type`/`stage`/`seq`/`interface` の直前の `//` 行を収集する。

```rust
pub fn extract_doc_comments(source: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();
    let lines: Vec<&str> = source.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if let Some(decl_name) = extract_decl_name(trimmed) {
            // i-1, i-2, ... まで遡って連続する // 行を収集
            let doc = collect_preceding_comments(&lines, i);
            if !doc.is_empty() {
                result.insert(decl_name, doc);
            }
        }
    }
    result
}

fn extract_decl_name(line: &str) -> Option<String> {
    // "fn name(", "type Name =", "stage Name:", "seq Name =", "interface Name" を解析
    // 正規表現なしで簡易パース
    for prefix in &["fn ", "type ", "stage ", "seq ", "interface "] {
        if let Some(rest) = line.strip_prefix(prefix) {
            let name: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            if !name.is_empty() { return Some(name); }
        }
    }
    None
}
```

### `src/lsp/hover.rs` の更新

hover 表示に doc comment を付加する。

```rust
// handle_hover の結果に doc comment を追加
let doc_text = doc.doc_comments.get(/* 識別子名 */).cloned().unwrap_or_default();
let value = if doc_text.is_empty() {
    format!("```favnir\n{}\n```", display_type(ty))
} else {
    format!("{}\n\n```favnir\n{}\n```", doc_text, display_type(ty))
};
```

---

## Phase 7 — テスト追加

### `src/lsp/completion.rs`

```rust
#[test]
fn test_completion_returns_field_items_on_dot_trigger() {
    // type Point = { x: Int  y: Int }
    // "point." でフィールド補完 → ["x", "y"] が含まれる
}

#[test]
fn test_completion_returns_global_fn_name() {
    // "fn double" が定義されていて、グローバル補完に "double" が含まれる
}

#[test]
fn test_completion_includes_keywords() {
    // キーワード補完に "match", "fn", "bind" 等が含まれる
}

#[test]
fn test_completion_includes_snippets() {
    // スニペット補完に insertTextFormat=2 のアイテムが含まれる
}
```

### `src/lsp/definition.rs`

```rust
#[test]
fn test_definition_returns_location_for_global_fn() {
    // "divide" の使用箇所で definition を呼ぶと定義行の Location が返る
}

#[test]
fn test_definition_returns_none_for_unknown_position() {
    // 何もない位置では None が返る
}
```

### `src/lsp/doc_comment.rs`

```rust
#[test]
fn test_extract_doc_comment_before_fn() {
    // "// Returns double\nfn double..." → doc_comments["double"] == "Returns double"
}
```

### `src/lsp/mod.rs`

```rust
#[test]
fn test_lsp_completion_request_returns_items() {
    // textDocument/completion リクエストに対してアイテムリストが返る
}

#[test]
fn test_lsp_definition_request_returns_location() {
    // textDocument/definition リクエストに対して Location が返る
}

#[test]
fn test_lsp_capabilities_include_completion_and_definition() {
    // initialize レスポンスに completionProvider と definitionProvider: true が含まれる
}
```

---

## Phase 8 — ドキュメント

- `versions/v2.5.0/langspec.md` を作成
  - `textDocument/completion` の対応種別（field / global / keyword / snippet）
  - `textDocument/definition` の動作説明
  - `CheckedDoc` の新フィールド説明
  - doc comment 記法（`//` 行が定義直前にある場合に hover に表示）
  - v2.4.0 との互換性

---

## テスト数の見込み

v2.4.0 ベースライン: 584

- completion テスト: +4
- definition テスト: +2
- doc_comment テスト: +1
- mod.rs integration テスト: +3
- 合計目標: **594**（+10 程度）

---

## 注意点

### `FnDef.name_span` の存在確認

`global_def_spans` に登録するには関数名の span が必要。
`FnDef` が `name_span: Span` を持っていない場合は `span: Span`（関数全体）で代替する。

### `Type::Record` のフィールド情報

フィールド補完は `type_defs: HashMap<String, TypeBody>` から取得するが、
`type_defs` は `Checker` の private フィールド。
`CheckedDoc.type_field_map: HashMap<String, Vec<(String, Type)>>` を追加するか、
`CheckedDoc.symbols` に型定義のフィールド情報を含めるかを選択する。

最もシンプルな方法: `type_at` で得た `Type::Record(fields)` に直接フィールド名が含まれている場合はそこから取得する。含まれていない場合は `CheckedDoc.type_field_map` を別途用意する。

### カーソル前の式の型取得（フィールド補完）

`.` の直前の式の span を特定するのは難しい。実用的なアプローチ：

1. カーソル位置から遡り `.` を見つける
2. `.` の直前にある識別子の span を `type_at` で検索
3. 型が `Type::Record(fields)` → フィールド一覧を返す

これで `user.` → `user` の型 → フィールド一覧 の流れが実現できる。

### `span_to_range` の実装

`definition.rs` では span（byte offset）を LSP の `Position`（line/character）に変換する必要がある。
`hover.rs` の `position_to_char_offset` の逆変換を実装する。
