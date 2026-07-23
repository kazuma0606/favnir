# Plan: v49.8.0 — ドキュメントサイト全面更新 Phase 2 + CHANGELOG 整理

Date: 2026-07-18

---

## 実装方針

### Step 1: `site/content/docs/language-maturity-overview.mdx` 新規作成

`site/content/docs/` 配下に新規作成する。
`include_str!` パスは `driver.rs` から `../../site/content/docs/language-maturity-overview.mdx`。

内容:
- frontmatter: `title`, `category: "Docs"`, `description`
- v50.0 言語成熟度の概要（return / stdlib / modules / inline tests の4本柱）
- 文字列 `"Language Maturity"` と `"v50"` を必ず含める

### Step 2: `MILESTONE.md` 更新

既存の `MILESTONE.md` に Language Maturity セクションを追加。
`"Language Maturity"` という文字列を含むエントリを記載する。

### Step 3: `v498000_tests` モジュール追加

`v497000_tests` の直前に挿入。

```rust
// -- v498000_tests (v49.8.0) -- ドキュメントサイト全面更新 Phase 2 --
#[cfg(test)]
mod v498000_tests {
    #[test]
    fn docs_site_v50_overview_exists() {
        let content = include_str!("../../site/content/docs/language-maturity-overview.mdx");
        assert!(
            content.contains("Language Maturity") && content.contains("v50"),
            "language-maturity-overview.mdx should contain 'Language Maturity' and 'v50'"
        );
    }

    #[test]
    fn milestone_has_language_maturity() {
        let content = include_str!("../../MILESTONE.md");
        assert!(
            content.contains("Language Maturity"),
            "MILESTONE.md should contain 'Language Maturity'"
        );
    }
}
```

### Step 4: バージョン更新・完了

- `Cargo.toml` version → `"49.8.0"`
- `cargo test` 3085 passed 確認
- `cargo clippy` クリーン確認
- `CHANGELOG.md` 更新
- `versions/current.md` 更新
- `roadmap-v49.1-v50.0.md` 実績記入

---

## 注意事項

- `include_str!` パスは `src/driver.rs` から相対: `../../` = repo root (`favnir/`)
- `MILESTONE.md` は `favnir/` 直下に存在するか事前確認が必要
- v49.8.0 では `language-maturity-overview.mdx` は骨子のみ（完成は v49.9.0）
