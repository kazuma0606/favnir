# Plan: v48.9.0 — Module ドキュメント + migration guide + v49.0 前調整

## 作業順序

### Step 1: MDX ファイル作成

1. `site/content/docs/module-system.mdx` 新規作成
2. `site/content/docs/migration-guide-import.mdx` 新規作成

### Step 2: `driver.rs` に `v489000_tests` 追加

`v488000_tests` の直前に挿入（2テスト）:
- `module_system_doc_exists`: `include_str!("../../site/content/docs/module-system.mdx")` → `import` 等のキーワード含有確認
- `import_migration_guide_exists`: `include_str!("../../site/content/docs/migration-guide-import.mdx")` → `import rune` / `migration` / `W035` 含有確認

### Step 3: `Cargo.toml` version 更新

`"48.8.0"` → `"48.9.0"`

### Step 4: 完了処理

- `cargo test` 3065 passed を確認
- `cargo clippy -- -D warnings` クリーン確認
- `CHANGELOG.md` に v48.9.0 エントリ追加
- `versions/current.md` 更新（v48.9.0・3065 tests・進行中 v49.0.0）
- `versions/roadmap/roadmap-v48.1-v49.0.md` の v48.9.0 実績を記入
- `tasks.md` を COMPLETE に更新

---

## 変更ファイル一覧

| ファイル | 変更種別 |
|---|---|
| `site/content/docs/module-system.mdx` | 新規作成 |
| `site/content/docs/migration-guide-import.mdx` | 新規作成 |
| `fav/src/driver.rs` | `v489000_tests` 追加（2テスト）|
| `fav/Cargo.toml` | version 更新 |
| `CHANGELOG.md` | v48.9.0 エントリ |
| `versions/current.md` | バージョン更新 |
| `versions/roadmap/roadmap-v48.1-v49.0.md` | 実績記入 |
| `versions/v45-v50/v48.9.0/tasks.md` | COMPLETE 更新 |

## 変更しないファイル

| ファイル | 理由 |
|---|---|
| `fav/src/main.rs` | コードフリーズ（v49.0 前） |
| `fav/src/lint.rs` | コードフリーズ（v49.0 前） |
| `fav/src/frontend/parser.rs` | コードフリーズ（v49.0 前） |
