# v67.9.0 実装計画

## ステップ

### Step 1: `site/content/docs/tools/developer-intelligence.mdx` 新規作成

MDX ファイルを作成し、v67.1〜v67.8 の全機能を紹介する。

必須キーワード（テスト要件）: `"fav debug"`

### Step 2: `driver.rs` — `v67900_tests` 追加

`// -- v67800_tests (v67.8.0)` の直前に挿入（新しいものが上）:

```rust
// -- v67900_tests (v67.9.0) -- 安定化・コードフリーズ --
#[cfg(test)]
mod v67900_tests {
    #[test]
    fn dev_intelligence_all_stable() {
        // debug / viz / suggest / simulate の各ソースが存在し必要な定義を含む
        let debug_src   = include_str!("debug.rs");
        let viz_src     = include_str!("viz.rs");
        let suggest_src = include_str!("suggest.rs");
        let simulate_src = include_str!("simulate.rs");
        assert!(debug_src.contains("cmd_debug"),     "debug.rs should define cmd_debug");
        assert!(viz_src.contains("cmd_viz"),          "viz.rs should define cmd_viz");
        assert!(suggest_src.contains("cmd_suggest"),  "suggest.rs should define cmd_suggest");
        assert!(simulate_src.contains("cmd_simulate"), "simulate.rs should define cmd_simulate");
    }

    #[test]
    fn debug_viz_suggest_docs_complete() {
        let mdx = include_str!("../../site/content/docs/tools/developer-intelligence.mdx");
        assert!(
            mdx.contains("fav debug"),
            "developer-intelligence.mdx should mention 'fav debug'"
        );
    }
}
```

## `include_str!` パスの計算

`driver.rs` は `fav/src/driver.rs` にある。

| ファイル | `include_str!` パス |
|---|---|
| `fav/src/debug.rs` | `"debug.rs"` |
| `fav/src/viz.rs` | `"viz.rs"` |
| `fav/src/suggest.rs` | `"suggest.rs"` |
| `fav/src/simulate.rs` | `"simulate.rs"` |
| `site/content/docs/tools/developer-intelligence.mdx` | `"../../site/content/docs/tools/developer-intelligence.mdx"` |

## 注意事項

- v67.9.0 はコードフリーズ版: `fav/src/*.rs` のコード変更は行わない
- sub-version ポリシー: `Cargo.toml` / `CHANGELOG.md` は変更しない
