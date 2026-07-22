# Spec: v51.9.0 — 安定化・コードフリーズ（Performance & Scale 前調整）

Date: 2026-07-20
Status: 設計中

---

## 目的

v52.0 「Performance & Scale」宣言の直前準備として:
1. 全 lint / clippy がクリーンであることを確認
2. `site/content/docs/performance-overview.mdx` の骨子（概要ドキュメント）を作成する

---

## 成果物

### `site/content/docs/performance-overview.mdx`

`site/content/docs/` 直下に新規作成する Performance & Scale 概要ページ。
v51.1〜v51.8 で実装した機能群を俯瞰的にまとめ、各詳細ページへのリンクを提供する。

frontmatter: `title` / `description` を含む（テスト外要件）。

必須キーワード（テストで検証）:
- `par` — 並列ステージ実行への言及
- `fav bench` — ベンチマーク・回帰検出への言及
- `Performance & Scale` — マイルストーン名への言及

---

## テスト仕様

### `cargo_toml_version_is_51_9_0`

```rust
let content = include_str!("../Cargo.toml");
assert!(content.contains("version = \"51.9.0\""),
    "Cargo.toml version should be 51.9.0");
```

### `perf_overview_doc_exists`

```rust
let src = include_str!("../../site/content/docs/performance-overview.mdx");
assert!(src.contains("par"), "performance-overview.mdx must mention par");
assert!(src.contains("fav bench"), "performance-overview.mdx must mention fav bench");
assert!(src.contains("Performance & Scale"),
    "performance-overview.mdx must mention Performance & Scale");
```

`include_str!` のパス:
- `fav/src/driver.rs` から `../Cargo.toml` → `fav/Cargo.toml`
- `fav/src/driver.rs` から `../../site/content/docs/performance-overview.mdx` → `favnir/site/content/docs/performance-overview.mdx`

---

## テスト数

- ベース: 3131（v51.8.0 完了時点）
- v51.8.0 はバージョンテストなし（削除なし）
- 新規追加: +2（`cargo_toml_version_is_51_9_0` + `perf_overview_doc_exists`）
- **完了後合計: 3133 tests passed, 0 failed**

---

## 完了条件

- `cargo clippy -- -D warnings` クリーン
- `site/content/docs/performance-overview.mdx` が作成され、`par` / `fav bench` / `Performance & Scale` を含む
- `fav/Cargo.toml` version → `"51.9.0"`
- `cargo test` 3133 passed, 0 failed
- `CHANGELOG.md` に v51.9.0 エントリ追加
- `versions/current.md` を v51.9.0（3133 tests）に更新
- `roadmap-v51.1-v52.0.md` の v51.9.0 実績欄・v52.0.0 推定値を更新
- v52.0.0 の「テスト数 ≥ 3135」要件は本バージョン完了後（3133）+ v52.0.0 の 4 テストで 3137 になり要件を満たすことを確認
