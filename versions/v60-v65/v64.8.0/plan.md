# v64.8.0 Plan — ドキュメントサイト Performance 1.0 総括記事

Version: 64.8.0
Status: 未着手

---

## 作業順序

### Step 1: 前提確認

- `site/content/docs/performance/performance1-overview.mdx` の非存在確認
- `driver.rs` に `v64700_tests` が存在し `v64800_tests` がないことを確認
- ベーステスト数 3445 の確認

### Step 2: `site/content/docs/performance/performance1-overview.mdx` 作成

spec.md §1 のコンテンツを使って作成。チェックポイント:
- `"Performance 1.0"` を含む（`docs_performance1_overview_exists` が通過するため）
- `"fav build"` / `"fav bench"` / `"fav profile"` / `"fav lint"` を含む（`docs_performance1_has_quickstart` が通過するため）

### Step 3: `driver.rs` — `v64800_tests` 追加

`// -- v64700_tests` コメント行の直前に以下を挿入:

```rust
// -- v64800_tests (v64.8.0) -- Performance 1.0 総括記事 --
#[cfg(test)]
mod v64800_tests {
    #[test]
    fn docs_performance1_overview_exists() {
        let content = include_str!("../../site/content/docs/performance/performance1-overview.mdx");
        assert!(!content.is_empty(), "performance1-overview.mdx should not be empty");
        assert!(
            content.contains("Performance 1.0"),
            "should mention 'Performance 1.0': {}",
            &content[..content.len().min(200)]
        );
    }

    #[test]
    fn docs_performance1_has_quickstart() {
        let content = include_str!("../../site/content/docs/performance/performance1-overview.mdx");
        assert!(
            content.contains("fav build"),
            "quickstart should mention 'fav build': {}",
            &content[..content.len().min(200)]
        );
        assert!(
            content.contains("fav bench"),
            "quickstart should mention 'fav bench': {}",
            &content[..content.len().min(200)]
        );
        assert!(
            content.contains("fav profile"),
            "quickstart should mention 'fav profile': {}",
            &content[..content.len().min(200)]
        );
        assert!(
            content.contains("fav lint"),
            "quickstart should mention 'fav lint': {}",
            &content[..content.len().min(200)]
        );
    }
}
```

### Step 4: ビルド・テスト

```bash
cargo build 2>&1 | tail -5
cargo test --bin fav v64800_tests 2>&1 | tail -10
cargo test -j 8 -- --test-threads=8 2>&1 | grep "^test result"
```

### Step 5: ドキュメント更新（T4）

- `CHANGELOG.md` 先頭に v64.8.0 エントリ追加
- `roadmap-v64.1-v65.0.md` v64.8.0 セクションに実績追記
- `versions/current.md` を v64.8.0（3447 tests）に更新
- `tasks.md` を COMPLETE に更新

---

## 注意事項

- `use super::*` は不要（`include_str!` のみ使用、`v64300_tests` と同パターン）
- `include_str!` パス: `"../../site/content/docs/performance/performance1-overview.mdx"`
  - `fav/src/driver.rs` から `../../` = `favnir/`（リポジトリルート）
- MDX のコードブロック内の `bash` フェンスがネスト問題を起こさないよう、
  spec の MDX コンテンツは実際のファイルでは正しいフェンス構文（` ``` ` ）を使う
