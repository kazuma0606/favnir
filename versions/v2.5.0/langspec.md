# Favnir v2.5.0 Language Specification

更新日: 2026-05-13

## 概要

v2.5.0 では Favnir LSP に以下を追加する。

- `textDocument/completion`
- `textDocument/definition`
- doc comment の hover 表示

## Completion

`textDocument/completion` は 4 種類の候補を返す。

- field completion
  - `.` トリガー時に record 型のフィールド候補を返す
  - `label` はフィールド名、`detail` は型表示
  - `CompletionItemKind = Field (5)`
- global completion
  - top-level の `fn` / `type` / `stage` / `seq` / `interface` を返す
  - `fn` は `Function (3)`、それ以外は `Class (7)`
- keyword completion
  - `fn`, `type`, `stage`, `seq`, `interface`, `impl`, `match`, `if`, `else`,
    `bind`, `chain`, `collect`, `yield`, `public`, `async`, `for`, `in`,
    `where`, `bench`, `test`
  - `CompletionItemKind = Keyword (14)`
- snippet completion
  - `fn`, `type`, `interface`, `match`
  - `insertTextFormat = 2`

## Definition

`textDocument/definition` は識別子使用位置から top-level 定義位置へ移動する。

- checker は `def_at: HashMap<Span, Span>` を保持する
  - key: 使用箇所 span
  - value: 定義箇所 span
- 現在の対象は global symbol
  - `fn`
  - `type`
  - `stage`
  - `seq`
  - `interface`

LSP 返却値は `Location { uri, range }`。

## Hover と doc comment

定義直前の `//` コメントは doc comment として扱う。

```favnir
// Returns double of n.
fn double(n: Int) -> Int = n * 2
```

hover 表示は次の形式。

```markdown
Returns double of n.

```favnir
Int
```
```

doc comment は lexer ではなくソーステキストから直接抽出する。

## CheckedDoc

`CheckedDoc` は以下を保持する。

```rust
pub struct CheckedDoc {
    pub source: String,
    pub errors: Vec<TypeError>,
    pub type_at: HashMap<Span, Type>,
    pub symbols: Vec<LspSymbol>,
    pub def_at: HashMap<Span, Span>,
    pub doc_comments: HashMap<String, String>,
}
```

v2.5.0 実装では field completion 用の record field index も保持する。

## LSP Capabilities

初期化応答の capabilities は以下。

```json
{
  "capabilities": {
    "textDocumentSync": 1,
    "hoverProvider": true,
    "completionProvider": {
      "triggerCharacters": ["."]
    },
    "definitionProvider": true
  }
}
```

## 互換性

- `fav run` / `fav check` / `fav test` の挙動は変わらない
- 既存 hover は維持され、doc comment があれば先頭に追加される
- `textDocument/definition` は `null` から `Location` 返却に変わる
