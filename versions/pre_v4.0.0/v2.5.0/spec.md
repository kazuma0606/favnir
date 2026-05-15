# Favnir v2.5.0 Language Specification

作成日: 2026-05-13

---

## テーマ

エディタなしでは書けない状態を解消する。

v2.5.0 では Favnir LSP に補完と定義ジャンプを追加する。

1. **`textDocument/completion`** — フィールド・グローバル・キーワード・スニペット補完
2. **`textDocument/definition`** — 関数名・型名の定義位置へジャンプ
3. **doc comment の hover 表示** — `//` コメントをシグネチャ表示に付加

---

## 1. 補完（textDocument/completion）

### 1-1. フィールド補完（トリガー文字 `.`）

```
// ユーザーが "user." と入力した瞬間
user.  →  [name: String, age: Int, role: String]
```

- カーソル直前の式の型を `type_at` から取得する
- 型が `Type::Record` なら `CheckedDoc.type_defs` のフィールド一覧を返す
- 各フィールドは `label: "name"`, `detail: "String"` の形式で返す
- `completionItemKind: Field (5)` を使用する

### 1-2. グローバル補完

カーソルが識別子入力中（`.` トリガーなし）のとき、全トップレベルシンボルを候補とする。

```
do  →  [double, Debug, ...]
```

- `CheckedDoc.symbols: Vec<LspSymbol>` （新フィールド）から取得
- `LspSymbol { name: String, kind: SymbolKind, detail: String }` の新型
- `SymbolKind`: `Function`, `Type`, `Stage`, `Seq`, `Interface`
- `completionItemKind: Function (3)` / `Class (7)` 等にマッピング

### 1-3. キーワード補完

静的リストから Favnir のキーワードを返す。

```
ma  →  [match, ...]
st  →  [stage, ...]
```

対象キーワード：`fn`, `type`, `stage`, `seq`, `interface`, `impl`, `match`, `if`, `else`,
`bind`, `chain`, `collect`, `yield`, `public`, `async`, `for`, `in`, `where`, `bench`, `test`

`completionItemKind: Keyword (14)`

### 1-4. スニペット補完

キーワードに加え、よく使うボイラープレートをスニペットとして提供する。

| トリガー | スニペット展開 |
|---|---|
| `fn` | `fn ${1:name}(${2:param}: ${3:Type}) -> ${4:RetType} {\n    $0\n}` |
| `type` | `type ${1:Name} = { ${2:field}: ${3:Type} }` |
| `interface` | `interface ${1:Name} {\n    ${2:method}: ${3:Type}\n}` |
| `match` | `match ${1:expr} {\n    ${2:pattern} => $0\n}` |

`insertTextFormat: Snippet (2)`

---

## 2. 定義ジャンプ（textDocument/definition）

### 2-1. グローバル関数・型の定義ジャンプ

F12（または Ctrl+Click）で関数名や型名の定義位置へジャンプする。

```
divide(n)  →  fn divide(...) の定義行へ
User { }   →  type User = { ... } の定義行へ
```

- `Checker` に `def_at: HashMap<Span, Span>` を追加
  - キー: 使用箇所の span（`Expr::Ident` など）
  - 値: 定義箇所の span
- 第 1 パスで `global_def_spans: HashMap<String, Span>` を構築（既存の first_pass に追加）
- `check_expr` の `Ident` アームで、解決した名前を `def_at` に記録
- `handle_definition` が `def_at` を検索し `Location { uri, range }` を返す

### 2-2. interface と impl 間のジャンプ（ベストエフォート）

`interface Foo` のメソッド名上で F12 → `impl Foo for Bar` の対応メソッドへ（またはその逆）。
v2.5.0 では単純な名前マッチングで実装し、将来的により精密にする。

---

## 3. doc comment の hover 表示

### 3-1. `//` コメントの保存

現状はレキサーが `//` コメントを読み捨てる。
v2.5.0 では LSP モードのみ、関数・型定義の直前にある `//` 行を保存し hover に表示する。

実装方針：レキサーを変更せず、ソーステキストを直接スキャンして
定義の直前の `//` 行を収集する（`src/lsp/doc_comment.rs` として独立）。

```
// Returns double of n.
fn double(n: Int) -> Int = n * 2
```

hover 時：

```markdown
Returns double of n.

```favnir
Int
```
```

### 3-2. `CheckedDoc` への統合

`CheckedDoc.doc_comments: HashMap<String, String>` — グローバル名 → doc comment テキスト

---

## 4. CheckedDoc の新フィールド

```rust
pub struct CheckedDoc {
    pub source: String,
    pub errors: Vec<TypeError>,
    pub type_at: HashMap<Span, Type>,
    // v2.5.0 追加
    pub symbols: Vec<LspSymbol>,
    pub def_at: HashMap<Span, Span>,
    pub doc_comments: HashMap<String, String>,
}
```

---

## 5. protocol.rs の追加型

```rust
// 補完アイテム
pub struct CompletionItem {
    pub label: String,
    pub kind: u32,              // LSP CompletionItemKind
    pub detail: Option<String>,
    pub insert_text: Option<String>,
    pub insert_text_format: Option<u32>,  // 1=PlainText, 2=Snippet
    pub documentation: Option<MarkupContent>,
}

// 定義ジャンプ
pub struct Location {
    pub uri: String,
    pub range: Range,
}
```

---

## 6. LSP capabilities の更新

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

---

## 7. 互換性

- v2.4.0 までの全コードはそのまま有効
- `fav run` / `fav check` / `fav test` には影響しない
- LSP の hover 挙動は変わらない（doc comment が付加されるだけ）
- `textDocument/definition` が `null` から実際の Location を返すようになる
