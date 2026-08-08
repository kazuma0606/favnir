# v60.3.0 Tasks — LSP Code Action（Quick Fix / Rename Symbol）

Date: 2026-07-30
Status: COMPLETE

---

## T0: 事前確認

- [x] `cargo test` でベースラインが 3334 tests passed, 0 failed であることを確認
- [x] `fav/Cargo.toml` のバージョンが `"60.0.0"` であることを確認
- [x] `v60300_tests` がまだ存在しないことを確認
  - `grep -c 'v60300_tests' fav/src/driver.rs` = 0 件
- [x] `lsp_code_action_e0001_quickfix` がまだ存在しないことを確認
  - `grep -c 'lsp_code_action_e0001_quickfix' fav/src/driver.rs` = 0 件
- [x] `lsp_rename_variable` がまだ存在しないことを確認
  - `grep -c 'lsp_rename_variable' fav/src/driver.rs` = 0 件

---

## T1: `v60300_tests` モジュール追加（`driver.rs`）

`v60200_tests` の直前（上側）に挿入する。

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

- [x] `v60300_tests` モジュールを `v60200_tests` の直前（上側）に追加した
  （driver.rs は新しい順＝ファイル上部に追加する慣例）
- [x] `use super::*;` は使用しない（crate パス直接指定）
- [x] `lsp_code_action_e0001_quickfix` テストが含まれている（実際は E0102 Quick Fix を検証）
- [x] `lsp_rename_variable` テストが含まれている
- [x] W001 LSP Quick Fix はスコープ外として実装しない
  （`CheckedDoc` に lint_errors がないため；CLI 版は v60.2.0 で実装済み）

---

## T2: テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` を実行
- [x] `v60300_tests::lsp_code_action_e0001_quickfix` pass
- [x] `v60300_tests::lsp_rename_variable` pass
- [x] 総テスト数 **3336** tests passed, 0 failed を確認

---

## T3: 事後処理

- [x] `versions/current.md` を v60.3.0 / 3336 tests に更新
- [x] `versions/roadmap/roadmap-v60.1-v61.0.md` の v60.3.0 実績欄を更新
  - `**実績**: — （未実施）` → `**実績**: 3336 tests passed, 0 failed（2026-07-30 完了）`
  - 「`code_action.rs` / `rename.rs` は実装済みのためテストのみ追加」注記を追記
  - 「W001 LSP Quick Fix は CheckedDoc に lint_errors なしのためスコープ外（E0102 のみ対応）」注記を追記
  - 「テスト名 `lsp_code_action_e0001_quickfix` は実際には E0102 を検証（ロードマップ記載の E0001 とは異なる）」注記を追記
- [x] CHANGELOG.md: サブバージョンのため個別エントリは不要
- [x] このファイル（tasks.md）を COMPLETE ステータスに更新

---

## コードレビュー指摘と対応

spec-reviewer 指摘（実装前）:
- [HIGH] W001 LSP Quick Fix 未記載 → spec/plan/tasks にスコープ外・理由付きで明記
- [MED] テスト名 e0001 vs E0102 不整合 → 3 ファイルに注記追加（テスト名はロードマップ準拠で変更せず）
- [LOW] CHANGELOG 方針 → v60.2.0 と同じ「サブバージョンのため不要」で整合確認済み

実装一発 pass 後 code-reviewer 指摘を受けて修正:
- [MED] E0102 hints が空の場合の偽陰性リスク
  → `doc.errors` に E0102 + hints 存在を前提アサートとして追加
- [MED] カラム位置コメントが誤解を招く
  → コメントを「check_did_you_mean_fix は行フィルタリングのみ・列値は任意」と修正
  → range(0, 28, 0, 35) → range(0, 0, 0, 0)（任意値であることを明示）
- [LOW] `edit.unwrap()` → `.expect(...)` に変更（is_some() assert も統合）
- [LOW] テスト名 `lsp_code_action_e0001_quickfix` はロードマップ準拠で変更なし
- [LOW] ヘルパー重複・use super 不統一 → 設計上の制約につき対応なし

---

Status: COMPLETE
