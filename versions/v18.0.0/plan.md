# v18.0.0 — 実装計画

## 方針

- v18.0.0 はコードの追加・変更なし（マイルストーン宣言）
- 作業内容: バージョン更新 + テキストファイル編集 + MDX 作成
- ドライバーテスト（v180000_tests）は include_str! でファイル存在・内容を確認するだけ

---

## 実装ステップ

### Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml` の `version` を `17.8.0` → `18.0.0` に変更。
`cargo build` を実行して `Cargo.lock` を更新。

---

### Step 2: CHANGELOG.md 更新

`CHANGELOG.md` の先頭に以下の形式でエントリを追記（既存エントリの前に挿入）：

```markdown
## v18.0.0 — Language Power（2026-06-16）
...

## v17.8.0 — パッケージシステム成熟（2026-06-16）
...
（v17.1.0〜v17.7.0 も同様）
```

---

### Step 3: README.md 更新

1. 「現在のバージョン」を v18.0.0 に更新
2. 主要機能リストに v17.x 系の機能を追記：
   - Bounded Generics (`fn f<T with Ord>(...)`)
   - Pattern matching extensions (or-pattern, list-pattern, guard)
   - Collection comprehensions (`[x * 2 | x <- list]`)
   - Property-based testing (`forall`)
   - Package system (`fav add`, `fav publish`)
3. バージョン履歴表に v17.1.0〜v18.0.0 エントリを追加

---

### Step 4: サイトドキュメント作成（4ファイル）

以下を並列作成：

#### `site/content/docs/language/patterns.mdx`

内容:
- or-pattern: `"active" | "pending" => ...`
- list-pattern: `[] / [x] / [head, ..tail]`
- guard: `if a > 1000.0 => high_value(row)`
- 実用例（パイプラインのステータス分岐）

#### `site/content/docs/language/comprehensions.mdx`

内容:
- 基本構文 `[expr | x <- src]`
- フィルタ付き `[expr | x <- src, guard]`
- 複数ソース `[Pair(a,b) | a <- as, b <- bs]`
- Result 内包 `[? f(x) | x <- xs]`
- Before/After 比較（`List.map + List.filter` との比較）

#### `site/content/docs/language/bind.mdx`

内容:
- `bind x <- expr`（Result・非 Result 両対応）
- `let` を使わない理由
- パイプライン内での使い方

#### `site/content/docs/packages/publishing.mdx`

内容:
- `fav.toml` の `[rune]` セクション設定
- `fav publish --dry-run` で確認
- `fav login` で認証
- `fav publish` で公開

---

### Step 5: driver.rs — v180000_tests 追加

`v178000_tests` の `version_is_17_8_0` を削除し、`v180000_tests` を追加。

```rust
#[cfg(test)]
mod v180000_tests {
    #[test]
    fn version_is_18_0_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("\"18.0.0\""), "Cargo.toml should have version 18.0.0");
    }

    #[test]
    fn changelog_has_v17_entries() {
        let changelog = include_str!("../../CHANGELOG.md");
        assert!(changelog.contains("v17."), "CHANGELOG.md should have v17.x entries");
    }

    #[test]
    fn readme_mentions_bounded_generics() {
        let readme = include_str!("../../README.md");
        assert!(
            readme.to_lowercase().contains("bounded generics") || readme.contains("Bounded Generics"),
            "README.md should mention bounded generics"
        );
    }

    #[test]
    fn readme_mentions_package_system() {
        let readme = include_str!("../../README.md");
        assert!(
            readme.contains("fav add") || readme.contains("package system"),
            "README.md should mention the package system"
        );
    }

    #[test]
    fn docs_generics_exists() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("site/content/docs/language/generics.mdx");
        assert!(path.exists(), "site/content/docs/language/generics.mdx should exist");
    }
}
```

---

## 依存関係

```
Step 1 (Cargo.toml)
    ↕ 独立
Step 2 (CHANGELOG.md)
    ↕ 独立
Step 3 (README.md)    ← Step 2 と整合性確認
Step 4 (MDX 4ファイル)  ← 並列作成可
    ↓
Step 5 (v180000_tests) ← Step 1〜4 すべて完了後
```

Steps 1〜4 はすべて並列実施可能。
Step 5 は最後（`include_str!` でファイルをコンパイル時に読み込むため）。

---

## 注意事項

- `CHANGELOG.md` / `README.md` の場所: プロジェクトルート `C:\Users\yoshi\favnir\`
- `include_str!("../../CHANGELOG.md")` は `fav/src/driver.rs` からの相対パス → `../../` でリポジトリルートを指す
- `CARGO_MANIFEST_DIR` は `fav/` ディレクトリを指す
- `site/content/docs/language/generics.mdx` は v17.1.0 ですでに作成済み（テストはそれを確認するだけ）
- `site/content/docs/packages/getting-started.mdx` は v17.8.0 で作成済み
- `site/content/docs/tools/property-testing.mdx` は v17.7.0 で作成済み
