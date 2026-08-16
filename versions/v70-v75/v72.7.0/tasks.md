# v72.7.0 タスクリスト — Hot Reload 改善（`fav watch` 2.0）

Date: 2026-08-12
Status: 完了

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `72.6.0` であることを確認
- [x] `cargo test` が 3635 tests pass（0 failures）であることを確認
- [x] `driver.rs` に `v726000_tests` モジュールが存在することを確認
- [x] `driver.rs` に `v727000_tests` が未存在であることを確認
- [x] `driver.rs` 内に `cmd_watch` が存在することを確認（既存実装の把握）
- [x] `driver.rs` 内の `"72.6.0"` 文字列（バージョンアサーション）の件数を grep で確認しておく

---

## T1: `WatchSession` 構造体 + `watch_session_on_change_label` 追加

- [x] `pub struct WatchSession` を追加した（`file: Option<String>` / `on_change_cmd: String` / `debounce_ms: u64`）
- [x] `pub fn watch_session_on_change_label(session: &WatchSession) -> String` を追加した
  - `format!("[watch] Running: {}", session.on_change_cmd)` を返す
- [x] `cargo build` でエラーがないことを確認

---

## T2: `cmd_watch2` 追加

- [x] `pub fn cmd_watch2(file: Option<&str>, on_change: &str, debounce_ms: u64)` を追加した
  - `WatchSession` を内部で構築する
  - `collect_watch_paths` を使ってファイルを収集する（`Vec<PathBuf>` を返す）
  - 変更検知ループ（既存 `cmd_watch` と同様）で `on_change_cmd` を実行する
  - Windows: `cmd /C <on_change_cmd>`、Unix: `sh -c <on_change_cmd>` で実行する
- [x] 既存 `cmd_watch` はそのまま保持していることを確認（後方互換）
- [x] `cargo build` でエラーがないことを確認

---

## T3: `main.rs` — `--on-change` フラグ対応

- [x] `cmd_watch2` を import リストに追加した
- [x] `Some("watch")` アームに `--on-change` フラグ検出を追加した
- [x] `--on-change` フラグがある場合は `cmd_watch2` を呼ぶ分岐を追加した
- [x] `--on-change` フラグがない場合は既存 `cmd_watch` を呼ぶことを確認
- [x] `cargo build` でエラーがないことを確認

---

## T4: `v727000_tests` モジュール追加

- [x] `v726000_tests` モジュールの直後に `v727000_tests` モジュールを追加した
- [x] `use super::{WatchSession, watch_session_on_change_label}` を追加した
- [x] `watch2_session_field_defaults` テストを実装した
  - `WatchSession { file: Some("pipeline.fav"), on_change_cmd: "fav check", debounce_ms: 500 }` を構築
  - `session.on_change_cmd == "fav check"` を assert
  - `session.debounce_ms == 500` を assert
- [x] `watch2_runs_custom_command` テストを実装した
  - `WatchSession { file: None, on_change_cmd: "fav check && fav run --dry-run", ... }` を構築
  - `watch_session_on_change_label(&session)` が `"fav check && fav run --dry-run"` を含むことを assert
- [x] `watch2_on_change_label_format` テストを実装した（code-reviewer [LOW] 指摘対応）
  - ラベルが `"[watch]"` で始まることを assert
  - ラベルが on_change_cmd を含むことを assert
- [x] `cargo test v727000` で 3 件 pass することを確認

---

## T5: `fav/Cargo.toml` バージョン更新 + `driver.rs` version アサーション更新

- [x] `fav/Cargo.toml` の `version = "72.6.0"` → `version = "72.7.0"` に変更した
- [x] `driver.rs` 内の `version = \"72.6.0\"` 文字列を `version = \"72.7.0\"` に replace_all した
- [x] `driver.rs` 内のエラーメッセージ `"Cargo.toml version should be 72.6.0"` を `"72.7.0"` に replace_all した
- [x] `driver.rs` 内のエラーメッセージ `"Cargo.toml should declare version 72.6.0"` を `"72.7.0"` に replace_all した
- [x] 残存 72.6.0 はコメント・セクションヘッダーのみで意図的保持を確認

---

## T6: 部分テスト確認

- [x] `cargo test v727000` で 3 件 pass することを確認

---

## T7: 全体テスト確認

- [x] `cargo test` 全体で 3638 tests pass（0 failures）であることを確認

---

## T8: `CHANGELOG.md` 更新

- [x] `## [v72.7.0]` エントリを先頭に追加した

---

## T9: `versions/current.md` 更新

- [x] 「進行中バージョン」を `v72.7.0`（Hot Reload 改善）に更新した
- [x] 「次に切る版」を `v72.8.0` に更新した

---

## T10: 最終確認（T8・T9 完了後）

- [x] `cargo test v727000` で 3 件 pass することを確認
- [x] `cargo test` 全体で 3638 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `72.7.0` であることを確認
- [x] `WatchSession` 構造体と `watch_session_on_change_label` が pub で存在することを確認
- [x] `cmd_watch2` が pub で存在することを確認
- [x] `main.rs` に `--on-change` フラグ対応が存在することを確認
- [x] `versions/current.md` の「進行中バージョン」が `v72.7.0` であることを確認
- [x] `versions/current.md` の「次に切る版」が `v72.8.0` であることを確認

---

## コードレビュー指摘対応

| 優先度 | 指摘 | 対応 |
|---|---|---|
| [HIGH] | `cmd_watch2` が初回起動時にコマンドを実行しない（`cmd_watch` との非対称性） | ウォッチループ前に `on_change_cmd` を初回実行するコードを追加 |
| [HIGH] | `--on-change` 値が別フラグ（`--debounce` 等）になりえる | main.rs で `starts_with("--")` チェックを追加 |
| [MED] | `cmd_watch2` に空文字列ガードがない | `on_change.is_empty()` チェックで `process::exit(1)` |
| [MED] | `WatchSession` に `#[derive(Debug)]` が欠落 | `#[derive(Debug)]` を追加 |
| [LOW] | `watch2_session_field_defaults` テスト名が実態と乖離 | コメントで「構造体フィールドの設定確認が目的」と明記 |
| [LOW] | `watch_session_on_change_label` のテストが 1 件のみ | `watch2_on_change_label_format` テストを追加（ラベルの prefix / コマンド含有を確認） |

---

## スコープ外（明示的除外）

- 差分ステージ検出（変更されたステージの上流のみ再実行）— v73.x 以降
- `~/fav_watch_history` 等の永続化
- サイト側ドキュメント更新（v73.x 以降）
- ファイルシステムイベント駆動への完全移行（既存の `notify` 統合は維持）
