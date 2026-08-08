# v59.4.0 Tasks — Rune マーケットプレイス Phase 1（`fav marketplace`）

Date: 2026-07-29
Status: COMPLETE（2026-07-29）— 3316 tests passed, 0 failed

---

## T0: 事前確認

- [x] `cargo test` でベースラインが 3314 tests passed, 0 failed であることを確認
- [x] `fav/Cargo.toml` のバージョンが `"59.3.0"` であることを確認
- [x] `fav/src/driver.rs` に `cmd_marketplace_list` がまだ存在しないことを確認
- [x] `fav/src/driver.rs` に `v59400_tests` がまだ存在しないことを確認
- [x] `grep -c '59\.3\.0' fav/src/driver.rs` でローリング文字列件数を確認（14 件: assertion 7 件 + failure メッセージ 7 件）

---

## T1: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml`: `version = "59.3.0"` → `"59.4.0"`

---

## T2: roadmap 更新

- [x] `roadmap-v59.1-v60.0.md` の v59.5.0 ベース数を `3302 → 3316`、目標を `3304 → 3318` に修正
- **注意**: v59.6.0 以降のベース数も連鎖的にずれるが、各バージョン着手時に都度修正する運用とする（今回は v59.5.0 のみ修正）

---

## T3: driver.rs に cmd_marketplace_list / cmd_marketplace_publish 追加

- [x] `cmd_marketplace_list() -> i32` を追加（`cmd_cost_estimate` の直後）
  - Rune 一覧ヘッダ行と 3 件のエントリを出力
  - `0` を返す
- [x] `cmd_marketplace_publish(rune: &str) -> i32` を追加（`cmd_marketplace_list` の直後）
  - `Publishing rune '<name>'...` と `[OK] Rune '<name>' published successfully.` を出力
  - `0` を返す

---

## T4: driver.rs テストモジュール追加

- [x] **注意**: T3（関数追加）を先に行うこと
- [x] `v59400_tests` モジュールを `v59300_tests` の直前に挿入
  - [x] テスト関数名が pub fn と同名のため `use super::` は使わず `super::` 修飾のみで呼び出す（コンパイルエラー回避）
  - [x] `cmd_marketplace_list` テスト: `super::cmd_marketplace_list()` が `0` を返すことを検証
  - [x] `cmd_marketplace_publish` テスト: `super::cmd_marketplace_publish("my-rune")` が `0` を返すことを検証

---

## T5: main.rs 更新

- [x] `use crate::driver::` インポートに `cmd_marketplace_list`・`cmd_marketplace_publish` を追加
- [x] `Some("marketplace")` アームを `Some("cost-estimate")` の直前に追加
  - `list` → `cmd_marketplace_list()` を呼んで `process::exit(code)`
  - `search <query>` → 検索スタブ出力後 `process::exit(0)`
  - `publish --rune <name>` → `let mut rune_name: &str = "";`（型注釈明示）→ `cmd_marketplace_publish(rune_name)` → `process::exit(code)`
  - `--rune` 未指定 → `eprintln!` + `exit(1)`
  - 不明サブコマンド → `eprintln!` + `exit(1)`
- [x] HELP テキストに `marketplace list|search <q>|publish --rune <n>` を追加（`sla report` の直前）

---

## T6: driver.rs ローリングチェック更新

- [x] `version = \"59.3.0\"` → `\"59.4.0\"` に一括更新（7 件）
- [x] failure メッセージ `"Cargo.toml version should be 59.3.0"` → `"59.4.0"` に更新（7 件）
  - `cargo_toml_version_is_59_0_0`（ローリング）
  - `cargo_toml_version_is_58_9_0`（ローリング）
  - `cargo_toml_version_is_58_0_0`（ローリング）
  - `cargo_toml_version_is_57_9_0`（ローリング）
  - `cargo_toml_version_is_57_0_0`（`rolling check from v57.0.0` 付き）
  - `cargo_toml_version_is_56_9_0`（`rolling check from v56.9.0` 付き）
  - `cargo_toml_version_is_56_3_0`（ローリング）

---

## T7: テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` を実行
- [x] `v59400_tests::cmd_marketplace_list` pass を確認
- [x] `v59400_tests::cmd_marketplace_publish` pass を確認
- [x] 総テスト数 **3316** tests passed, 0 failed を確認
- [x] failures=0 であることを確認（全既存テストが通過）

---

## T8: 事後処理

- [x] `CHANGELOG.md` に v59.4.0 エントリを追加
- [x] `versions/current.md` を v59.4.0 / 3316 tests に更新
- [x] `versions/roadmap/roadmap-v59.1-v60.0.md` の v59.4.0 実績欄を更新
- [x] v59.5.0 ベース数を実績値（3316）に確定（T2 で修正済み）
- [x] このファイル（tasks.md）を COMPLETE ステータスに更新

---

## コードレビュー記録

spec-reviewer レビュー実施（2026-07-30）— 4 件の指摘:
- [HIGH] ロードマップの Private Registry 記述に「Phase 2 延期」注記を追加 → 対応済み
- [MED] spec.md Status を「設計中」→「COMPLETE（2026-07-29）」に更新 → 対応済み
- [MED] ロードマップ v59.6.0〜v59.9.0 のベース数に着手時更新注記 → 次バージョン着手時対応
- [LOW] コードレビュー記録欄が空 → 本記録で対応済み

---

Status: COMPLETE（2026-07-29）— 3316 tests passed, 0 failed
