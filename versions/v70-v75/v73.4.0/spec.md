# v73.4.0 仕様書 — 監査ログ + OpenLineage エクスポート

Date: 2026-08-13
Status: 計画中

---

## Background

Favnir v73.3.0 で PII 保護 Rune を実装した。
v73.4.0 では「監査ログ」と「OpenLineage エクスポート」を実装し、
エンタープライズデータパイプラインが要求する法的追跡可能性・リネージ可視化を提供する。

> **注**: ロードマップには `--audit-log <path>` CLI フラグや `fav lineage --export openlineage` コマンドが記載されているが、
> 本バージョンでは構造体（`AuditLogEntry` / `OpenLineageEvent`）とフォーマット関数の実装に絞る。
> CLI への組み込みは将来バージョン（v73.7.0 ドッグフーディング Sprint 等）で実施する。

主な成果物:
1. `AuditLogEntry` 構造体 + `format_audit_log_entry` 関数（JSONL 形式出力）
2. `OpenLineageEvent` 構造体 + `format_openlineage_event` 関数（OpenLineage JSON 出力）
3. `v734000_tests` モジュール（Rust テスト 2 件）

---

## Goals

| 優先度 | 目標 |
|---|---|
| P0 | `AuditLogEntry` — runId / parentRunId / 開始・完了時刻 / ステータスを記録 |
| P0 | `format_audit_log_entry` — JSONL 形式の文字列を生成 |
| P0 | `OpenLineageEvent` — ジョブ名 / runId / 入出力データセット / ステータスを記録 |
| P0 | `format_openlineage_event` — OpenLineage JSON 文字列を生成 |
| P0 | `v734000_tests` — 2 件（`audit_log_records_run_start_end` / `lineage_export_openlineage_format`） |

---

## API 設計

### `AuditLogEntry`

```rust
pub struct AuditLogEntry {
    pub run_id: String,
    pub parent_run_id: Option<String>,
    pub pipeline_name: String,
    pub status: String,      // "started" | "completed" | "failed"
    pub started_at: String,  // ISO 8601 文字列（スタブでは固定値）
    pub ended_at: Option<String>,
    pub row_count: Option<u64>,
}

pub fn format_audit_log_entry(entry: &AuditLogEntry) -> String
// → JSONL 1 行（JSON オブジェクト）を返す
// 例: {"run_id":"abc123","parent_run_id":null,"pipeline_name":"orders","status":"completed",...}
```

### `OpenLineageEvent`

```rust
pub struct OpenLineageEvent {
    pub event_type: String,      // "START" | "COMPLETE" | "FAIL"
    pub run_id: String,
    pub job_name: String,
    pub job_namespace: String,   // 例: "favnir"
    pub inputs: Vec<String>,     // データセット名リスト
    pub outputs: Vec<String>,
    pub event_time: String,      // ISO 8601 文字列（スタブでは固定値）
}

pub fn format_openlineage_event(event: &OpenLineageEvent) -> String
// → OpenLineage JSON 文字列を返す
// 例: {"eventType":"COMPLETE","run":{"runId":"abc123"},"job":{"name":"orders","namespace":"favnir"},...}
```

---

## スコープ外

- ファイルシステムへの実際の JSONL 書き込み（`--audit-log <path>` フラグは将来バージョン）
- Marquez / DataHub への HTTP 送信
- `main.rs` の `audit-log` / `lineage` コマンド追加（将来バージョン）
- 実行時フック（VM パイプライン実行への組み込み）

---

## 成功条件

1. `cargo build` がエラーなし
2. `cargo test v734000` で 2 件 pass
3. `cargo test` 全体で 3655 tests pass（3653 + 2）
4. `fav/Cargo.toml` version = "73.4.0"
5. `CHANGELOG.md` に `[v73.4.0]` エントリあり
6. `versions/current.md` の進行中バージョンが v73.4.0

---

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `fav/src/driver.rs` | `AuditLogEntry` / `format_audit_log_entry` / `OpenLineageEvent` / `format_openlineage_event` 追加、`v734000_tests` モジュール追加 |
| `fav/Cargo.toml` | version → "73.4.0" |
| `CHANGELOG.md` | v73.4.0 エントリ追加 |
| `versions/current.md` | 進行中バージョン・次バージョン更新 |
