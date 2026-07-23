# Spec: v46.4.0 — LSP inlay hints 強化（bind 変数 + パイプラインステージ型表示）

Date: 2026-07-17
Status: TODO

---

## 概要

LSP `textDocument/inlayHint` で `bind` 変数とパイプラインステージの推論型を行末に表示する。
`collect_bind_hints` は既存実装だが、実際の LSP パスではチェッカーが `type_at` に
bind 変数の型を記録していないため、現状は常に空ヒントになっている。

本バージョンで:
1. チェッカーの `Stmt::Bind` ハンドラを修正して `type_at` に bind 変数の型を記録する
2. `collect_stage_hints` を新規追加してパイプラインステージ名の型ヒントを提供する

---

## 調査結果（実装前に確認済み）

### `collect_bind_hints` の現状（`inlay_hints.rs`）

テキストスキャンで `bind ` 行のバインド変数名のバイトオフセットを求め、
`type_at` を引くことで型ヒントを生成している（line 24〜62）。
ただし、チェッカーが `type_at` に bind パターンの型を記録していないため、
`DocumentStore` 経由の実 LSP パスでは常に hints が空になる。

### チェッカーの `Stmt::Bind` ハンドラ（`checker.rs` line ≈ 3962）

```rust
if let Pattern::Bind(name, span) = &b.pattern {
    if effective_ty == Type::Unknown { ... }  // W001
}
// check_pattern_bindings(Pattern::Bind(name, _)) → env.define のみ
```

`remember_type(span, ty)` が呼ばれていないため `type_at` に記録されない。

### `collect_stage_hints` の現状

未実装。`handle_inlay_hints` は `collect_bind_hints` のみを呼ぶ。

### テスト数

- v46.3.0 完了時: 2999（ロードマップ推定 2997 より +2 — 実態と乖離あり）
- 本バージョン完了時推定: 2999 + 2 = **3001**

---

## 変更対象

### §1 — `checker.rs`: `Stmt::Bind` ハンドラに `remember_type` 追加

`Stmt::Bind` 処理（line ≈ 3969）の `Pattern::Bind` 分岐に追加:

```rust
if let Pattern::Bind(name, span) = &b.pattern {
    // v46.4.0: bind 変数の型を LSP inlay hints 用に記録
    self.remember_type(span, &effective_ty);
    if effective_ty == Type::Unknown {
        self.type_warning("W001", ...);
    }
}
```

**注意**:
- `check_pattern_bindings` の `Pattern::Bind(name, _)` は変更しない（match arm 等の内部パターンに広く適用されてしまうため）
- `Stmt::Bind` ハンドラのみ修正してトップレベルの bind 文の変数のみ記録する
- 型アノテーション付き bind 文（`bind x: Int <- 42`）でも `effective_ty`（推論型）を記録する
  （アノテーション型との整合は E0009 チェックで別途検証済みのため）

### §2 — `inlay_hints.rs`: `collect_stage_hints` 追加

**現状**: `handle_inlay_hints` は `collect_bind_hints` のみを呼ぶ。
**変更後**: `collect_stage_hints` も結合して返す。

`collect_bind_hints` と対称な実装でソースラインの `stage ` プレフィックスを走査:
（`find_type_at` は `inlay_hints.rs` 内のプライベート関数のため変更不要。同ファイルから直接呼べる）

テキストスキャン方式の制限（コメント行・文字列リテラル内の `stage ` に誤検出する可能性）は
v46.4.0 のスコープ外として許容する（`collect_bind_hints` と同じ制限）。

```rust
pub(crate) fn collect_stage_hints(
    source: &str,
    type_at: &HashMap<Span, Type>,
) -> Vec<InlayHint> {
    // collect_bind_hints と同じパターン、ただし "stage " プレフィックスを検索
}

fn find_stage_prefix(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    trimmed.strip_prefix("stage ").map(|r| r.trim_start())
}
```

`handle_inlay_hints` を更新して両方を結合:

```rust
pub fn handle_inlay_hints(store: &DocumentStore, uri: &str) -> Vec<InlayHint> {
    let doc = match store.get(uri) {
        Some(d) => d,
        None => return vec![],
    };
    let mut hints = collect_bind_hints(&doc.source, &doc.type_at);
    hints.extend(collect_stage_hints(&doc.source, &doc.type_at));
    hints
}
```

### §3 — `driver.rs`: `v464000_tests` 追加

`v463000_tests` の直後に 2 件追加:

**`lsp_inlay_hints_type_annotation`**: `LspServer` 経由の統合テスト。
`textDocument/didOpen` → `textDocument/inlayHint` の JSON-RPC シーケンスを送り、
レスポンス文字列に `": Int"` が含まれることをアサートする。
（既存の `completion_request_returns_items` テストのパターンに倣う）

```rust
#[test]
fn lsp_inlay_hints_type_annotation() {
    use crate::lsp::LspServer;
    use crate::lsp::protocol::RpcRequest;
    let mut out = Vec::new();
    let mut server = LspServer::new(&mut out);
    // didOpen: bind x <- 42 を含む関数
    server.handle(RpcRequest {
        id: None,
        method: "textDocument/didOpen".to_string(),
        params: serde_json::json!({
            "textDocument": {
                "uri": "file:///hints.fav",
                "text": "fn f() -> Int {\n  bind x <- 42\n  x\n}"
            }
        }),
    }).expect("didOpen");
    // inlayHint を要求
    server.handle(RpcRequest {
        id: Some(serde_json::json!(1)),
        method: "textDocument/inlayHint".to_string(),
        params: serde_json::json!({
            "textDocument": { "uri": "file:///hints.fav" }
        }),
    }).expect("inlayHint");
    let text = String::from_utf8(out).expect("utf8");
    // §1 の checker 修正で type_at に型が記録され ": Int" ヒントが返る
    assert!(text.contains("\": Int\""), "expected ': Int' hint in response: {}", &text[..200.min(text.len())]);
}
```

**`lsp_inlay_hints_pipeline`**: `collect_stage_hints` の単体テスト（`collect_bind_hints` の
既存テスト `lsp_inlay_hints_bind_variable` と同じパターン）。
`type_at` を手動構築して関数を直接呼ぶ。

`Span::new` のシグネチャ: `Span::new(file: &str, start: usize, end: usize, line: u32, col: u32)`
（`lexer.rs:16` で定義。`start`/`end` はバイトオフセット、`end` は exclusive）

`"stage LoadData"` の場合:
- `"stage "` = 6 バイト（0-5）
- `"LoadData"` = バイト 6-13（8文字）、`end` = 14（exclusive）
- `Span::new("test", 6, 14, 1, 7)`（line=1, col=7 は LSP ヒントには未使用）

```rust
#[test]
fn lsp_inlay_hints_pipeline() {
    use crate::lsp::inlay_hints::collect_stage_hints;
    use crate::frontend::lexer::Span;
    use crate::middle::checker::Type;
    use std::collections::HashMap;
    // "stage LoadData" の 1 行ソース
    let source = "stage LoadData\n";
    // "LoadData" は byte offset 6-14
    let mut type_at = HashMap::new();
    type_at.insert(Span::new("test", 6, 14, 1, 7), Type::Int);
    let hints = collect_stage_hints(source, &type_at);
    assert!(!hints.is_empty(), "should generate a hint for 'stage LoadData'");
    assert!(
        hints[0].label.starts_with(": "),
        "hint label must start with ': ', got: {}",
        hints[0].label
    );
}
```

---

## 変更しないファイル

- `ast.rs`: 変更なし
- `parser.rs`: 変更なし
- `lsp/mod.rs`: 変更なし（`textDocument/inlayHint` ハンドラはすでに `handle_inlay_hints` を呼ぶ）
- `fav check --show-inference`: 変更なし（LSP 側で自動的に利用）

---

## 完了条件

- `cargo test` 全通過（failures=0、実績: 2999 + 2 = **3001** tests passed）
- `cargo clippy -- -D warnings` クリーン
- `v464000_tests` 2 件すべて pass（`lsp_inlay_hints_type_annotation` / `lsp_inlay_hints_pipeline`）
- `Stmt::Bind` の `Pattern::Bind` 分岐で `remember_type` が呼ばれること（§1）
- `collect_stage_hints` が `inlay_hints.rs` に追加されること（§2）
- `handle_inlay_hints` が bind + stage 両方のヒントを結合して返すこと
- `CHANGELOG.md` に v46.4.0 エントリ追加
- `versions/current.md` を v46.4.0（3001 tests）に更新
- `fav/Cargo.toml` version → `46.4.0`
