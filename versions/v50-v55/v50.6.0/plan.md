# Plan: v50.6.0 — LSP ホバー情報強化（Rune メソッドシグネチャ）

Date: 2026-07-19

---

## 実装方針

### Step 1: 現状確認

```bash
# handle_hover の現状を確認
grep -n "handle_hover\|builtin_hover\|rune_hover" fav/src/lsp/hover.rs

# BUILTIN_FNS の構造を確認
grep -n "BuiltinFn\|pub const BUILTIN_FNS" fav/src/lsp/completion.rs | head -10

# rune.toml の [[exports]] が存在しないことを確認
grep -rn "exports" fav/runes/ 2>/dev/null || echo "no exports found"
```

**確認済み事項（事前調査）:**
- `handle_hover` は `type_at` スパンから型情報のみ返す
- `BUILTIN_FNS` は `completion.rs` に定義済み（`namespace`, `name`, `signature`, `params`）
- `rune.toml` に `[[exports]]` セクシ��ン未実装 → 静的 `RUNE_FNS` テーブルで代替
- `builtin_hover_at` / `rune_hover_at` は未存在 → v50.6.0 で追加

### Step 2: `lsp/hover.rs` — `RuneFn` + `RUNE_FNS` + ヘルパー追加

��存 `display_type` 関数の直後（`pub(crate) fn span_contains` の直前）に追加する。

追加内容（上から順に）:
1. `pub(crate) struct RuneFn` — rune, name, signature, effect, doc フィールド
2. `pub(crate) const RUNE_FNS: &[RuneFn]` — kafka(consume/produce)、csv(read/write) の 4 エントリ
3. `fn word_and_ns_at(source, offset)` — `(namespace, method)` を byte offset から抽出するヘルパー
4. `pub(crate) fn builtin_hover_at(source, offset)` — PascalCase NS → BUILTIN_FNS 検索
5. `pub(crate) fn rune_hover_at(source, offset)` — lowercase NS → RUNE_FNS 検索

### Step 3: `handle_hover` に builtin/rune lookup を追加

`position_to_char_offset` の直後に `builtin_hover_at` / `rune_hover_at` の優先チェックを挿入。
マッチしない場合は既存の `type_at` ロジッ��へフォールバック。

```rust
pub fn handle_hover(store: &DocumentStore, uri: &str, pos: Position) -> Option<Hover> {
    let doc = store.get(uri)?;
    let offset = position_to_char_offset(&doc.source, pos)?;

    // v50.6.0: try builtin / rune method hover first (text-scan, richer info)
    if let Some(content) = builtin_hover_at(&doc.source, offset)
        .or_else(|| rune_hover_at(&doc.source, offset))
    {
        return Some(Hover {
            contents: MarkupContent { kind: "markdown".to_string(), value: content },
        });
    }

    // existing type_at lookup (unchanged)
    let (span, ty) = doc
        .type_at
        .iter()
        .filter(|(span, _)| span_contains(span, offset))
        .min_by_key(|(span, _)| span.end.saturating_sub(span.start))?;

    let type_block = format!("```favnir\n{}\n```", display_type(ty));
    let value = match doc_comment_for_span(doc, span, offset) {
        Some(text) if !text.is_empty() => format!("{}\n\n{}", text, type_block),
        _ => type_block,
    };
    Some(Hover {
        contents: MarkupContent { kind: "markdown".to_string(), value },
    })
}
```

### Step 4: `v506000_tests` モジュール追加

`driver.rs` の `v505000_tests` 直前に挿入��る。

```rust
// -- v506000_tests (v50.6.0) -- LSP ホバー情報強化 --
#[cfg(test)]
mod v506000_tests {
    #[test]
    fn cargo_toml_version_is_50_6_0() {
        let content = include_str!("../Cargo.toml");
        assert!(content.contains("version = \"50.6.0\""),
            "Cargo.toml version should be 50.6.0");
    }

    #[test]
    fn lsp_hover_builtin_fn() {
        use crate::lsp::hover::builtin_hover_at;
        // "List.map(items, f)" — offset=5 is on "m" of "map"
        let source = "List.map(items, f)";
        let result = builtin_hover_at(source, 5);
        assert!(result.is_some(), "expected hover for List.map");
        let text = result.unwrap();
        assert!(text.contains("map"),  "hover should mention function name, got: {}", text);
        assert!(text.contains("List"), "hover should show List signature, got: {}", text);
    }

    #[test]
    fn lsp_hover_rune_method() {
        use crate::lsp::hover::rune_hover_at;
        // "kafka.consume(topic)" — offset=6 is on "c" of "consume"
        let source = "kafka.consume(topic)";
        let result = rune_hover_at(source, 6);
        assert!(result.is_some(), "expected hover for kafka.consume");
        let text = result.unwrap();
        assert!(text.contains("consume"), "hover should mention method name, got: {}", text);
        assert!(text.contains("Kafka"),   "hover should show !Kafka effect, got: {}", text);
    }
}
```

### Step 5: バージ��ン更新・完了

1. `fav/Cargo.toml` version → `"50.6.0"`
2. `v505000_tests::cargo_toml_version_is_50_5_0` を削除
3. `cargo test` 通過確認（3103）
4. `cargo clippy -- -D warnings` クリーン確認
5. `CHANGELOG.md` に v50.6.0 エントリ追加
6. `versions/current.md` 更新
7. `versions/roadmap/roadmap-v50.1-v51.0.md` の v50.6.0 実績を記入

---

## 注意事項

- `word_and_ns_at` は ASCII バイト���作��み使用。マルチバイト文字は未対応（既存関数と同様の許容済み制限）。
- `builtin_hover_at` はビルトイン（PascalCase）専用、`rune_hover_at` は rune（lowercase）専用で互いに排他。
- 既存の `handle_hover` テスト（`tests` モジュール内）は変更しない。
  新規追加の `builtin_hover_at` / `rune_hover_at` が `type_at` と競合しない入力（単純な `NS.method` 文字列）を使うため、既存テストは通過するはず。
- self-hosted ファイル（`compiler.fav` / `checker.fav`）への変更なし。
