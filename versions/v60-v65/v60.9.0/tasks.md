# v60.9.0 Tasks — 安定化・DX チェックリスト

Date: 2026-07-31
Status: COMPLETE

---

## T0: 事前確認

- [x] `cargo test` でベースラインが 3347 tests passed, 0 failed であることを確認
  （注: ロードマップ記載は 3346 だが v60.8.0 で XSS テスト追加のため実際は 3347）
- [x] `fav/Cargo.toml` のバージョンが `"60.0.0"` であることを確認
- [x] `v60900_tests` がまだ存在しないことを確認
  - `grep -c 'v60900_tests' fav/src/driver.rs` = 0 件
- [x] `v60800_tests` が存在すること（挿入先が実在すること）を確認
  - `grep -c 'v60800_tests' fav/src/driver.rs` ≥ 1 件
- [x] `cmd_check_fix_src` が存在することを確認
  - `grep -c 'pub fn cmd_check_fix_src' fav/src/driver.rs` ≥ 1 件
- [x] `cmd_explain_error_collect` が存在することを確認
  - `grep -c 'cmd_explain_error_collect' fav/src/driver.rs` ≥ 1 件
- [x] `handle_load_cmd` が存在することを確認
  - `grep -c 'fn handle_load_cmd' fav/src/driver.rs` ≥ 1 件

---

## T1: `driver.rs` — `v60900_tests` モジュール追加

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

- [x] `v60900_tests` モジュールを `v60800_tests` の直前（上側）に追加した
- [x] `use super::*;` が含まれている
- [x] `dx_e2e_check_fix_lsp_consistent` テストが含まれている
  - `cmd_check_fix_src` で fix なしを確認
  - `cmd_explain_error_collect("E0102")` で `Long Description` セクションを確認
- [x] `dx_repl_pipeline_e2e` テストが含まれている
  - `handle_load_cmd` → `def_names` 確認
  - `handle_debug_cmd` → 出力確認
  - `needs_continuation` 3 ケース確認

---

## T2: テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` を実行
- [x] `v60900_tests::dx_e2e_check_fix_lsp_consistent` pass
- [x] `v60900_tests::dx_repl_pipeline_e2e` pass
- [x] 総テスト数 **3349** tests passed, 0 failed を確認

---

## T3: 事後処理

- [x] `versions/current.md` を v60.9.0 / 3349 tests に更新
- [x] `versions/roadmap/roadmap-v60.1-v61.0.md` の v60.9.0 実績欄を更新
  - 実績欄に実際のテスト数（3349）と注記（ベース 3347 = ロードマップ記載 3346 + XSS テスト +1）を記録
- [x] CHANGELOG.md: サブバージョンのため個別エントリは不要（v61.0 でまとめて記載）
  - v61.0 記載範囲: v60.1〜v60.9 全機能
- [x] このファイル（tasks.md）を COMPLETE ステータスに更新

---

## コードレビュー指摘と対応

指摘なし（新規テスト追加のみ、セキュリティリスクなし）。

---

Status: COMPLETE
