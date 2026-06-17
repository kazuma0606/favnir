// v19.8.0 — text and JSON profiling reports.

use super::collector::StageRecord;

/// Format a human-readable text report with HOT PATH marker on the slowest stage.
pub fn format_text_report(records: &[StageRecord], label: &str) -> String {
    let total_ms: i64 = records.iter().map(|r| r.elapsed_ms).sum();
    let max_ms = records.iter().map(|r| r.elapsed_ms).max().unwrap_or(0);
    let name_w = records.iter().map(|r| r.name.len()).max().unwrap_or(5).max(5);

    let mut out = String::new();
    out.push_str(&format!(
        "Profile: {} ({}ms total)\n\n",
        label, total_ms
    ));
    let header = format!(
        "{:<width$}  {:>10}  {:>6}",
        "Stage", "Time (ms)", "%",
        width = name_w
    );
    out.push_str(&header);
    out.push('\n');
    out.push_str(&"─".repeat(name_w + 22));
    out.push('\n');

    for r in records {
        let pct = if total_ms > 0 {
            r.elapsed_ms as f64 / total_ms as f64 * 100.0
        } else {
            0.0
        };
        let hot = if r.elapsed_ms == max_ms && max_ms > 0 {
            "  *** HOT PATH ***"
        } else {
            ""
        };
        out.push_str(&format!(
            "{:<width$}  {:>10}  {:>5.1}%{}",
            r.name, r.elapsed_ms, pct, hot,
            width = name_w
        ));
        out.push('\n');
    }

    out.push_str(&"─".repeat(name_w + 22));
    out.push('\n');
    out.push_str(&format!(
        "{:<width$}  {:>10}  {:>5.1}%",
        "Total", total_ms, 100.0,
        width = name_w
    ));
    out.push('\n');
    out
}

/// Format an enhanced JSON report with a `pct` field.
/// Output: `[{"stage": "...", "ms": N, "pct": N.N}]`
pub fn format_json_report(records: &[StageRecord]) -> String {
    let total_ms: i64 = records.iter().map(|r| r.elapsed_ms).sum();
    let entries: Vec<String> = records
        .iter()
        .map(|r| {
            let pct = if total_ms > 0 {
                (r.elapsed_ms as f64 / total_ms as f64 * 1000.0).round() / 10.0
            } else {
                0.0
            };
            format!(
                "  {{\"stage\": {}, \"ms\": {}, \"pct\": {}}}",
                serde_json::to_string(&r.name).unwrap_or_default(),
                r.elapsed_ms,
                pct
            )
        })
        .collect();
    format!("[\n{}\n]", entries.join(",\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<StageRecord> {
        vec![
            StageRecord { name: "Load".into(), elapsed_ms: 45 },
            StageRecord { name: "Transform".into(), elapsed_ms: 210 },
            StageRecord { name: "Save".into(), elapsed_ms: 30 },
        ]
    }

    #[test]
    fn text_report_contains_hot_path() {
        let report = format_text_report(&sample(), "test.fav");
        assert!(report.contains("HOT PATH"), "should have HOT PATH: {report}");
        let transform_line = report.lines().find(|l| l.contains("Transform")).unwrap_or("");
        assert!(transform_line.contains("HOT PATH"), "Transform should be HOT PATH");
    }

    #[test]
    fn json_report_has_pct() {
        let json = format_json_report(&sample());
        assert!(json.contains("\"pct\""));
        let v: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert!(v.is_array());
        assert_eq!(v.as_array().unwrap().len(), 3);
    }

    #[test]
    fn empty_records_no_panic() {
        let report = format_text_report(&[], "empty.fav");
        assert!(report.contains("0ms total"));
        let json = format_json_report(&[]);
        assert_eq!(json.trim(), "[\n\n]");
    }
}
