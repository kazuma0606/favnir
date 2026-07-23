# Plan: v48.8.0 — `fav rune` コマンド群（純粋ヘルパー関数追加）

## 作業順序

### Step 1: `driver.rs` に純粋ヘルパー関数追加

`cmd_publish` の直後に追加する:

1. `list_installed_runes(root: &Path) -> Vec<String>` — `runes/` ディレクトリ走査 + ソート
2. `get_rune_version(root: &Path, name: &str) -> Option<String>` — `[rune]` セクション内の `version` を `split_once('=')` で取得

### Step 2: `driver.rs` に `v488000_tests` 追加

`v487000_tests` の直前に挿入（2テスト）:
- `fav_rune_list_shows_installed`: `runes/kafka/` + `runes/postgres/` → ソート済み `["kafka", "postgres"]`
- `fav_rune_info_shows_version`: `runes/kafka/rune.toml` に `version = "2.1.0"` → `Some("2.1.0")`

### Step 3: `Cargo.toml` version 更新

`"48.7.0"` → `"48.8.0"`

### Step 4: 完了処理

- `cargo test` 3063 passed を確認
- `cargo clippy -- -D warnings` クリーン確認
- `CHANGELOG.md` に v48.8.0 エントリ追加
- `versions/current.md` 更新（v48.8.0・3063 tests・進行中 v48.9.0）
- `versions/roadmap/roadmap-v48.1-v49.0.md` の v48.8.0 実績を記入
- `tasks.md` を COMPLETE に更新

---

## 変更ファイル一覧

| ファイル | 変更種別 |
|---|---|
| `fav/src/driver.rs` | `list_installed_runes` / `get_rune_version` 追加 + `v488000_tests` 追加 |
| `fav/Cargo.toml` | version 更新 |
| `CHANGELOG.md` | v48.8.0 エントリ |
| `versions/current.md` | バージョン更新 |
| `versions/roadmap/roadmap-v48.1-v49.0.md` | 実績記入 |
| `versions/v45-v50/v48.8.0/tasks.md` | COMPLETE 更新 |

## 変更しないファイル

| ファイル | 理由 |
|---|---|
| `fav/src/main.rs` | `"rune"` アームは既存（`rune_cmd.rs` の `cmd_rune` にルーティング済み）|
| `fav/src/rune_cmd.rs` | `cmd_rune_list` / `cmd_rune_info` / `cmd_rune_uninstall` は既存（`rune_modules/` 対象）|
