# v73.4.0 実装計画 — 監査ログ + OpenLineage エクスポート

Date: 2026-08-13
Status: 計画中

---

## 前提確認

- `fav/Cargo.toml` version = "73.3.0"
- `cargo test` 3653 tests pass（0 failures）
- `driver.rs` に `v733000_tests` / `v733000_truncate_tests` が存在する

---

## 実装ステップ

### Step 1: `AuditLogEntry` 構造体 + `format_audit_log_entry` 追加

`driver.rs` の v73.3.0 実装コードの直後（`v733000_tests` より前）に追加:

```rust
// --- v73.4.0: Audit Log + OpenLineage Export ---

pub struct AuditLogEntry {
    pub run_id: String,
    pub parent_run_id: Option<String>,
    pub pipeline_name: String,
    pub status: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub row_count: Option<u64>,
}

pub fn format_audit_log_entry(entry: &AuditLogEntry) -> String {
    let parent = match &entry.parent_run_id {
        Some(id) => format!("\"{}\"", id),
        None => "null".to_string(),
    };
    let ended = match &entry.ended_at {
        Some(t) => format!("\"{}\"", t),
        None => "null".to_string(),
    };
    let rows = match entry.row_count {
        Some(n) => n.to_string(),
        None => "null".to_string(),
    };
    format!(
        "{{\"run_id\":\"{}\",\"parent_run_id\":{},\"pipeline_name\":\"{}\",\"status\":\"{}\",\"started_at\":\"{}\",\"ended_at\":{},\"row_count\":{}}}",
        entry.run_id, parent, entry.pipeline_name, entry.status, entry.started_at, ended, rows
    )
}
```

### Step 2: `OpenLineageEvent` 構造体 + `format_openlineage_event` 追加

Step 1 の直後に追加:

```rust
pub struct OpenLineageEvent {
    pub event_type: String,
    pub run_id: String,
    pub job_name: String,
    pub job_namespace: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub event_time: String,
}

pub fn format_openlineage_event(event: &OpenLineageEvent) -> String {
    let inputs_json = event.inputs.iter()
        .map(|s| format!("{{\"name\":\"{}\",\"namespace\":\"{}\"}}", s, event.job_namespace))
        .collect::<Vec<_>>()
        .join(",");
    let outputs_json = event.outputs.iter()
        .map(|s| format!("{{\"name\":\"{}\",\"namespace\":\"{}\"}}", s, event.job_namespace))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"eventType\":\"{}\",\"eventTime\":\"{}\",\"run\":{{\"runId\":\"{}\"}},\"job\":{{\"name\":\"{}\",\"namespace\":\"{}\"}},\"inputs\":[{}],\"outputs\":[{}]}}",
        event.event_type, event.event_time, event.run_id,
        event.job_name, event.job_namespace,
        inputs_json, outputs_json
    )
}
```

### Step 3: `cargo build` 確認

### Step 4: `v734000_tests` モジュール追加

`v733000_truncate_tests` の直後に追加:

```rust
#[cfg(test)]
mod v734000_tests {
    use super::{
        AuditLogEntry, OpenLineageEvent,
        format_audit_log_entry, format_openlineage_event,
    };

    #[test]
    fn audit_log_records_run_start_end() {
        let entry = AuditLogEntry {
            run_id: "run-001".to_string(),
            parent_run_id: Some("parent-000".to_string()),
            pipeline_name: "orders".to_string(),
            status: "completed".to_string(),
            started_at: "2026-08-13T00:00:00Z".to_string(),
            ended_at: Some("2026-08-13T00:01:00Z".to_string()),
            row_count: Some(1000),
        };
        let jsonl = format_audit_log_entry(&entry);
        assert!(jsonl.contains("\"run_id\":\"run-001\""));
        assert!(jsonl.contains("\"parent_run_id\":\"parent-000\""));
        assert!(jsonl.contains("\"status\":\"completed\""));
        assert!(jsonl.contains("\"row_count\":1000"));

        // parentRunId なし
        let entry2 = AuditLogEntry {
            run_id: "run-002".to_string(),
            parent_run_id: None,
            pipeline_name: "users".to_string(),
            status: "started".to_string(),
            started_at: "2026-08-13T00:02:00Z".to_string(),
            ended_at: None,
            row_count: None,
        };
        let jsonl2 = format_audit_log_entry(&entry2);
        assert!(jsonl2.contains("\"parent_run_id\":null"));
        assert!(jsonl2.contains("\"ended_at\":null"));
    }

    #[test]
    fn lineage_export_openlineage_format() {
        let event = OpenLineageEvent {
            event_type: "COMPLETE".to_string(),
            run_id: "abc-123".to_string(),
            job_name: "ProcessOrders".to_string(),
            job_namespace: "favnir".to_string(),
            inputs: vec!["orders_raw".to_string()],
            outputs: vec!["orders_processed".to_string()],
            event_time: "2026-08-13T00:00:00Z".to_string(),
        };
        let json = format_openlineage_event(&event);
        assert!(json.contains("\"eventType\":\"COMPLETE\""));
        assert!(json.contains("\"runId\":\"abc-123\""));
        assert!(json.contains("\"name\":\"ProcessOrders\""));
        assert!(json.contains("\"namespace\":\"favnir\""));
        assert!(json.contains("orders_raw"));
        assert!(json.contains("orders_processed"));

        // START イベント（入出力なし）
        let event2 = OpenLineageEvent {
            event_type: "START".to_string(),
            run_id: "def-456".to_string(),
            job_name: "InitPipeline".to_string(),
            job_namespace: "favnir".to_string(),
            inputs: vec![],
            outputs: vec![],
            event_time: "2026-08-13T00:00:00Z".to_string(),
        };
        let json2 = format_openlineage_event(&event2);
        assert!(json2.contains("\"eventType\":\"START\""));
        assert!(json2.contains("\"inputs\":[]"));
        assert!(json2.contains("\"outputs\":[]"));
    }
}
```

### Step 5: `cargo test v734000` で 2 件 pass 確認

### Step 6: バージョン更新

- `fav/Cargo.toml`: version = "73.3.0" → "73.4.0"
- `driver.rs`: `version = \"73.3.0\"` → `version = \"73.4.0\"`（replace_all）
- `driver.rs`: `"Cargo.toml version should be 73.3.0"` → `"73.4.0"`（replace_all）
- `driver.rs`: `"Cargo.toml should declare version 73.3.0"` → `"73.4.0"`（replace_all）

### Step 7: `cargo build` 確認

### Step 8: `cargo test` 全体確認（3655 tests pass）

### Step 9: `CHANGELOG.md` 更新

先頭に追加:
```
## [v73.4.0] — 2026-08-13

### Added
- `AuditLogEntry` 構造体 + `format_audit_log_entry`（JSONL 形式監査ログ生成）
- `OpenLineageEvent` 構造体 + `format_openlineage_event`（OpenLineage JSON 形式リネージエクスポート）
- runId / parentRunId によるパイプライン系譜追跡

### Tests
- `audit_log_records_run_start_end`: 2 件（JSONL フォーマット検証）
- `lineage_export_openlineage_format`: 2 件（OpenLineage フォーマット検証）
- 合計テスト数: 3655（+2）
```

### Step 10: `versions/current.md` 更新

- 最終更新 → 2026-08-13 (v73.4.0)
- 進行中バージョン → v73.4.0
- 次に切る版 → v73.5.0
