# v73.5.0 実装計画 — SLA 監視 + アラート統合

Date: 2026-08-13
Status: 計画中

---

## 前提確認

- `fav/Cargo.toml` version = "73.4.0"
- `cargo test` 3655 tests pass（0 failures）
- `driver.rs` に `v734000_tests` が存在する

---

## 実装ステップ

### Step 1: `SlaConfig` / `SlaAlertConfig` 構造体追加

`driver.rs` の v73.4.0 実装コードの直後（`v734000_tests` より前）に追加:

```rust
// --- v73.5.0: SLA Monitoring + Alert Integration ---

pub struct SlaConfig {
    pub max_latency_ms: u64,
    pub min_throughput: u64,
    pub max_error_rate: f64,
}

pub struct SlaAlertConfig {
    pub slack: Option<String>,
    pub pagerduty: Option<String>,
}
```

### Step 2: `parse_sla_config` 追加

Step 1 の直後に追加:

```rust
pub fn parse_sla_config(toml_str: &str) -> Result<SlaConfig, String> {
    let mut max_latency_ms: Option<u64> = None;
    let mut min_throughput: Option<u64> = None;
    let mut max_error_rate: Option<f64> = None;

    let mut in_sla = false;
    for line in toml_str.lines() {
        let trimmed = line.trim();
        if trimmed == "[sla]" {
            in_sla = true;
            continue;
        }
        if trimmed.starts_with('[') {
            in_sla = false;
        }
        if !in_sla {
            continue;
        }
        if let Some(val) = trimmed.strip_prefix("max_latency_ms") {
            let v = val.trim_start_matches(|c: char| c == ' ' || c == '=').trim();
            max_latency_ms = v.parse::<u64>().ok();
        } else if let Some(val) = trimmed.strip_prefix("min_throughput") {
            let v = val.trim_start_matches(|c: char| c == ' ' || c == '=').trim();
            min_throughput = v.parse::<u64>().ok();
        } else if let Some(val) = trimmed.strip_prefix("max_error_rate") {
            let v = val.trim_start_matches(|c: char| c == ' ' || c == '=').trim();
            max_error_rate = v.parse::<f64>().ok();
        }
    }

    match (max_latency_ms, min_throughput, max_error_rate) {
        (Some(l), Some(t), Some(e)) => Ok(SlaConfig {
            max_latency_ms: l,
            min_throughput: t,
            max_error_rate: e,
        }),
        _ => Err("missing required sla fields: max_latency_ms, min_throughput, max_error_rate".to_string()),
    }
}
```

### Step 3: `check_sla` 追加

```rust
pub fn check_sla(
    config: &SlaConfig,
    actual_latency_ms: u64,
    actual_throughput: u64,
    actual_error_rate: f64,
) -> Vec<String> {
    let mut violations = Vec::new();
    if actual_latency_ms > config.max_latency_ms {
        violations.push(format!(
            "latency exceeded: {}ms > {}ms",
            actual_latency_ms, config.max_latency_ms
        ));
    }
    if actual_throughput < config.min_throughput {
        violations.push(format!(
            "throughput below: {} < {} rows/sec",
            actual_throughput, config.min_throughput
        ));
    }
    if actual_error_rate > config.max_error_rate {
        violations.push(format!(
            "error_rate exceeded: {:.2}% > {:.2}%",
            actual_error_rate * 100.0, config.max_error_rate * 100.0
        ));
    }
    violations
}
```

### Step 4: `format_sla_alert` 追加

```rust
pub fn format_sla_alert(violations: &[String]) -> String {
    if violations.is_empty() {
        return "All SLA conditions met.".to_string();
    }
    let items = violations.iter()
        .map(|v| format!("  - {}", v))
        .collect::<Vec<_>>()
        .join("\n");
    format!("[SLA ALERT] {} violation(s):\n{}", violations.len(), items)
}
```

### Step 5: `cargo build` 確認

### Step 6: `v735000_tests` モジュール追加

`v734000_tests` の直後に追加:

```rust
#[cfg(test)]
mod v735000_tests {
    use super::{SlaConfig, parse_sla_config, check_sla, format_sla_alert};

    #[test]
    fn sla_violation_triggers_alert() {
        let config = SlaConfig {
            max_latency_ms: 5000,
            min_throughput: 1000,
            max_error_rate: 0.01,
        };

        // 全条件違反
        let violations = check_sla(&config, 6200, 800, 0.025);
        assert_eq!(violations.len(), 3);
        assert!(violations[0].contains("latency exceeded"));
        assert!(violations[1].contains("throughput below"));
        assert!(violations[2].contains("error_rate exceeded"));

        let alert = format_sla_alert(&violations);
        assert!(alert.contains("[SLA ALERT]"));
        assert!(alert.contains("3 violation(s)"));
        assert!(alert.contains("6200ms > 5000ms"));

        // 全条件 OK
        let ok = check_sla(&config, 4800, 1200, 0.005);
        assert!(ok.is_empty());
        let msg = format_sla_alert(&ok);
        assert_eq!(msg, "All SLA conditions met.");
    }

    #[test]
    fn sla_toml_config_parsed() {
        let toml_str = r#"
[sla]
max_latency_ms   = 5000
min_throughput   = 1000
max_error_rate   = 0.01

[sla.alerts]
slack = "https://hooks.slack.com/test"
"#;
        let config = parse_sla_config(toml_str).expect("should parse sla config");
        assert_eq!(config.max_latency_ms, 5000);
        assert_eq!(config.min_throughput, 1000);
        assert!((config.max_error_rate - 0.01).abs() < f64::EPSILON);

        // 必須フィールド欠落 → Err
        let bad = parse_sla_config("[sla]\nmax_latency_ms = 1000\n");
        assert!(bad.is_err());
        assert!(bad.unwrap_err().contains("missing"));
    }
}
```

### Step 7: `cargo test v735000` で 2 件 pass 確認

### Step 8: バージョン更新

- `fav/Cargo.toml`: version = "73.4.0" → "73.5.0"
- `driver.rs`: `"73.4.0"` → `"73.5.0"`（replace_all）
  ※ `driver.rs` 内の `cargo_toml_version_is_*` テスト等バージョン検証文字列リテラルも対象

### Step 9: `cargo build` 確認

### Step 10: `cargo test` 全体確認（3657 tests pass）

### Step 11: `CHANGELOG.md` 更新

### Step 12: `versions/current.md` 更新
