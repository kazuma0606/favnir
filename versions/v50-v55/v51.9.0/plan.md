# Plan: v51.9.0 — 安定化・コードフリーズ（Performance & Scale 前調整）

Date: 2026-07-20

---

## 実装順序

### Step 1 — 事前確認

- `cargo test` 3131 passed, 0 failed を確認（ベース確認）
- `cargo clippy -- -D warnings` クリーンであることを確認
- `site/content/docs/performance-overview.mdx` が**存在しない**ことを確認（新規作成対象）
- `include_str!` パスの確認:
  - `fav/src/driver.rs` から `../Cargo.toml` → `fav/Cargo.toml` ✓
  - `fav/src/driver.rs` から `../../site/content/docs/performance-overview.mdx` → `favnir/site/content/docs/performance-overview.mdx` ✓
- v51.8.0 にバージョンテスト（`cargo_toml_version_is_51_8_0`）がないことを確認（削除不要）

---

### Step 2 — `site/content/docs/performance-overview.mdx` 作成

`site/content/docs/` 直下に Performance & Scale 概要ページを新規作成する。

必須キーワード:
- `par` — 並列ステージ実行の説明
- `fav bench` — ベンチマーク・回帰検出の説明
- `Performance & Scale` — マイルストーン名

既存の詳細ページ（`runtime/parallel.mdx` / `tools/bench-regression.mdx` 等）へのリンクを含む骨子とする。

---

### Step 3 — `v51900_tests` 追加 + バージョン更新

`driver.rs` に `v51900_tests` モジュールを追加（2 件）:

```rust
mod v51900_tests {
    #[test]
    fn cargo_toml_version_is_51_9_0() {
        let content = include_str!("../Cargo.toml");
        assert!(content.contains("version = \"51.9.0\""),
            "Cargo.toml version should be 51.9.0");
    }

    #[test]
    fn perf_overview_doc_exists() {
        let src = include_str!("../../site/content/docs/performance-overview.mdx");
        assert!(src.contains("par"), "performance-overview.mdx must mention par");
        assert!(src.contains("fav bench"),
            "performance-overview.mdx must mention fav bench");
        assert!(src.contains("Performance & Scale"),
            "performance-overview.mdx must mention Performance & Scale");
    }
}
```

`Cargo.toml` を `"51.9.0"` に更新。v51.8.0 にバージョンテストはないため削除なし。

---

### Step 4 — 後処理

- `cargo test` 3133 passed, 0 failed を確認
- `cargo clippy -- -D warnings` クリーンを確認
- `CHANGELOG.md` に v51.9.0 エントリ追加
- `versions/current.md` を v51.9.0（3133 tests）に更新
- `roadmap-v51.1-v52.0.md` の v51.9.0 実績欄を更新
- `roadmap-v51.1-v52.0.md` の v52.0.0 テスト数推定（≥3135）が実態と整合するか確認
- `tasks.md` を COMPLETE に更新

---

## 変更ファイル一覧

| ファイル | 変更種別 |
|---|---|
| `site/content/docs/performance-overview.mdx` | 新規作成 |
| `fav/src/driver.rs` | `v51900_tests` 追加 |
| `fav/Cargo.toml` | version → `"51.9.0"` |
| `fav/Cargo.lock` | 自動更新（`cargo test` 時） |
| `CHANGELOG.md` | v51.9.0 エントリ追加 |
| `versions/current.md` | v51.9.0 に更新 |
| `versions/roadmap/roadmap-v51.1-v52.0.md` | v51.9.0 実績欄・v52.0.0 推定値更新 |
