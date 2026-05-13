# Favnir v2.5.0 タスクリスト

作成日: 2026-05-13

---

## Phase 0 — バージョン更新

- [x] `Cargo.toml`: `version = "2.5.0"` に変更
- [x] `src/main.rs`: HELP テキストを `v2.5.0` に更新

---

## Phase 1 — protocol.rs に新型を追加

### `src/lsp/protocol.rs`

- [x] `CompletionItem` 構造体を追加
  - [x] `label: String`
  - [x] `kind: u32`（LSP CompletionItemKind）
  - [x] `detail: Option<String>`（`skip_serializing_if = "Option::is_none"`）
  - [x] `insert_text: Option<String>`（`rename = "insertText"`）
  - [x] `insert_text_format: Option<u32>`（`rename = "insertTextFormat"`, 1=PlainText / 2=Snippet）
  - [x] `documentation: Option<MarkupContent>`
- [x] `Location` 構造体を追加（`uri: String`, `range: Range`）
- [x] `completion_kind` モジュールを追加（FUNCTION=3, FIELD=5, KEYWORD=14, SNIPPET=15, CLASS=7）

---

## Phase 2 — Checker にシンボル収集と def_at を追加

### `src/middle/checker.rs`

- [x] `SymbolKind` enum を追加（`Function`, `Type`, `Stage`, `Seq`, `Interface`）
- [x] `LspSymbol` 構造体を追加（`name`, `kind`, `detail`, `def_span`）
- [x] `Checker` 構造体に新フィールドを追加
  - [x] `pub def_at: HashMap<Span, Span>` — 使用 span → 定義 span
  - [x] `pub symbol_index: Vec<LspSymbol>` — グローバルシンボル一覧
  - [x] `global_def_spans: HashMap<String, Span>` — 内部用: 名前 → 定義 span
- [x] first_pass でグローバルシンボルの定義 span と `symbol_index` を収集
  - [x] `FnDef` → `SymbolKind::Function`
  - [x] `TypeDef` → `SymbolKind::Type`
  - [x] `TrfDef`（stage）→ `SymbolKind::Stage`
  - [x] `FlwDef`（seq）→ `SymbolKind::Seq`
  - [x] `InterfaceDecl` → `SymbolKind::Interface`
- [x] `check_expr` の `Expr::Ident` アームで `def_at` に使用箇所 → 定義箇所を記録

---

## Phase 3 — CheckedDoc の拡張

### `src/lsp/document_store.rs`

- [x] `CheckedDoc` に新フィールドを追加
  - [x] `pub symbols: Vec<LspSymbol>`
  - [x] `pub def_at: HashMap<Span, Span>`
  - [x] `pub doc_comments: HashMap<String, String>`
- [x] `open_or_change` を修正
  - [x] `checker.symbol_index` → `CheckedDoc.symbols`
  - [x] `checker.def_at` → `CheckedDoc.def_at`
  - [x] `extract_doc_comments(&source)` → `CheckedDoc.doc_comments`

---

## Phase 4 — 補完ハンドラ

### `src/lsp/completion.rs` （新規ファイル）

- [x] `handle_completion(store, uri, pos, trigger_char) -> Vec<CompletionItem>` を実装
- [x] フィールド補完（`trigger_char == Some(".")`）
  - [x] カーソル 1 文字前の offset から `type_at` でレコード型を検索
  - [x] `Type::Record(fields)` からフィールド名と型を `CompletionItem` に変換
  - [x] `kind = completion_kind::FIELD`
- [x] グローバル補完（トリガーなし）
  - [x] `doc.symbols` から全シンボルを `CompletionItem` に変換
  - [x] `kind` は `SymbolKind` に応じて `FUNCTION` / `CLASS` 等にマッピング
- [x] キーワード補完
  - [x] 20 種のキーワード静的リストを `CompletionItem`（`kind=KEYWORD`）として返す
- [x] スニペット補完
  - [x] `fn` / `type` / `interface` / `match` の 4 スニペット
  - [x] `insert_text_format = Some(2)`（Snippet）

### `src/lsp/mod.rs`

- [x] `pub mod completion;` を追加
- [x] `initialize` レスポンスの capabilities を更新
  - [x] `"completionProvider": { "triggerCharacters": ["."] }` を追加
- [x] `"textDocument/completion"` ハンドラを追加
  - [x] リクエストパラメータから `(uri, pos, triggerChar?)` を取り出す `extract_completion_target` 補助関数
  - [x] `completion::handle_completion` を呼び出してアイテムリストを返す

---

## Phase 5 — 定義ジャンプハンドラ

### `src/lsp/definition.rs` （新規ファイル）

- [x] `handle_definition(store, uri, pos) -> Option<Location>` を実装
  - [x] カーソル位置の offset を計算
  - [x] `doc.def_at` から使用 span を検索
  - [x] ヒットした定義 span を `Range`（line/character）に変換
  - [x] `Location { uri, range }` を返す
- [x] `span_to_range(source, span) -> Range` ヘルパーを実装
  - [x] span の start/end を `(line, character)` に変換

### `src/lsp/mod.rs`

- [x] `pub mod definition;` を追加
- [x] `initialize` レスポンスの capabilities を更新
  - [x] `"definitionProvider": true` に変更（`false` から）
- [x] `"textDocument/definition"` ハンドラを実装
  - [x] `definition::handle_definition` を呼び出して `Location` or `null` を返す

---

## Phase 6 — doc comment の hover 統合

### `src/lsp/doc_comment.rs` （新規ファイル）

- [x] `extract_doc_comments(source: &str) -> HashMap<String, String>` を実装
  - [x] ソースを行単位でスキャン
  - [x] `fn` / `type` / `stage` / `seq` / `interface` 宣言の直前の連続 `//` 行を収集
  - [x] 宣言名をキーとして HashMap に格納

### `src/lsp/mod.rs`

- [x] `pub mod doc_comment;` を追加

### `src/lsp/document_store.rs`

- [x] `open_or_change` で `doc_comment::extract_doc_comments(&source)` を呼ぶ

### `src/lsp/hover.rs`

- [x] `handle_hover` で `doc.doc_comments` を参照し、hover 表示に doc text を付加
  - [x] doc text がある場合: `"doc_text\n\n```favnir\nType\n```"` 形式
  - [x] doc text がない場合: 従来通り `"```favnir\nType\n```"` 形式

---

## Phase 7 — テスト追加

### `src/lsp/completion.rs`

- [x] `completion_returns_field_items_on_dot_trigger`: `.` トリガーでレコードフィールドが返る
- [x] `completion_returns_global_fn_name`: グローバル補完に定義した fn 名が含まれる
- [x] `completion_includes_keywords`: キーワード補完に "match", "bind" 等が含まれる
- [x] `completion_includes_snippets`: スニペットに `insertTextFormat=2` のアイテムが含まれる

### `src/lsp/definition.rs`

- [x] `definition_returns_location_for_global_fn`: 関数使用箇所で定義 Location が返る
- [x] `definition_returns_none_for_unknown_position`: 何もない位置で `None` が返る

### `src/lsp/doc_comment.rs`

- [x] `extract_doc_comment_before_fn`: `// doc\nfn name...` → `doc_comments["name"] == "doc"`
- [x] `extract_doc_comment_multiline`: 複数行 `//` コメントが改行を保って収集される

### `src/lsp/hover.rs`

- [x] `hover_includes_doc_comment_for_symbol_use`: doc comment が hover 表示に付加される（Codex 追加）

### `src/lsp/mod.rs`

- [x] `initialize_returns_capabilities`: `completionProvider` と `"definitionProvider":true` の両方を検証
- [x] `completion_request_returns_items`: `textDocument/completion` リクエストで空でないリストが返る
- [x] `definition_request_returns_location`: `textDocument/definition` リクエストで Location が返る

---

## Phase 8 — 最終確認・ドキュメント

### 最終テスト確認

- [x] `cargo build` で警告ゼロを確認
- [x] `cargo test` で全テスト通過を確認（v2.4.0 の 584 → 595）
- [x] `fav lsp` で補完リクエストに応答することを手動確認
- [x] `fav lsp` で定義ジャンプリクエストに応答することを手動確認

### ドキュメント作成

- [x] `versions/v2.5.0/langspec.md` を作成
  - [x] `textDocument/completion` の対応種別（field / global / keyword / snippet）の説明
  - [x] `textDocument/definition` の動作説明
  - [x] `CheckedDoc` の新フィールド（`symbols`, `def_at`, `doc_comments`）の説明
  - [x] doc comment 記法（`//` 行を定義直前に置くと hover に表示）
  - [x] LSP capabilities の変更点（completionProvider 追加、definitionProvider true）
  - [x] v2.4.0 との互換性（完全上位互換）

---

## 完了条件チェック

- [x] `user.` と入力するとフィールド候補が補完に出る
- [x] グローバル関数名・型名が補完候補に出る
- [x] `fn` / `match` 等のキーワードが補完候補に出る
- [x] スニペット補完で `insertTextFormat=2` のアイテムが含まれる
- [x] 関数名上で `textDocument/definition` を呼ぶと定義 Location が返る
- [x] `definitionProvider: true` が `initialize` レスポンスに含まれる
- [x] `completionProvider` が `initialize` レスポンスに含まれる
- [x] `//` doc comment が hover 表示に付加される
- [x] `cargo test` 全テスト通過
- [x] `cargo build` 警告ゼロ
- [x] `Cargo.toml` バージョンが `"2.5.0"`
- [x] `versions/v2.5.0/langspec.md` 作成済み
