# Plan: v49.0.0 — Module & Package 2.0 宣言 ★クリーンアップ

## 作業順序

### Step 1: `MILESTONE.md` に Module & Package 2.0 エントリを先頭に追加

v48.0.0 エントリの直前に v49.0.0 ブロックを挿入する。

### Step 2: `README.md` に "Module & Package 2.0" を追加

v48.0 エントリ（`**v48.0 …Standard Library 2.0…**`）の直後に v49.0 の 2 行を追加する。

### Step 3: `driver.rs` に `v49000_tests` 追加

`v489000_tests` の直前に挿入（4テスト）:
- `cargo_toml_version_is_49_0_0`: `../Cargo.toml` に `version = "49.0.0"` が含まれる
- `changelog_has_v49_0_0`: `../../CHANGELOG.md` に `[v49.0.0]` が含まれる
- `milestone_has_module_package_v2`: `../../MILESTONE.md` に `"Module & Package 2.0"` が含まれる
- `readme_mentions_module_package_v2`: `../../README.md` に `"Module & Package 2.0"` が含まれる

### Step 4: `Cargo.toml` version 更新

`"48.9.0"` → `"49.0.0"`

### Step 5: `cargo test` 3069 passed を確認

### Step 6: `cargo clippy -- -D warnings` クリーン確認

### Step 7: `CHANGELOG.md` に v49.0.0 エントリ追加

### Step 8: `versions/current.md` 更新（v49.0.0・3069 tests・進行中 v49.1.0）

### Step 9: `roadmap-v48.1-v49.0.md` に v49.0.0 実績を記入

### Step 10: `roadmap-v45.1-v50.0.md` に v49.0 完了を反映（実績 3069 tests）

### Step 11: ★クリーンアップ（Step 5 の cargo test 全通過後に実施）

1. `cargo clean` 実施
2. `fav/tmp/hello.fav` 存在確認
3. `cargo test` 再実行（3069 passed を確認）

### Step 12: `tasks.md` を COMPLETE に更新

---

## 変更ファイル一覧

| ファイル | 変更種別 |
|---|---|
| `MILESTONE.md` | v49.0.0 エントリ先頭追加 |
| `README.md` | Module & Package 2.0 言及追加 |
| `fav/src/driver.rs` | `v49000_tests` 追加（4テスト）|
| `fav/Cargo.toml` | version 更新 |
| `CHANGELOG.md` | v49.0.0 エントリ |
| `versions/current.md` | バージョン更新 |
| `versions/roadmap/roadmap-v48.1-v49.0.md` | 実績記入 |
| `versions/roadmap/roadmap-v45.1-v50.0.md` | v49.0 完了反映 |
| `versions/v45-v50/v49.0.0/tasks.md` | COMPLETE 更新 |

## 変更しないファイル

| ファイル | 理由 |
|---|---|
| `fav/src/frontend/parser.rs` | コード変更なし（宣言・クリーンアップのみ）|
| `fav/src/middle/checker.rs` | コード変更なし |
