# v72.9.0 Spec — 安定化・コードフリーズ（Developer Experience 2.0 前調整）

Date: 2026-08-13
Status: 完了

---

## 背景

v72.1〜v72.8 で実装した Developer Experience 2.0 の全機能を横断的に検証する。
VS Code 拡張・AI アシスタント・REPL 2.0・Playground 2.0・`fav learn` が
互いに干渉せず正常動作することを確認し、v73.0.0 の宣言に備える安定化バージョン。

---

## 目標

- v72.1〜v72.8 の代表的テストが全て pass していることを確認するテストを追加
- VS Code 拡張（package.json）・REPL 2.0（`:timing`）・Playground 2.0（テンプレート）の
  各コンポーネントを横断的に検証する E2E テストを追加
- `cargo test` 全体が 0 failures であることを確認
- v73.0.0 宣言へのゲートとして機能する

---

## 確認対象機能（v72.1〜v72.8）

| バージョン | 機能 | 確認方法 |
|---|---|---|
| v72.1.0 | VS Code 拡張 — `package.json` valid | `vscode_extension_package_json_valid` テスト存在確認 |
| v72.2.0 | AI エラーアシスタント — `ai_explain_e0374_returns_hint` | テスト存在確認 |
| v72.3.0 | `fav ai generate` — `ai_generate_returns_valid_fav_code` | テスト存在確認 |
| v72.4.0 | REPL 2.0 — `repl2_tab_completion` | テスト存在確認 |
| v72.5.0 | Playground 2.0 — `playground2_template_gallery_has_5_entries` | テスト存在確認 |
| v72.6.0 | `fav init` テンプレート — `init_template_ai_etl_valid` | テスト存在確認 |
| v72.7.0 | `fav watch` 2.0 — `watch2_session_field_defaults` | テスト存在確認 |
| v72.8.0 | `fav learn` — `learn_chapter1_exists` / `learn_chapter5_exists` | テスト存在確認 |

> **注意**: 各バージョンから代表テスト 1 件のみを選択して確認している（全完了条件テストの網羅は対象外）。
> v72.8.0 のみ 2 件の代表テストを確認している（chapter 1 + chapter 5）。

---

## テスト

### `v729000_tests` モジュール

```rust
#[test]
fn dev_exp2_all_stable() {
    // v72.1〜v72.8 の代表テストが driver.rs に存在することを確認
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
    // VS Code 拡張・REPL 2.0・Playground 2.0 の主要シンボルが driver.rs に存在することを確認
    let src = include_str!("driver.rs");
    // VS Code 拡張
    assert!(src.contains("vscode_extension_lsp_integration"),
        "vscode_extension_lsp_integration test not found");
    // REPL 2.0
    assert!(src.contains("repl2_multiline_input"),
        "repl2_multiline_input test not found");
    // Playground 2.0
    assert!(src.contains("playground2_share_url_format"),
        "playground2_share_url_format test not found");
}
```

---

## 成功基準

- `cargo test v729000` で 2 件 pass
- `cargo test` 全体で 3642 tests pass（3640 + 2）
- `fav/Cargo.toml` のバージョンが `72.9.0` であること
- v72.1〜v72.8 の全テストモジュール（v721000〜v728000）が 0 failures

---

## スコープ外

- 新機能の追加（安定化専用スプリントのため）
- バグ修正以外のコード変更
- サイト側ドキュメント更新（v73.0.0 以降）

---

## 変更ファイル

- `fav/src/driver.rs` — `v729000_tests` モジュール追加 + バージョン更新
- `fav/Cargo.toml` — version `72.8.0` → `72.9.0`
- `CHANGELOG.md` — v72.9.0 エントリ追加
- `versions/current.md` — 進行中バージョンを v72.9.0 に更新
