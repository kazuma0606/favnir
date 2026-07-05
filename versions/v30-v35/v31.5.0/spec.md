# v31.5.0 仕様書 — LSP Inlay Hints（型推論結果インライン表示）

## 概要

`bind x <- expr` の型推論結果をエディタでインライン表示する。
LSP `textDocument/inlayHint` リクエストに対応し、
`bind` 変数の型を `: Type` 形式でヒント表示する。

---

## 背景

ロードマップ v31.5 より:

```favnir
bind rows <- LoadCsv(path)   // : List<RawRow>  ← インライン表示
bind n    <- List.length(rows) // : Int
```

---

## 既存実装の確認事項

| 項目 | 状態 |
|---|---|
| `textDocument/inlayHint` ハンドラ | **未実装** — 追加対象 |
| `initialize` 応答の `inlayHintProvider` | **未宣言** — 追加対象 |
| `type_at: HashMap<Span, Type>` | **実装済み** (`CheckedDoc`, document_store.rs) |
| `display_type()` / `Type::display()` | **実装済み** (lsp/hover.rs) |
| VS Code 拡張 `package.json` の inlayHints 機能 | **未記載** — 追加対象 |

---

## スコープ

### IN SCOPE

- `fav/Cargo.toml` — version `31.4.0` → `31.5.0`
- `fav/src/driver.rs` — `cargo_toml_version_is_31_4_0` をスタブ化
- `fav/src/lsp/inlay_hints.rs` — **新規作成**: `handle_inlay_hints()` 実装
- `fav/src/lsp/mod.rs` — `inlayHintProvider: true` を initialize 応答に追加
- `fav/src/lsp/mod.rs` — `"textDocument/inlayHint"` ハンドラを追加
- `fav/src/lsp/mod.rs` — `pub mod inlay_hints;` 宣言を追加
- `fav/editors/favnir-vscode/package.json` — `inlayHints` capability を追記
- `fav/src/driver.rs` — `v315000_tests`（3 件）追加（`use super::*` あり）
- `CHANGELOG.md` — `[v31.5.0]` セクション追加
- `benchmarks/v31.5.0.json` 新規作成
- `versions/current.md` — v31.5.0 に更新

### OUT OF SCOPE

- 関数の戻り型推論結果の inlay hint（ロードマップ v31.5 記載だが v31.6.0 以降に延期 — bind 変数ヒントを優先）
- `range` フィルタリング（リクエストの `range` パラメータによる絞り込み）— v31.5.0 では全ファイルを対象
- `InlayHintKind` の細分化（Type=1 のみ使用、Parameter=2 は使用しない）
- CRLF ファイルのサポート — `source.lines()` + `line.len() + 1` の計算は LF ファイルのみ正確。CRLF 対応は v31.6 以降
- タブ文字・マルチバイト文字（日本語変数名等）の `character` 正規化 — ASCII / LF 限定（UTF-16 コードユニット換算は行わない）
- リアルタイム補完との連携
- site/ MDX 更新

---

## 実装詳細

### InlayHint プロトコル構造

```rust
// fav/src/lsp/inlay_hints.rs
use serde::Serialize;
use crate::lsp::document_store::DocumentStore;
use crate::lsp::protocol::Position;

#[derive(Debug, Serialize)]
pub struct InlayHint {
    pub position: Position,
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
```

> `display_type()` は `hover.rs` のプライベート関数のため使用しない。`Type::display()` を直接呼ぶ（`ty.display()`）。

### `collect_bind_hints()` アルゴリズム

ソーステキストを行単位でスキャンし、`bind <name> <-` パターンを検出する:

1. 各行について `bind ` で始まる（またはインデント後に `bind ` がある）行を検出
2. `bind ` の直後にある変数名 `<name>` を抽出（`_` は除外）
3. 変数名のバイトオフセットを計算
4. `doc.type_at` からそのオフセット付近の Span の型を探す
5. 型が見つかれば InlayHint を生成（`position` = 変数名の末尾、`label` = `": Type"`）

```rust
fn collect_bind_hints(
    source: &str,
    type_at: &std::collections::HashMap<crate::frontend::lexer::Span, crate::middle::checker::Type>,
) -> Vec<InlayHint> {
    let mut hints = Vec::new();
    let mut byte_offset: usize = 0;
    for (line_idx, line) in source.lines().enumerate() {
        if let Some(rest) = find_bind_prefix(line) {
            // rest = "<name> <- ..."
            let name_end = rest.find(|c: char| !c.is_alphanumeric() && c != '_')
                .unwrap_or(rest.len());
            if name_end == 0 { byte_offset += line.len() + 1; continue; }
            let name = &rest[..name_end];
            if name == "_" { byte_offset += line.len() + 1; continue; }
            // name の先頭バイトオフセット
            let name_start_offset = byte_offset
                + (line.len() - rest.len());
            let name_end_offset = name_start_offset + name.len();
            // type_at から name_end_offset 付近の型を探す
            if let Some(ty) = find_type_at(type_at, name_start_offset, name_end_offset) {
                let col = (line.len() - rest.len() + name.len()) as u32;
                hints.push(InlayHint {
                    position: Position { line: line_idx as u32, character: col },
                    label: format!(": {}", ty.display()),
                    kind: 1,
                });
            }
        }
        byte_offset += line.len() + 1; // +1 for '\n'
    }
    hints
}
```

### LSP initialize 応答への追加

```rust
"capabilities": {
    ...,
    "inlayHintProvider": true   // ← 追加
}
```

### `textDocument/inlayHint` ハンドラ

```rust
"textDocument/inlayHint" => {
    let result = request.params
        .get("textDocument")
        .and_then(|td| td.get("uri"))
        .and_then(|u| u.as_str())
        .map(|uri| handle_inlay_hints(&self.store, uri))
        .and_then(|hints| serde_json::to_value(hints).ok())
        .unwrap_or_else(|| serde_json::json!([]));
    self.write_response(request.id.unwrap_or(serde_json::Value::Null), result)?;
    Ok(false)
}
```

### VS Code package.json 追記

`fav/editors/favnir-vscode/package.json` の `"contributes"` セクションに:

```json
"capabilities": {
    "inlayHints": {
        "resolveProvider": false
    }
}
```

---

## テスト設計（v315000_tests — 3 件）

| # | テスト名 | 確認内容 |
|---|---------|----------|
| 1 | `cargo_toml_version_is_31_5_0` | `Cargo.toml` に `version = "31.5.0"` |
| 2 | `benchmark_v31_5_0_exists` | `benchmarks/v31.5.0.json` に `"31.5.0"` |
| 3 | `lsp_inlay_hints_bind_variable` | `collect_bind_hints()` が `bind n <- ...` で `": Int"` 形式のヒントを生成する |

> テスト#3 は `collect_bind_hints()` を直接呼び出す。`type_at` に `"n"` のスパン（offset 5〜6）と `Type::Int` を手動で挿入し、ヒントが生成されること・ラベルが `": "` で始まることを検証する。
> `v315000_tests` は `use super::*` あり。

---

## 完了条件

- `Cargo.toml` version = `"31.5.0"`
- `lsp/inlay_hints.rs` に `handle_inlay_hints()` が実装されている
- LSP `initialize` 応答に `"inlayHintProvider": true` が含まれる
- `"textDocument/inlayHint"` ハンドラが LSP サーバに追加されている
- `cargo test v315000` — 3/3 PASS
- `cargo test` — 全件 PASS（0 failures）
- `CHANGELOG.md` に `[v31.5.0]` セクション
- `benchmarks/v31.5.0.json` 存在かつ `tests_passed` が実測値
- `versions/current.md` を v31.5.0 に更新
- `tasks.md` が COMPLETE
