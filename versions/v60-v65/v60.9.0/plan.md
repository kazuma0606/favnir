# v60.9.0 Plan — 安定化・DX チェックリスト

Date: 2026-07-31
Status: COMPLETE

---

## 変更ファイル一覧

| ファイル | 変更種別 | 内容 |
|---|---|---|
| `fav/src/driver.rs` | 追加 | `v60900_tests` モジュール（テスト 2 件） |

変更は `driver.rs` のみ。新規関数・新規ファイルは追加しない。

---

## 実装ステップ

### Step 1: `driver.rs` — `v60900_tests` モジュール追加

`v60800_tests` モジュールの直前（上側）に挿入する。

```rust
// -- v60900_tests (v60.9.0) -- 安定化・DX チェックリスト --
#[cfg(test)]
mod v60900_tests {
    use super::*;

    /// v60.2 (cmd_check_fix_src) + v60.6 (long_description) の統合確認
    #[test]
    fn dx_e2e_check_fix_lsp_consistent() {
        // クリーンなソースは fix なし
        let source = "stage Foo: Int -> Int = |x| { x + 1 }\n";
        let fix_out = cmd_check_fix_src(source, true);
        assert!(
            fix_out.is_empty() || fix_out.contains("0 fix") || fix_out.contains("no fix"),
            "clean source should produce no fixes; got: {:?}", fix_out
        );

        // E0102 の explain-error に Long Description セクションが含まれる（v60.6.0）
        let explain = cmd_explain_error_collect("E0102");
        assert!(explain.is_some(), "E0102 should be explainable");
        let text = explain.unwrap();
        assert!(
            text.contains("Long Description"),
            "explain output should include Long Description (v60.6.0); got: {:?}", text
        );
    }

    /// v60.5 (REPL :load / :debug / multiline) の統合確認
    #[test]
    fn dx_repl_pipeline_e2e() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("pipeline.fav");
        std::fs::write(
            &path,
            "stage AddOne: Int -> Int = |x| { x + 1 }\n",
        )
        .expect("write pipeline.fav");

        // :load
        let mut session = ReplSession::new();
        handle_load_cmd(path.to_str().unwrap(), &mut session);
        assert!(
            session.def_names.contains(&"AddOne".to_string()),
            "stage should be loaded via :load; def_names = {:?}", session.def_names
        );

        // :debug
        let debug_out = handle_debug_cmd("AddOne", &session);
        assert!(
            debug_out.contains("AddOne"),
            ":debug should show stage name; got: {:?}", debug_out
        );

        // multiline continuation
        assert!(needs_continuation("bind x <-\\"), "backslash should trigger continuation");
        assert!(
            needs_continuation("stage S: Int -> Int = |x| {"),
            "unclosed brace should trigger continuation"
        );
        assert!(
            !needs_continuation("bind x <- 42"),
            "complete line should not trigger continuation"
        );
    }
}
```

---

## 挿入位置サマリ

| 対象 | 挿入位置 |
|---|---|
| `v60900_tests` | `driver.rs` の `v60800_tests` の直前（上側） |

---

## 注意点

- `ReplSession` は `pub` でない構造体だが、`use super::*` により `v60900_tests` 内からアクセス可能。
- `handle_load_cmd` も `fn`（非公開）だが、同じく `use super::*` でアクセス可能。
- `cmd_check_fix_src` の戻り値はコードベースの状態に依存するが、クリーンな stage ソースは typo/未使用 bind を含まないため `fix_out.is_empty()` になる見込み。OR 条件で `"0 fix"` / `"no fix"` も許容する。
- ベーステスト数は 3347（ロードマップ記載の 3346 ではなく、v60.8.0 の XSS テスト追加分 +1）。
