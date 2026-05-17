# Favnir v4.8.0 仕様書 — LSP (Language Server Protocol)

作成日: 2026-05-17

---

## 概要

Favnir の開発体験を向上させるため、Language Server Protocol (LSP) サーバーを実装する。VSCode・Neovim・Zed など LSP 対応エディタで Favnir を書く際にリアルタイムの型エラー表示・補完・ホバー情報を提供する。

**主な追加機能:**
- `fav lsp` コマンド（LSP サーバーとして起動）
- `textDocument/publishDiagnostics` — 保存不要のリアルタイム型エラー表示
- `textDocument/completion` — キーワード・組み込み namespace・rune 関数の補完
- `textDocument/hover` — カーソル位置の型情報・ドキュメント表示
- `textDocument/definition` — 関数定義への Go to Definition

---

## 設計方針

### トランスポート

LSP の標準的な JSON-RPC 2.0 over stdin/stdout with `Content-Length` フレーミングを使用。外部クレート（tower-lsp 等）は使用せず、`serde_json`（既存依存）のみで実装する。

```
Content-Length: 123\r\n
\r\n
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{...}}
```

### モジュール構成

```
fav/src/lsp/
  mod.rs        — cmd_lsp エントリポイント; サーバーメインループ
  transport.rs  — Content-Length フレーミングの読み書き
  types.rs      — 最小限の LSP 型定義（serde_json ベース）
  state.rs      — ドキュメントストア + 解析キャッシュ
  handlers.rs   — リクエスト・通知ハンドラ群
  completion.rs — 補完ロジック
  hover.rs      — ホバーロジック
```

### 解析パイプライン

LSP サーバーは `didOpen` / `didChange` を受け取るたびに以下を実行する：

```
ソーステキスト
  → Parser::parse_str         (パース)
  → load_all_items            (rune import 解決)
  → Checker::check_with_self  (型チェック)
  → TypeError → Diagnostic    (エラー変換)
  → textDocument/publishDiagnostics (クライアントに送信)
```

解析は同期的（スレッド不使用）。リクエストは逐次処理する。

---

## 対応するリクエスト・通知

### Phase 1: 基盤

| Method | 種別 | 説明 |
|--------|------|------|
| `initialize` | Request | サーバーケーパビリティを返す |
| `initialized` | Notification | 無視（初期化完了通知） |
| `shutdown` | Request | null を返す |
| `exit` | Notification | `process::exit(0)` |
| `$/cancelRequest` | Notification | 無視 |

### Phase 2: 診断

| Method | 種別 | 説明 |
|--------|------|------|
| `textDocument/didOpen` | Notification | ファイルを開いた → 解析 → 診断送信 |
| `textDocument/didChange` | Notification | ファイル変更 → 再解析 → 診断送信 |
| `textDocument/didClose` | Notification | 診断をクリア |
| `textDocument/publishDiagnostics` | Notification (送信) | エラーをエディタに表示 |

### Phase 3: 補完

| Method | 種別 | 説明 |
|--------|------|------|
| `textDocument/completion` | Request | 補完候補リストを返す |

### Phase 4: ホバー

| Method | 種別 | 説明 |
|--------|------|------|
| `textDocument/hover` | Request | 型情報・説明を返す |

### Phase 5: 定義ジャンプ

| Method | 種別 | 説明 |
|--------|------|------|
| `textDocument/definition` | Request | 定義位置を返す |

---

## `initialize` レスポンス

```json
{
  "capabilities": {
    "textDocumentSync": 1,
    "completionProvider": {
      "triggerCharacters": [".", " "]
    },
    "hoverProvider": true,
    "definitionProvider": true,
    "diagnosticProvider": {
      "interFileDependencies": false,
      "workspaceDiagnostics": false
    }
  },
  "serverInfo": {
    "name": "favnir-lsp",
    "version": "4.8.0"
  }
}
```

`textDocumentSync: 1` = Full（差分でなく全文送信）。v4.8.0 は最もシンプルな Full sync を使う。

---

## 診断（Diagnostics）

### TypeError → Diagnostic 変換

`Checker::check_with_self` が返す `TypeError` を LSP `Diagnostic` に変換する。

```rust
pub struct Diagnostic {
    pub range: Range,       // 行・列（0 ベース）
    pub severity: u32,      // 1 = Error
    pub code: String,       // "E0102" 等
    pub source: String,     // "favnir"
    pub message: String,
}
```

Favnir の `Span` は 1-based line を持つ。LSP は 0-based → `line - 1` で変換。
`col` は 0-based → そのまま使用（Favnir の `col` が 0-based であることを確認）。

### 診断の送信例

```json
{
  "jsonrpc": "2.0",
  "method": "textDocument/publishDiagnostics",
  "params": {
    "uri": "file:///path/to/main.fav",
    "diagnostics": [
      {
        "range": {
          "start": {"line": 4, "character": 2},
          "end": {"line": 4, "character": 10}
        },
        "severity": 1,
        "code": "E0102",
        "source": "favnir",
        "message": "undefined: `foo`"
      }
    ]
  }
}
```

---

## 補完（Completion）

### 補完候補の種類

1. **キーワード補完**: `fn`, `public`, `match`, `type`, `import`, `true`, `false`, `stage`, `test` 等
2. **組み込み namespace 補完**: `String.`, `List.`, `Map.`, `Option.`, `Result.`, `DB.`, `Http.`, `Log.`, `Env.`, etc.
3. **Namespace メソッド補完**: `String.` の後に `contains`, `split`, `concat` 等
4. **Rune 関数補完**: `import rune "log"` 後に `log.` で `info`, `warn`, `error` 等
5. **スニペット補完**: `fn` → `public fn name() -> ReturnType { ... }`

### トリガー条件

- `.` で区切られたとき: namespace 補完
- スペース・改行後: キーワード補完
- `import rune "X"` が宣言済み: rune 関数補完

### 組み込み namespace メソッドリスト（主要）

```
String:  contains, split, concat, length, starts_with, ends_with, trim, to_uppercase, to_lowercase, replace, from_bool, from_int
List:    map, filter, fold, any, all, length, append, first, last, reverse, zip, flat_map, sort_by
Map:     get, set, delete, keys, values, contains_key, size, merge, to_list
Option:  map, and_then, unwrap_or, is_some, is_none
Result:  map, and_then, unwrap_or, is_ok, is_err, ok, err
Int:     to_string, abs, max, min
Bool:    to_string
```

---

## ホバー（Hover）

### 対象

カーソル位置の識別子に対して型情報を返す。

```json
{
  "contents": {
    "kind": "markdown",
    "value": "**fn get_user** `(id: Int) -> Result<User, String> !Db`\n\nRetrieves a user by ID."
  },
  "range": {...}
}
```

### 実装戦略（v4.8.0）

1. カーソル位置のトークンを特定
2. 識別子の場合: チェッカーの環境スコープから型を検索
3. 組み込み namespace の場合: ハードコードされた型文字列を返す
4. 不明な場合: `null` を返す（ホバーなし）

---

## 定義ジャンプ（Go to Definition）

### 実装戦略（v4.8.0）

1. カーソル位置の識別子を特定
2. チェッカーが収集した関数定義の `Span` を参照
3. 同ファイル内の関数: そのまま返す
4. rune 内の関数: rune ファイルパスを返す
5. 組み込み: null を返す

---

## ポジション変換

LSP は 0-based line/character を使用。Favnir の `Span` は:
- `line`: 1-based（変換: `line - 1`）
- `col`: 0-based（そのまま使用）
- `start`/`end`: byte オフセット（UTF-8 文字数への変換が必要）

v4.8.0 では ASCII を基本とし、UTF-8 マルチバイト文字の精度は追求しない（byte offset ≈ char offset として扱う）。

---

## VSCode 拡張機能との連携

v4.8.0 では拡張機能のパッケージングはしない。以下の設定を `settings.json` に追加することで手動で使用可能：

```json
{
  "favnir.lsp.command": "fav lsp",
  "favnir.lsp.filePattern": "**/*.fav"
}
```

将来 v4.9.0 以降で `favnir-vscode` 拡張機能としてパッケージ化する。

---

## ロギング

LSP サーバーは stdio を使用するため、デバッグログは stderr に出力する：

```rust
fn lsp_log(msg: &str) {
    eprintln!("[favnir-lsp] {}", msg);
}
```

または `window/logMessage` 通知をクライアントに送る。

---

## エラーハンドリング

- パースエラー: 診断として送信（サーバーはクラッシュしない）
- 型エラー: 診断として送信
- rune import 失敗: 警告診断として送信
- JSON-RPC デコードエラー: stderr にログ、次のメッセージへ

---

## テスト方針

### ユニットテスト（`fav/src/lsp/` 内）

- `test_content_length_framing` — メッセージの読み書き
- `test_diagnostics_from_type_errors` — TypeError → Diagnostic 変換
- `test_completion_keywords` — キーワード補完候補
- `test_completion_string_namespace` — `String.` 後の補完
- `test_span_to_lsp_range` — Span → LSP Range 変換

### 統合テスト（`driver.rs`）

- `lsp_initialize_response` — `initialize` リクエストに正しいケーパビリティを返す
- `lsp_diagnostics_on_type_error` — 型エラーを含むファイルで診断が送信される
- `lsp_no_diagnostics_on_valid_file` — 正しいファイルで診断が空

---

## 既知の制約

- v4.8.0 は Full sync のみ（Incremental sync は将来）
- ワークスペース全体の診断は未対応（開いているファイルのみ）
- UTF-8 マルチバイト文字のカーソル位置精度は低い（ASCII 前提）
- rune の補完は `import rune "X"` が書かれたファイルのみ
- `window/showMessage` や `workspace/applyEdit` は未対応
- 同期処理のため、大きなファイルで遅延する可能性あり
