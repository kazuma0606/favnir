# v21.5.0 Spec — LSP コードアクション強化

## 概要

LSP サーバーに 3 つの新機能を追加し、VS Code / Neovim でのリファクタリング体験を改善する。

- `textDocument/codeAction` — カーソル位置に応じた自動修正・リファクタリング提案
- `textDocument/rename` — fn / type / stage の一括リネーム（同一ファイル内）
- `textDocument/references` — シンボルの全参照箇所を一覧表示

**テーマ**: Developer Tooling Complete シリーズ第5弾 — 「LSP でリファクタリングを安全に」

---

## 動機

v9.11.0〜v9.12.0 で補完・定義ジャンプ・シグネチャヘルプを実装した。
次の摩擦点は「名前を変えると怖い」「String.concat を修正したい」という
リファクタリング時の安全性と自動化である。

---

## 新機能一覧

| LSP メソッド | 機能 | 説明 |
|---|---|---|
| `textDocument/codeAction` | コードアクション | カーソル位置で利用可能なアクション一覧を返す |
| `textDocument/rename` | リネーム | シンボルを新しい名前に一括置換（WorkspaceEdit） |
| `textDocument/references` | 参照検索 | シンボルの全参照箇所（Location リスト）を返す |

---

## 実装仕様

### プロトコル拡張（`src/lsp/protocol.rs`）

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
    // LSP 仕様: CodeActionKind は省略可能。Option<String> + skip_serializing_if で表現。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edit: Option<WorkspaceEdit>,
}
```

---

### コードアクション（`src/lsp/code_action.rs`）

**エントリポイント:**
```rust
pub fn handle_code_action(
    store: &DocumentStore,
    uri: &str,
    range: Range,
) -> Vec<CodeAction>
```

カーソル行を取得し、以下の3つのアクションを検出する。

#### CA-1: addMissingImport

```
条件: カーソル行が `<NS>.<fn>(` の形式 かつ
      NS が KNOWN_RUNES に含まれる かつ
      program.uses（型: Vec<Vec<String>>）の先頭要素に NS が存在しない
      チェック: uses.iter().any(|path| path.first().map(|s| s == ns).unwrap_or(false))
アクション:
  title: "Add missing import: use <ns>"
  kind:  Some("quickfix")
  edit:  line 0 の先頭に "use <ns>\n" を挿入する TextEdit
```

#### CA-2: convertToFstring

```
条件: カーソル行に "String.concat(" が含まれる
アクション:
  title: "Convert to f-string"
  kind:  Some("refactor.rewrite")
  edit:  None（手動修正を促す — edit は将来バージョンで実装）
```

#### CA-3: inlineBinding

```
条件: カーソル行が "bind <name> <- <expr>" の形式で
      その名前がブロック内で1回しか使われていない
アクション:
  title: "Inline binding `<name>`"
  kind:  Some("refactor.inline")
  edit:  None（手動修正を促す — edit は将来バージョンで実装）
```

#### CA-4: extractToStage（将来バージョン持ち越し）

ロードマップには「Extract to stage（選択範囲を新しい stage に抽出）」が定義されているが、
選択範囲を新規ステージに変換するためには複雑な AST 書き換えが必要なため、本バージョンでは実装しない。
CA-4 として v21.6 以降に持ち越す。

---

### 参照検索（`src/lsp/references.rs`）

**エントリポイント:**
```rust
pub fn handle_references(
    store: &DocumentStore,
    uri: &str,
    pos: Position,
) -> Vec<Location>

pub fn collect_symbol_occurrences(
    program: &Program,
    name: &str,
) -> Vec<Span>
```

**スコープ外（v21.5.0）**: ロードマップには「リネーム時に use 参照も追跡」とあるが、本バージョンでは `program.items` の AST 走査のみとし、`program.uses`（import 文）のシンボル追跡は v21.6 以降に持ち越す。

`collect_symbol_occurrences` は以下のノードを走査:
- `Expr::Ident(n, span)` — 変数参照
- `Item::FnDef(fd)` where `fd.name == name` — 宣言
- `Item::TrfDef(td)` where `td.name == name` — 宣言
- `Item::TypeDef(td)` where `td.name == name` — 宣言
- `TypeExpr::Named(n, _, span)` — 型参照
- `Expr::Apply(Expr::Ident(n, _), ...)` — 関数呼び出し（上記 Ident で兼ねる）

span → Range 変換:
- `Span { line: u32, col: u32, start: usize, end: usize }` のバイトオフセットを使う
- `start.character = span.col - 1`（0-based）
- `end.character = start.character + (span.end - span.start)` — ASCII 識別子はバイト長 = 文字長
- `start.line = span.line - 1`（0-based）、`end.line = start.line`（同一行を仮定）

---

### リネーム（`src/lsp/rename.rs`）

**エントリポイント:**
```rust
pub fn handle_rename(
    store: &DocumentStore,
    uri: &str,
    pos: Position,
    new_name: &str,
) -> Option<WorkspaceEdit>
```

実装:
1. `pos` でカーソル下のシンボル名を取得（テキストスキャン）
2. `collect_symbol_occurrences(program, name)` で全 Span を取得
3. 各 Span を `TextEdit { range, new_text: new_name }` に変換
4. `WorkspaceEdit { changes: { uri: Vec<TextEdit> } }` を返す
5. シンボルが見つからない場合（キーワード等）は `None` を返す

---

### LSP サーバー統合（`src/lsp/mod.rs`）

**`initialize` capabilities に追加:**
```json
{
  "codeActionProvider": true,
  "renameProvider": true,
  "referencesProvider": true
}
```

**新規ハンドラ追加:**
```rust
"textDocument/codeAction"  => { ... handle_code_action(...) ... }
"textDocument/rename"      => { ... handle_rename(...) ... }
"textDocument/references"  => { ... handle_references(...) ... }
```

**`textDocument/codeAction` params パース:**
```
params.textDocument.uri
params.range.start / end
```

**`textDocument/rename` params パース:**
```
params.textDocument.uri
params.position
params.newName
```

---

## テスト（v215000_tests、12件）

| テスト名 | 内容 |
|----------|------|
| `version_is_21_5_0` | Cargo.toml に `"21.5.0"` が含まれる |
| `lsp_capabilities_code_action` | initialize → `codeActionProvider: true` |
| `lsp_capabilities_rename` | initialize → `renameProvider: true` |
| `lsp_capabilities_references` | initialize → `referencesProvider: true` |
| `lsp_code_action_returns_array` | 任意位置で codeAction → JSON 配列を返す |
| `lsp_code_action_convert_to_fstring` | String.concat 行 → "Convert to f-string" アクション含む |
| `lsp_code_action_add_missing_import` | http.get 行（use なし）→ "Add missing import: use http" アクション |
| `lsp_references_returns_call_sites` | fn `double` の参照 → 定義と呼び出し箇所を含む Location リスト |
| `lsp_references_empty_for_unknown` | 不明シンボル → 空リスト |
| `lsp_rename_fn_updates_occurrences` | `double` → `multiply` リネーム → WorkspaceEdit に全箇所含む |
| `lsp_rename_returns_null_for_keyword` | `fn` キーワード位置でリネーム → null |
| `lsp_rename_response_contains_changes_key` | rename レスポンスに `"changes"` キーが存在する |

---

## 新規ファイル一覧

| ファイル | 役割 |
|----------|------|
| `fav/src/lsp/code_action.rs` | コードアクション 3 種（CA-1〜CA-3） |
| `fav/src/lsp/references.rs` | 参照検索 + `collect_symbol_occurrences` |
| `fav/src/lsp/rename.rs` | リネーム（WorkspaceEdit 生成） |

## 変更ファイル一覧

| ファイル | 変更内容 |
|----------|----------|
| `fav/src/lsp/document_store.rs` | `CheckedDoc` に `program: Option<Program>` フィールドを追加し `open_or_change` で格納 |
| `fav/src/lsp/protocol.rs` | TextEdit / WorkspaceEdit / CodeAction 型追加 |
| `fav/src/lsp/mod.rs` | 3 ハンドラ追加、capabilities 更新 |
| `fav/Cargo.toml` | version `21.4.0` → `21.5.0` |
| `CHANGELOG.md` | v21.5.0 エントリ追加 |
| `site/content/docs/tools/lsp.mdx` | 更新（コードアクション・リネーム・参照検索） |

---

## 完了条件

- [ ] `cargo test v215000` — 12/12 PASS
- [ ] `cargo test` — リグレッションなし（exit 0）
- [ ] `CHANGELOG.md` に v21.5.0 エントリ
- [ ] `fav/Cargo.toml` version が `21.5.0`
- [ ] `site/content/docs/tools/lsp.mdx` に更新済み
