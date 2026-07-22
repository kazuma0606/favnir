# Plan: v51.8.0 — ドキュメントサイト Performance 記事

Date: 2026-07-20

---

## 実装順序

### Step 1 — 事前確認

- `cargo test` 3130 passed, 0 failed を確認（ベース確認）
- `cargo clippy -- -D warnings` クリーンであることを確認
- `site/content/docs/runtime/` ディレクトリが存在しないことを確認（新規作成対象）
- `site/content/docs/tools/bench-regression.mdx` が存在しないことを確認（新規作成対象）
- `include_str!` パスの確認:
  - `driver.rs` は `fav/src/driver.rs`
  - `../../` → `favnir/`
  - `../../site/content/docs/runtime/parallel.mdx` → `favnir/site/content/docs/runtime/parallel.mdx`
  - `../../site/content/docs/tools/bench-regression.mdx` → `favnir/site/content/docs/tools/bench-regression.mdx`

---

### Step 2 — `site/content/docs/runtime/parallel.mdx` 作成

新規ディレクトリ `site/content/docs/runtime/` を作成し、`parallel.mdx` を配置する。

記事に必須キーワード（テストで検証）:
- `par` — `par [A, B] |> Merge` の構文説明
- `Merge` — `Merge.ordered` / `Merge.any` の使い分け説明
- `buffer_size` — バックプレッシャー設定（`fav.toml [stream]`）

---

### Step 3 — `site/content/docs/tools/bench-regression.mdx` 作成

既存の `site/content/docs/tools/bench.mdx` と重複しないよう、`--compare` フラグによる
差分回帰検出に特化した記事を追加する。

記事に必須キーワード（テストで検証）:
- `--compare` — `fav bench --compare <baseline.json>` の使い方
- `--fail-on-regression` — CI 向けフラグの説明

---

### Step 4 — `v51800_tests` 追加 + バージョン更新

`driver.rs` に `v51800_tests` モジュールを追加（2 件）:

```rust
mod v51800_tests {
    #[test]
    fn docs_parallel_page_exists() {
        let src = include_str!("../../site/content/docs/runtime/parallel.mdx");
        assert!(src.contains("par"), ...);
        assert!(src.contains("Merge"), ...);
        assert!(src.contains("buffer_size"), ...);
    }

    #[test]
    fn docs_bench_regression_page_exists() {
        let src = include_str!("../../site/content/docs/tools/bench-regression.mdx");
        assert!(src.contains("--compare"), ...);
        assert!(src.contains("--fail-on-regression"), ...);
    }
}
```

`v51700_tests` から `cargo_toml_version_is_51_7_0` を削除し、`Cargo.toml` を `"51.8.0"` に更新。

---

### Step 5 — 後処理

- `cargo test` 3131 passed, 0 failed を確認
- `cargo clippy -- -D warnings` クリーンを確認
- `CHANGELOG.md` に v51.8.0 エントリ追加
- `versions/current.md` を v51.8.0（3131 tests）に更新
- `roadmap-v51.1-v52.0.md` の v51.8.0 実績欄を更新
- `tasks.md` を COMPLETE に更新

---

## 変更ファイル一覧

| ファイル | 変更種別 |
|---|---|
| `site/content/docs/runtime/parallel.mdx` | 新規作成 |
| `site/content/docs/tools/bench-regression.mdx` | 新規作成 |
| `fav/src/driver.rs` | `v51800_tests` 追加、`cargo_toml_version_is_51_7_0` 削除 |
| `fav/Cargo.toml` | version → `"51.8.0"` |
| `CHANGELOG.md` | v51.8.0 エントリ追加 |
| `versions/current.md` | v51.8.0 に更新 |
| `versions/roadmap/roadmap-v51.1-v52.0.md` | v51.8.0 実績欄更新 |
