# v60.3.0 Plan — LSP Code Action（Quick Fix / Rename Symbol）

Date: 2026-07-30

---

## 実装方針

`v60300_tests` モジュールを `v60200_tests` の直前（上側）に追加するだけ。
`code_action.rs` / `rename.rs` はすでに完全実装済みのため実装作業ゼロ。

---

## ステップ詳細

### Step 1: `v60300_tests` モジュール追加（`driver.rs`）

`v60200_tests` の直前に挿入する。

```rust
// -- v60300_tests (v60.3.0) -- LSP Code Action / Rename Symbol --
#[cfg(test)]
mod v60300_tests {
    use crate::lsp::code_action::handle_code_action;
    use crate::lsp::document_store::DocumentStore;
    use crate::lsp::protocol::{Position, Range};
    use crate::lsp::rename::handle_rename;

    fn pos(line: u32, character: u32) -> Position {
        Position { line, character }
    }

    fn range(sl: u32, sc: u32, el: u32, ec: u32) -> Range {
        Range { start: pos(sl, sc), end: pos(el, ec) }
    }

    fn store_with(uri: &str, src: &str) -> DocumentStore {
        let mut store = DocumentStore::new();
        store.open_or_change(uri.to_string(), src.to_string());
        store
    }

    #[test]
    fn lsp_code_action_e0001_quickfix() {
        // userId を定義して user_id を参照 → E0102 + "did you mean `userId`?" hint
        // → handle_code_action が "Did you mean `userId`?" Quick Fix を返す
        let src = "fn go(userId: Int) -> Int { user_id }";
        let store = store_with("file:///qa.fav", src);
        // user_id は line 0、col 28 付近（0-indexed）
        let actions = handle_code_action(&store, "file:///qa.fav", range(0, 28, 0, 35));
        let has_quickfix = actions.iter().any(|a| {
            a.title.contains("Did you mean")
                && a.kind.as_deref() == Some("quickfix")
        });
        assert!(
            has_quickfix,
            "expected 'Did you mean' quickfix action but got: {:?}",
            actions.iter().map(|a| &a.title).collect::<Vec<_>>()
        );
    }

    #[test]
    fn lsp_rename_variable() {
        // 関数引数 userId を newName にリネーム → WorkspaceEdit が返り全 edit が newName になる
        let src = "fn go(userId: Int) -> Int { userId }";
        let store = store_with("file:///rb.fav", src);
        // "fn go(userId..." — userId は line 0、char 6 から始まる
        let edit = handle_rename(&store, "file:///rb.fav", pos(0, 6), "newName");
        assert!(edit.is_some(), "expected WorkspaceEdit for variable rename but got None");
        let edit = edit.unwrap();
        let edits = edit.changes.get("file:///rb.fav").expect("edits for uri");
        assert!(!edits.is_empty(), "expected at least one text edit");
        assert!(
            edits.iter().all(|e| e.new_text == "newName"),
            "all edits should use new name 'newName'"
        );
    }
}
```

---

## 注意事項

- **W001 LSP Quick Fix はスコープ外**: `DocumentStore.CheckedDoc` に `lint_errors` フィールドがないため
  LSP 側では L002 を取得できない。CLI 版（v60.2.0 の `cmd_check_fix_src`）と異なりスコープ外とする。
- **テスト名 `lsp_code_action_e0001_quickfix`**: ロードマップ記載に合わせるが、実際は E0102 Quick Fix を検証する。
  コメントで E0102 であることを明記する。
- `Cargo.toml` version は `"60.0.0"` のまま変更しない
- rolling check の更新は不要
- `v60300_tests` は `v60200_tests` の直前（上側）に追加（driver.rs の慣例）
- `use super::*` は不要（`cmd_check_fix_src` 等を呼ばないため）
- `store_with` ヘルパーは v215000_tests と同じパターン（コピーして独立させる）
- `DocumentStore::open_or_change` が内部で `Checker::check_with_self` を実行し errors に E0102 を積む
- テスト実行: `cargo test -j 8 -- --test-threads=8`

---

## テスト数推移

| バージョン | テスト数 | 増加 |
|---|---|---|
| v60.2.0（ベース） | 3334 | — |
| v60.3.0 | 3336 | +2 |
