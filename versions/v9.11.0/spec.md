# Favnir v9.11.0 Spec

Date: 2026-06-02
Theme: LSP 補完 + go-to-definition 強化

---

## 概要

既存 LSP（hover / diagnostics / 基本フィールド補完 / 基本定義ジャンプ）に
モジュール補完・Rune 補完・Signature help を追加する。
VSCode 上での Favnir 開発体験を大幅に改善し、v10.0.0 の OSS 公開に備える。

---

## 現状（v9.10.0 時点）

`fav/src/lsp/` には以下が実装済み:

| 機能 | 状態 |
|---|---|
| `textDocument/hover` | 実装済み（型表示） |
| `textDocument/publishDiagnostics` | 実装済み（型エラー表示） |
| `textDocument/completion` | 部分実装（フィールド補完・グローバルシンボル・キーワード・スニペット） |
| `textDocument/definition` | 部分実装（ユーザー定義関数・型へのジャンプ） |

**不足**:
- `List.` / `String.` / `Map.` 等の後にビルトイン関数候補が出ない
- `import rune "` の後に Rune 名候補が出ない
- 関数呼び出しの引数ヒント（Signature help）がない
- `textDocument/signatureHelp` ハンドラが未登録

---

## 機能仕様

### A. モジュール補完（Module Completion）

**トリガー**: `.` の後、識別子がビルトイン名前空間と一致する場合

対象 namespace: `List` / `String` / `Map` / `Result` / `Option` / `IO` / `Json` / `Csv` / `Gen` / `Http` / `Llm` / `Env` / `DB` / `AWS` / `Debug` / `Schema` / `T` / `Float` / `Int`

各関数の表示形式:
```
List.map(f, xs)       — (('a -> 'b), List<'a>) -> List<'b>
List.filter(pred, xs) — (('a -> Bool), List<'a>) -> List<'a>
```

- `label`: 関数名のみ（例: `map`）
- `detail`: 関数フルシグネチャ（例: `(('a -> 'b), List<'a>) -> List<'b>`）
- `insertText`: 関数名のみ（`.` は既に入力済み）
- `kind`: Function (3)

### B. Rune 補完（Rune Import Completion）

**トリガー**: `import rune "` の直後、または文字列リテラル内

候補リスト（固定）: `aws` / `cache` / `csv` / `db` / `email` / `fs` / `gen` / `graphql` / `grpc` / `http` / `json` / `llm` / `queue` / `slack` / `sql`

- `label`: Rune 名（例: `http`）
- `detail`: 短い説明（例: `HTTP client !Http`）
- `kind`: Module (9)

### C. Signature Help（引数ヒント）

**LSP メソッド**: `textDocument/signatureHelp`
**トリガー**: `(` を入力したとき

対象:
- ユーザー定義 `fn` / `stage`
- ビルトイン namespace 関数（上記モジュール補完と同じ候補）

レスポンス形式:
```json
{
  "signatures": [{
    "label": "List.map(f: ('a -> 'b), xs: List<'a>) -> List<'b>",
    "parameters": [
      { "label": "f: ('a -> 'b)" },
      { "label": "xs: List<'a>" }
    ]
  }],
  "activeSignature": 0,
  "activeParameter": <現在の引数インデックス>
}
```

`initialize` レスポンスに以下を追加:
```json
"signatureHelpProvider": {
  "triggerCharacters": ["(", ","]
}
```

### D. 定義ジャンプ改善（Definition Enhancement）

現状: ユーザー定義 `fn` / `stage` / `type` へのジャンプが動作。
追加:
- **Rune 関数ジャンプ**: `http.get(...)` の `get` にカーソルを置いたとき、`runes/http/http.fav` の該当行へジャンプ
- **`seq` 内の stage 名ジャンプ**: `seq Pipeline = FetchOrders |> Save` の `FetchOrders` をクリックで定義へ

---

## 実装方針

### Rust 変更範囲

1. **`fav/src/lsp/completion.rs`**: モジュール補完・Rune 補完を追加
2. **`fav/src/lsp/mod.rs`**: `textDocument/signatureHelp` ハンドラを追加、`initialize` に `signatureHelpProvider` を追加
3. **`fav/src/lsp/signature.rs`** （新規）: Signature help ロジック

### データ定義

モジュール関数テーブル（静的）を `completion.rs` に定義:
```rust
struct BuiltinFn {
    namespace: &'static str,
    name: &'static str,
    signature: &'static str,  // 表示用
    params: &'static [&'static str],  // Signature help 用
}
```

### 既存テストとの互換性

既存 8 件のテストは変更なし。新規テストを `v9110_tests` モジュールに追加。

---

## 完了条件

| 条件 | 確認 |
|---|---|
| `List.` の後にビルトイン関数補完が出る | |
| `String.` の後にビルトイン関数補完が出る（型シグネチャ付き） | |
| `import rune "` の後に Rune 名補完が出る | |
| `foo(` の後に Signature help が表示される（ユーザー定義関数） | |
| `List.map(` の後に Signature help が表示される | |
| Rune 関数の定義ジャンプが `runes/<name>/<name>.fav` へ飛ぶ | |
| `seq` 内の stage 名ジャンプが動作する | |
| `cargo test v9110` — 5 件以上通過 | |
| `cargo test checker_fav_wire_self_check` 通過 | |
| `cargo test bootstrap` 維持 | |

---

## スコープ外（将来版へ延期）

- Tab 補完の詳細なランキング（使用頻度順ソート）
- `///` docstring の補完候補への組み込み（v9.8.0 で追加した doc コメント）
- インクリメンタルパース（補完レスポンスの高速化）
- `fav/src/lsp/` の Favnir 化
