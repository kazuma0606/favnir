# v73.4.0 タスクリスト — 監査ログ + OpenLineage エクスポート

Date: 2026-08-13
Status: 完了

---

## T0: 事前確認

- [x] `fav/Cargo.toml` のバージョンが `73.3.0` であることを確認
- [x] `cargo test` が 3653 tests pass（0 failures）であることを確認
- [x] `driver.rs` に `v733000_tests` / `v733000_truncate_tests` モジュールが存在することを確認
- [x] `driver.rs` に `v734000_tests` が未存在であることを確認

---

## T1: `AuditLogEntry` 構造体 + `format_audit_log_entry` 追加

- [x] `AuditLogEntry { run_id, parent_run_id, pipeline_name, status, started_at, ended_at, row_count }` を `driver.rs` に追加した
- [x] `pub struct` であることを確認
- [x] `format_audit_log_entry(entry: &AuditLogEntry) -> String` を実装した
  - `parent_run_id: None` → `"parent_run_id":null`
  - `ended_at: None` → `"ended_at":null`
  - `row_count: None` → `"row_count":null`
- [x] `cargo build` でエラーがないことを確認

---

## T2: `OpenLineageEvent` 構造体 + `format_openlineage_event` 追加

- [x] `OpenLineageEvent { event_type, run_id, job_name, job_namespace, inputs, outputs, event_time }` を `driver.rs` に追加した
- [x] `pub struct` であることを確認
- [x] `format_openlineage_event(event: &OpenLineageEvent) -> String` を実装した
  - `inputs: vec![]` → `"inputs":[]`
  - `outputs: vec![]` → `"outputs":[]`
  - inputs/outputs は `{"name":"...","namespace":"..."}` 形式
- [x] `cargo build` でエラーがないことを確認

---

## T3: `v734000_tests` モジュール追加

- [x] `v733000_truncate_tests` の直後に `v734000_tests` モジュールを追加した
- [x] `use super::{AuditLogEntry, OpenLineageEvent, format_audit_log_entry, format_openlineage_event}` を追加した
- [x] `audit_log_records_run_start_end` テストを実装した
  - `run_id` / `parent_run_id` / `status` / `row_count` の含有を assert
  - `parent_run_id: None` → `"parent_run_id":null` を assert
  - `ended_at: None` → `"ended_at":null` を assert
- [x] `lineage_export_openlineage_format` テストを実装した
  - `eventType` / `runId` / `name` / `namespace` / 入出力データセット名の含有を assert
  - `inputs: vec![]` → `"inputs":[]` を assert
  - `outputs: vec![]` → `"outputs":[]` を assert
- [x] `cargo test v734000` で 2 件 pass することを確認

---

## T4: バージョン更新

- [x] `fav/Cargo.toml` の `version = "73.3.0"` → `version = "73.4.0"` に変更した
- [x] `driver.rs` 内の `version = \"73.3.0\"` を `version = \"73.4.0\"` に replace_all した
- [x] `driver.rs` 内のエラーメッセージ `"Cargo.toml version should be 73.3.0"` を `"73.4.0"` に replace_all した
- [x] `driver.rs` 内のエラーメッセージ `"Cargo.toml should declare version 73.3.0"` を `"73.4.0"` に replace_all した
- [x] 残存 `73.3.0` はコメント・セクションヘッダーのみで意図的保持を確認
- [x] `cargo build` 後に `fav/Cargo.lock` が `version = "73.4.0"` を含むことを確認

---

## T5: バージョン更新後の部分テスト確認

- [x] T4 のバージョン更新後も `cargo test v734000` で引き続き 2 件 pass することを確認

---

## T6: 全体テスト確認

- [x] `cargo test` 全体で 3655 tests pass（0 failures）であることを確認

---

## T7: `CHANGELOG.md` 更新

- [x] `## [v73.4.0]` エントリを先頭に追加した
  - Added: `AuditLogEntry` / `format_audit_log_entry` / `OpenLineageEvent` / `format_openlineage_event`
  - Tests: 2 件、合計テスト数 3655（+2）

---

## T8: `versions/current.md` 更新

- [x] 「最終更新」を `2026-08-13 (v73.4.0)` に更新した
- [x] 「進行中バージョン」を `v73.4.0` に更新した
- [x] 「次に切る版」を `v73.5.0` に更新した

---

## T9: 最終確認（T7・T8 完了後）

- [x] `cargo test v734000` で 2 件 pass することを確認
- [x] `cargo test` 全体で 3655 tests pass（0 failures）であることを確認
- [x] `fav/Cargo.toml` のバージョンが `73.4.0` であることを確認
- [x] `AuditLogEntry` / `format_audit_log_entry` / `OpenLineageEvent` / `format_openlineage_event` が pub で存在することを確認
- [x] `CHANGELOG.md` に `[v73.4.0]` エントリが存在することを確認
- [x] `versions/current.md` の「進行中バージョン」が `v73.4.0` であることを確認

---

## コードレビュー指摘対応

| 優先度 | 指摘 | 対応 |
|---|---|---|
| [HIGH] | 全文字列フィールドが JSON エスケープ未処理（`"` / `\` でJSON破壊） | `json_escape` ヘルパー（`serde_json::Value::String().to_string()`）を追加し全フィールドに適用 |
| [MED] | inputs 内の namespace フィールド出力をテストがアサートしていない | `"name":"orders_raw","namespace":"favnir"` のアサーションに強化 |
| [MED] | `ended_at` の Some 値・`event2` の `runId` 未確認 | `started_at` / `ended_at` の値確認・`event2` に `runId` アサーション追加 |
| [LOW] | `pub struct` フィールドへの doc コメント欠如 | `json_escape` に `///` コメント追加（struct フィールドは対応不要と判断） |
| [LOW] | `collect().join()` パターン | 現スケールで問題なし — 対応不要 |

---

## スコープ外（明示的除外）

- ファイルシステムへの実際の JSONL 書き込み（`--audit-log <path>` フラグ）
- Marquez / DataHub / OpenMetadata への HTTP 送信
- `main.rs` への `audit-log` / `lineage` コマンド追加（将来バージョン）
- 実行時フック（VM パイプライン実行への組み込み）
