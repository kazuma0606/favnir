# Plan: v49.9.0 — v50.0 前調整・安定化

Date: 2026-07-18

---

## 実装方針

### Step 1: `language-maturity-overview.mdx` に v46〜v49 機能一覧を追加

v49.8.0 で作成した骨子に以下セクションを追記する。
テスト `language_maturity_overview_doc_exists` が `"v49"` の含有を確認するため、
バージョン番号を含む記述が必要。

追加内容（`## 関連ドキュメント` の前に挿入）:

```markdown
## v46〜v49 主要機能一覧

| バージョン | 機能 |
|---|---|
| v46 | `return` ガード節 |
| v47 | stdlib 2.0 拡充 |
| v48 | import 2.0 / `fav install` |
| v49 | `fav test` インラインテスト / セキュリティ審査 |
```

### Step 2: `v499000_tests` モジュール追加

`v498000_tests` の直前に挿入。

```rust
// -- v499000_tests (v49.9.0) -- v50.0 前調整・安定化 --
#[cfg(test)]
mod v499000_tests {
    #[test]
    fn cargo_toml_version_is_49_9_0() {
        let content = include_str!("../Cargo.toml");
        assert!(
            content.contains("49.9.0"),
            "Cargo.toml version should be 49.9.0"
        );
    }

    #[test]
    fn language_maturity_overview_doc_exists() {
        let content =
            include_str!("../../site/content/docs/language-maturity-overview.mdx");
        assert!(
            content.contains("| v49 |"),
            "language-maturity-overview.mdx should contain the v49 table row '| v49 |'"
        );
        assert!(
            content.contains("| v46 |"),
            "language-maturity-overview.mdx should contain the v46 table row '| v46 |'"
        );
    }
}
```

### Step 3: バージョン更新・完了

順序を守ること:
1. `Cargo.toml` version → `"49.9.0"`（先に更新）
2. `cargo test` 3087 passed 確認（`cargo_toml_version_is_49_9_0` は Cargo.toml 更新後でないと FAIL）
- `cargo clippy -- -D warnings` クリーン確認
- `cargo fmt -- --check` クリーン確認（フォーマット整合性）
- `CHANGELOG.md` 更新
- `versions/current.md` 更新
- `roadmap-v49.1-v50.0.md` 実績記入

---

## 注意事項

- `include_str!("../Cargo.toml")`: `src/driver.rs` → `../` = `fav/` → `fav/Cargo.toml`
- `include_str!("../../site/content/docs/language-maturity-overview.mdx")`: v49.8.0 で作成済み
- v49.9.0 はコードフリーズ版 — 新機能追加・API 変更は行わない
- `cargo_toml_version_is_49_9_0` テストは Cargo.toml を直接読み込むため、バージョン更新後に実行すること
