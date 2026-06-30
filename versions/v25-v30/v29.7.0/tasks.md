# v29.7.0 Tasks — VS Code 拡張 公式リリース

**状態**: COMPLETE
**開始日**: 2026-06-30
**完了日**: 2026-06-30

---

## 事前確認（T0）

- [x] `Cargo.toml` の version が `29.6.0` であること
- [x] `cargo test --bin fav 2>&1 | grep "^test result"` が `2348 passed` を含むこと
- [x] `driver.rs` に `mod v297000_tests` が存在しないこと
- [x] `extensions/vscode-favnir/` ディレクトリが存在しないこと

---

## タスク一覧

| タスク | 内容 | 状態 |
|---|---|---|
| T1 | `Cargo.toml` version `29.6.0` → `29.7.0` | [x] |
| T2 | `extensions/vscode-favnir/package.json` 作成 | [x] |
| T3 | `extensions/vscode-favnir/language-configuration.json` 作成 | [x] |
| T4 | `extensions/vscode-favnir/syntaxes/favnir.tmLanguage.json` 作成（TextMate grammar）| [x] |
| T5 | `extensions/vscode-favnir/src/extension.ts` 作成（LSP クライアント）| [x] |
| T6 | `CHANGELOG.md` に `[v29.7.0]` セクション追加 | [x] |
| T7 | `benchmarks/v29.7.0.json` 作成（test_count: 2354）| [x] |
| T8 | `site/content/docs/tools/vscode-extension.mdx` 作成 | [x] |
| T9 | `driver.rs` に `v297000_tests` 6 件追加 | [x] |
| T10 | `cargo test --bin fav v297000` — 6/6 PASS 確認 | [x] |
| T11 | `cargo test --bin fav` — 2354 tests PASS 確認 | [x] |
| T12 | tasks.md を COMPLETE に更新 | [x] |

---

## テスト詳細（T9）

```rust
// v297000_tests (v29.7.0) -- VS Code 拡張
#[cfg(test)]
mod v297000_tests {
    #[test]
    fn vscode_package_json_exists() {
        let src = include_str!("../../extensions/vscode-favnir/package.json");
        assert!(
            src.contains("vscode-favnir"),
            "extensions/vscode-favnir/package.json must contain 'vscode-favnir'"
        );
    }
    #[test]
    fn vscode_grammar_exists() {
        let src = include_str!("../../extensions/vscode-favnir/syntaxes/favnir.tmLanguage.json");
        assert!(
            src.contains("source.favnir"),
            "syntaxes/favnir.tmLanguage.json must contain 'source.favnir'"
        );
    }
    #[test]
    fn vscode_language_config_exists() {
        let src = include_str!("../../extensions/vscode-favnir/language-configuration.json");
        assert!(
            src.contains("lineComment"),
            "language-configuration.json must contain 'lineComment'"
        );
    }
    #[test]
    fn vscode_extension_src_exists() {
        let src = include_str!("../../extensions/vscode-favnir/src/extension.ts");
        assert!(
            src.contains("LanguageClient"),
            "extension.ts must contain 'LanguageClient'"
        );
    }
    #[test]
    fn vscode_extension_mdx_exists() {
        let src = include_str!("../../site/content/docs/tools/vscode-extension.mdx");
        assert!(
            src.contains("vscode-favnir"),
            "vscode-extension.mdx must contain 'vscode-favnir'"
        );
    }
    #[test]
    fn changelog_has_v29_7_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(
            src.contains("[v29.7.0]") || src.contains("## v29.7.0"),
            "CHANGELOG.md must contain '[v29.7.0]'"
        );
    }
}
```

---

## 完了条件チェックリスト

- [x] `Cargo.toml` version = "29.7.0"
- [x] `extensions/vscode-favnir/package.json` が存在し `vscode-favnir` を含む
- [x] `extensions/vscode-favnir/syntaxes/favnir.tmLanguage.json` が存在する（TextMate grammar）
- [x] `extensions/vscode-favnir/language-configuration.json` が存在する
- [x] `extensions/vscode-favnir/src/extension.ts` が存在し LSP クライアント起動コードを含む
- [x] `CHANGELOG.md` に `[v29.7.0]` セクションあり
- [x] `benchmarks/v29.7.0.json` 存在（test_count: 2354）
- [x] `site/content/docs/tools/vscode-extension.mdx` 存在
- [x] `cargo test --bin fav v297000` — 6/6 PASS
- [x] `cargo test --bin fav` — 2354 tests PASS
- [x] tasks.md を COMPLETE に更新
