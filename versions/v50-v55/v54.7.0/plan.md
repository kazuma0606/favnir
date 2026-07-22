# Plan: v54.7.0 — ドキュメントサイト Production 3.0 overview ページ

---

## ステップ 1: 事前確認

```bash
cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
# → 3197 passed, 0 failed を確認

cargo clippy -- -D warnings
# → warnings なしであることを確認

# v54700_tests が未存在を確認
rg -n "v54700_tests" fav/src/driver.rs  # → 0 件

# v54600_tests の行番号を確認（挿入位置）
rg -n "v54600_tests" fav/src/driver.rs

# production3-overview.mdx が未存在を確認
ls site/content/docs/production3-overview.mdx  # → not found

# Cargo.toml が 54.6.0 であることを確認
grep "^version" fav/Cargo.toml  # → version = "54.6.0"
```

---

## ステップ 2: `site/content/docs/production3-overview.mdx` — 新規作成

以下の構成で MDX ファイルを作成（詳細は spec.md §実装スコープ §1 参照）:

- タイトル: `# Production 3.0 — Favnir v55 への道のり`
- セクション: `## v51` / `## v52` / `## v53` / `## v54` / `## v55`
- 各セクションに主要機能リストを記載

帰属バージョンの注意:
- `--watch-diff` / `--watch-summary` → **v51 ではなく v54 セクションに記載**
- `MILESTONE.md` リンク → **含めない**（サイト構造から到達不可）

---

## ステップ 3: `driver.rs` — `v54700_tests` 追加

`v54600_tests` の直前に追加:

```rust
// -- v54700_tests (v54.7.0) -- ドキュメントサイト Production 3.0 overview ページ --
#[cfg(test)]
mod v54700_tests {
    use super::*;

    #[test]
    fn docs_production3_overview_exists() {
        let doc = include_str!("../../site/content/docs/production3-overview.mdx");
        assert!(!doc.is_empty(), "production3-overview.mdx should not be empty");
        assert!(doc.contains("Production 3.0"), "production3-overview.mdx should contain 'Production 3.0'");
    }

    #[test]
    fn docs_production3_has_v55() {
        let doc = include_str!("../../site/content/docs/production3-overview.mdx");
        assert!(doc.contains("v55"), "production3-overview.mdx should mention v55");
    }
}
```

`cargo build` → コンパイルエラーなし確認（`include_str!` パス検証）。

---

## ステップ 4: `fav/Cargo.toml` バージョン更新

`version = "54.6.0"` → `version = "54.7.0"`

---

## ステップ 5: テスト実行・確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
# 期待値: 3199 passed, 0 failed
```

```bash
cargo clippy -- -D warnings
# 期待値: warnings なし
```

---

## ステップ 6: 後処理

- `CHANGELOG.md`: v54.7.0 エントリ追加（v54.6.0 の直上）
- `versions/current.md` を v54.7.0（3199 tests）に更新
- `roadmap-v54.1-v55.0.md` の v54.7.0 実績欄を COMPLETE に更新
- `Cargo.lock` が `cargo test` / `cargo build` 実行で自動更新されていることを確認し、コミットに含める
- `tasks.md` を COMPLETE に更新（T0〜T6 全 `[x]`）

コードレビュー対応（実施済み）:
- [MED] `--watch-diff` / `--watch-summary` が v51 セクションに誤帰属 → v51 セクションから削除（v54 セクションに正しく記載済み）
- [LOW] `MILESTONE.md` への相対パスがサイト構造から到達不可 → リンクを削除
- [LOW] `.mdx` 拡張子リンク形式 — 既存 MDX と同形式のため対応不要
- [LOW] `par` の説明が既存実装との区別が曖昧 — overview ページとして許容範囲内のため対応不要
- [MED] テストのアサーションが浅い — v54.9.0 で `production3_overview_doc_complete` 拡充予定
