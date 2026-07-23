# Tasks: v48.8.0 — `fav rune` コマンド群（純粋ヘルパー関数追加）

Status: COMPLETE
Date: 2026-07-18

---

## T0 — 事前確認

- [x] `cargo test` 3061 passed, 0 failed を確認（ベース確認）
- [x] `driver.rs` に `list_installed_runes` が存在しないことを確認
- [x] `driver.rs` に `get_rune_version` が存在しないことを確認
- [x] `main.rs` に `"rune"` アームが既存（`cmd_rune` にルーティング）であることを確認（変更不要）

## T1 — `driver.rs` ヘルパー関数追加

- [x] `cmd_publish` の直後に `list_installed_runes` を追加（pub）
  - [x] `runes/` ディレクトリ内のサブディレクトリ名を Vec で返す
  - [x] `names.sort()` でソート済みを返す
- [x] `list_installed_runes` の直後に `get_rune_version` を追加（pub）
  - [x] `runes/<name>/rune.toml` を読み `[rune]` セクション内の `version` を返す
  - [x] `split_once('=')` + `k.trim() == "version"` で取得（`strip_prefix` は使わない）
  - [x] ファイル不在・フィールド不在の場合は `None`

## T2 — `driver.rs` テスト追加

- [x] `v488000_tests` モジュールを `v487000_tests` の直前に追加（2テスト）
  - [x] `fav_rune_list_shows_installed`: `runes/kafka/` + `runes/postgres/` → `["kafka", "postgres"]`（テスト側ソート不要、`list_installed_runes` がソート済みを返す）
  - [x] `fav_rune_info_shows_version`: `runes/kafka/rune.toml` に `version = "2.1.0"` → `Some("2.1.0")`

## T3 — バージョン更新・完了

- [x] `fav/Cargo.toml` version → `"48.8.0"`
- [x] `CHANGELOG.md` に v48.8.0 エントリ追加
- [x] `cargo test` 3063 passed, 0 failed（3061 + 2 件）
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `versions/current.md` を v48.8.0（3063 tests）に更新、進行中バージョンを `v48.9.0` に更新
- [x] `versions/roadmap/roadmap-v48.1-v49.0.md` の v48.8.0 テスト数を実績値 3063 に更新（`roadmap-v45.1-v50.0.md` への反映は v49.0.0 時・変更不要）
- [x] tasks.md を COMPLETE に更新（T0〜T3 全 `[x]`）

---

> **注記**: `main.rs` / `rune_cmd.rs` は変更しない（既存 `"rune"` アーム + `cmd_rune_list` / `cmd_rune_info` / `cmd_rune_uninstall` は `rune_modules/` 対象の旧系統として共存）
> **注記**: `fav rune info` の「関数一覧」表示は v48.8.0 のスコープ外
> **注記**: `cargo clean` はこのバージョンのスコープ外（v49.0.0 で実施）
