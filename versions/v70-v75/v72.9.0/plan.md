# v72.9.0 実装計画 — 安定化・コードフリーズ（Developer Experience 2.0 前調整）

Date: 2026-08-13

---

## 実装ステップ

### T0: 事前確認
1. `fav/Cargo.toml` のバージョンが `72.8.0` であることを確認
2. `cargo test` が 3640 tests pass（0 failures）であることを確認
3. `driver.rs` に `v728000_tests` モジュールが存在することを確認
4. `driver.rs` に `v729000_tests` が未存在であることを確認
5. v72.1〜v72.8 の各テストモジュール（v721000〜v728000）が driver.rs に存在することを grep で確認

### T1: `v729000_tests` モジュール追加

`v728000_tests` モジュールの直後に `v729000_tests` を追加する。

```rust
#[cfg(test)]
mod v729000_tests {
    use super::*;

    #[test]
    fn dev_exp2_all_stable() {
        let src = include_str!("driver.rs");
        let required = vec![
            "vscode_extension_package_json_valid",
            "ai_explain_e0374_returns_hint",
            "ai_generate_returns_valid_fav_code",
            "repl2_tab_completion",
            "playground2_template_gallery_has_5_entries",
            "init_template_ai_etl_valid",
            "watch2_session_field_defaults",
            "learn_chapter1_exists",
            "learn_chapter5_exists",
        ];
        for name in &required {
            assert!(src.contains(name),
                "dev exp 2.0 stability check: test '{}' not found in driver.rs", name);
        }
    }

    #[test]
    fn vscode_repl2_playground2_e2e() {
        let src = include_str!("driver.rs");
        assert!(src.contains("vscode_extension_lsp_integration"),
            "vscode_extension_lsp_integration test not found");
        assert!(src.contains("repl2_multiline_input"),
            "repl2_multiline_input test not found");
        assert!(src.contains("playground2_share_url_format"),
            "playground2_share_url_format test not found");
    }
}
```

確認: `cargo test v729000` で 2 件 pass

### T2: バージョン更新（`fav/Cargo.toml` + `driver.rs`）

- `fav/Cargo.toml`: `version = "72.8.0"` → `version = "72.9.0"`
- `driver.rs` 内のバージョン文字列を一括 replace（`cargo_toml_version_is_X` テスト用）

### T3: 部分テスト確認

- `cargo test v729000` で 2 件 pass することを確認

### T4: 全体テスト確認

- `cargo test` 全体で 3642 tests pass（0 failures）を確認

### T5: `CHANGELOG.md` 更新

`## [v72.9.0]` エントリを先頭に追加:

```markdown
## [v72.9.0] — 2026-08-13 — 安定化・コードフリーズ（Developer Experience 2.0 前調整）

### Added
- `dev_exp2_all_stable` — v72.1〜v72.8 の代表テスト全存在確認
- `vscode_repl2_playground2_e2e` — VS Code 拡張・REPL 2.0・Playground 2.0 横断 E2E 確認

### Tests
- `dev_exp2_all_stable` — v72.1〜v72.8 の 9 件の代表テスト名が driver.rs に存在することを確認
- `vscode_repl2_playground2_e2e` — vscode/repl2/playground2/learn の 4 シンボルが存在することを確認
- 合計テスト数: 3642（+2）
```

### T6: `versions/current.md` 更新

- 「進行中バージョン」を `v72.9.0` に更新
- 「次に切る版」を `v73.0.0` に更新
- 「最終更新」を `2026-08-13 (v72.9.0)` に更新

> **確認**: v73.0.0 は `cargo clean` 必須のクリーンアップバージョンである。
> ロードマップ `roadmap-v72.1-v73.0.md` の v73.0.0 セクションで `cargo clean` / `MILESTONE.md` 更新などの要件を事前確認しておくこと。

### T7: 最終確認

- `cargo test v729000` で 2 件 pass
- `cargo test` 全体で 3642 tests pass（0 failures）
- `fav/Cargo.toml` のバージョンが `72.9.0`
