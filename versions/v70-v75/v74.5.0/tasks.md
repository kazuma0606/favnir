# v74.5.0 タスクリスト — Pipeline Scheduling（`fav schedule`）

Date: 2026-08-14
Status: COMPLETE

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `74.4.0` であることを確認
- [x] `cargo test` が 3678 tests pass（0 failures）であることを確認
- [x] `driver.rs` に `v744000_tests` モジュールが存在することを確認
- [x] `driver.rs` に `v745000_tests` が未存在であることを確認

---

## T1: 構造体 + 関数を `driver.rs` に追加

- [x] `// --- v74.5.0: Pipeline Scheduling（fav schedule） ---` セクションコメントを追加した
- [x] `#[derive(Debug, Clone, PartialEq)] pub struct ScheduleEntry` を追加した（name / cron / pipeline / notify）
- [x] `pub fn validate_cron_expr(expr: &str) -> bool` を実装した
  - スペース区切りで 5 フィールドなら `true`、それ以外は `false`
- [x] `pub fn cmd_schedule_list(entries: &[ScheduleEntry]) -> String` を実装した
  - 各エントリを `"name    cron    pipeline"` 形式で改行区切りに連結
  - 空スライスは空文字列を返す
- [x] `cargo build` でエラーがないことを確認

---

## T2: `v745000_tests` モジュールを追加

- [x] `v744000_tests` の直後に `v745000_tests` モジュールを追加した
- [x] `use super::{ScheduleEntry, validate_cron_expr, cmd_schedule_list}` を追加した
- [x] `schedule_add_parses_cron` テストを実装した
  - `ScheduleEntry` を構築し各フィールドを assert
  - `validate_cron_expr("0 9 * * *")` → `true`
  - `validate_cron_expr("invalid")` → `false`
  - `validate_cron_expr("0 9 * *")` → `false`（4 フィールド）
  - `validate_cron_expr("")` → `false`（空文字列）
- [x] `schedule_list_returns_entries` テストを実装した
  - 2 エントリの一覧で name / cron を含むことを assert
  - 空スライスで空文字列を返すことを assert

---

## T3: バージョン更新

- [x] `fav/Cargo.toml` の `version = "74.4.0"` → `version = "74.5.0"` に変更した
- [x] `driver.rs` 内の `version = "74.4.0"` 参照を `version = "74.5.0"` に replace_all した（コメント・セクションヘッダーは置換不要）
- [x] `version should be 74.4.0` を `version should be 74.5.0` に replace_all した（アサートメッセージのみ対象）
- [x] 残存 `74.4.0` はコメント・セクションヘッダーのみで意図的保持を確認
- [x] `cargo build` でエラーがないことを確認
- [x] `fav/Cargo.lock` が `version = "74.5.0"` を含むことを確認

---

## T3.5: バージョン更新後の部分テスト再確認

- [x] `cargo test v745000` で 2 件 pass することを確認

---

## T4: 全体テスト確認

- [x] `cargo test` 全体で 3680 tests pass（0 failures）であることを確認

---

## T5: `CHANGELOG.md` 更新

- [x] `## [v74.5.0]` エントリを先頭に追加した
  - Added: `ScheduleEntry` / `validate_cron_expr` / `cmd_schedule_list`
  - Tests: 2 件、合計テスト数 3680（+2）

---

## T6: `versions/current.md` 更新

- [x] 「最終更新」を `2026-08-14 (v74.5.0)` に更新した
- [x] 「進行中バージョン」を `v74.5.0` に更新した
- [x] 「次に切る版」を `v74.6.0` に更新した

---

## T7: 最終確認（T5・T6 完了後）

- [x] `cargo test v745000` で 2 件 pass することを確認
- [x] `cargo test` 全体で 3680 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `74.5.0` であることを確認
- [x] `CHANGELOG.md` に `[v74.5.0]` エントリが存在することを確認
- [x] `versions/current.md` の「進行中バージョン」が `v74.5.0` であることを確認

---

## スコープ外（明示的除外）

- `cmd_schedule_add` / `cmd_schedule_run` 関数の実装（後続バージョンで対応）
- 実際の cron デーモン・定期実行エンジン（後続バージョンで対応）
- `~/.fav_schedules.toml` への永続化（後続バージョンで対応）
- `--notify` の実際の通知送信（後続バージョンで対応）
- `site/` MDX 追加（後続バージョンで対応）
- MILESTONE.md 更新（宣言バージョンではないため不要）
