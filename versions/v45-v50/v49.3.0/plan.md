# Plan: v49.3.0 — `fav check` インクリメンタル型チェック

## 作業順序

### Step 1: `driver.rs` にヘルパー関数追加

`compute_file_fingerprint` / `file_needs_recheck` / `update_fingerprint_cache` の 3 関数を
`driver.rs` の既存ヘルパー群（`list_installed_runes` / `get_rune_version` の近く）に追加。

**依存ライブラリ確認**:
- `sha2 = "0.10"` — 既存（Cargo.toml 変更不要）
- `tempfile` — dev-dependencies に既存（テスト用）

### Step 2: `v493000_tests` 追加

`v492000_tests` の直前に挿入（2テスト）:
- `incremental_check_skips_unchanged`
- `incremental_check_detects_change`

両テストとも `tempfile::TempDir` でサンドボックス化。

### Step 3: `Cargo.toml` version 更新

`"49.2.0"` → `"49.3.0"`

### Step 4: 完了処理

- `cargo test` 3075 passed を確認
- `cargo clippy -- -D warnings` クリーン確認
- `CHANGELOG.md` に v49.3.0 エントリ追加
- `versions/current.md` 更新（v49.3.0・3075 tests・進行中 v49.4.0）
- `versions/roadmap/roadmap-v49.1-v50.0.md` の v49.3.0 実績を記入
- `tasks.md` を COMPLETE に更新

---

## 変更ファイル一覧

| ファイル | 変更種別 |
|---|---|
| `fav/src/driver.rs` | ヘルパー関数 3 件追加 + `v493000_tests` 追加（2テスト）|
| `fav/Cargo.toml` | version 更新 |
| `CHANGELOG.md` | v49.3.0 エントリ |
| `versions/current.md` | バージョン更新 |
| `versions/roadmap/roadmap-v49.1-v50.0.md` | 実績記入 |
| `versions/v45-v50/v49.3.0/tasks.md` | COMPLETE 更新 |

## 変更しないファイル

| ファイル | 理由 |
|---|---|
| `fav/src/middle/checker.rs` | Rust テスト検証のみ・実 checker への hookup は v50.0 以降 |
| `fav/Cargo.toml`（dependencies セクション） | `sha2` / `tempfile` は既存依存のため追加不要（version バンプのみ実施）|
| `site/` MDX | 新構文なし（ドキュメント更新は v49.4.0） |
