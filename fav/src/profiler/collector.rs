// v19.8.0 — stage record collection and folded stack conversion.

#[derive(Debug, Clone)]
pub struct StageRecord {
    pub name: String,
    pub elapsed_ms: i64,
}

/// Parse the JSON output from `take_profile_dump_json()` into StageRecords.
/// Input format: `[{"name": "...", "ms": 42}]`
pub fn parse_profile_json(json: &str) -> Vec<StageRecord> {
    #[derive(serde::Deserialize)]
    struct Raw {
        name: String,
        ms: i64,
    }
    let raw: Vec<Raw> = serde_json::from_str(json).unwrap_or_default();
    raw.into_iter().map(|r| StageRecord { name: r.name, elapsed_ms: r.ms }).collect()
}

/// Convert records to inferno folded stack format.
/// Each line: `"pipeline;<stage_name> <weight>"` where weight = elapsed_ms (min 1).
pub fn to_folded_stacks(records: &[StageRecord]) -> Vec<String> {
    records
        .iter()
        .map(|r| format!("pipeline;{} {}", r.name, r.elapsed_ms.max(1)))
        .collect()
}

/// Average N sets of records (for `--runs=N`).
/// Stage order is taken from the first run; missing stages in later runs are skipped.
pub fn average_records(runs: Vec<Vec<StageRecord>>) -> Vec<StageRecord> {
    if runs.is_empty() {
        return vec![];
    }
    let n = runs.len() as i64;
    let names: Vec<String> = runs[0].iter().map(|r| r.name.clone()).collect();
    names
        .into_iter()
        .map(|name| {
            let total: i64 = runs
                .iter()
                .flat_map(|run| run.iter().filter(|r| r.name == name))
                .map(|r| r.elapsed_ms)
                .sum();
            StageRecord { name, elapsed_ms: total / n }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_json() {
        assert!(parse_profile_json("[]").is_empty());
        assert!(parse_profile_json("invalid").is_empty());
    }

    #[test]
    fn folded_stack_format() {
        let records =
            vec![StageRecord { name: "Load".into(), elapsed_ms: 100 }];
        let folded = to_folded_stacks(&records);
        assert_eq!(folded, vec!["pipeline;Load 100"]);
    }

    #[test]
    fn folded_stack_min_one() {
        let records = vec![StageRecord { name: "X".into(), elapsed_ms: 0 }];
        assert_eq!(to_folded_stacks(&records), vec!["pipeline;X 1"]);
    }

    #[test]
    fn average_two_runs() {
        let run1 = vec![
            StageRecord { name: "A".into(), elapsed_ms: 100 },
            StageRecord { name: "B".into(), elapsed_ms: 200 },
        ];
        let run2 = vec![
            StageRecord { name: "A".into(), elapsed_ms: 200 },
            StageRecord { name: "B".into(), elapsed_ms: 400 },
        ];
        let avg = average_records(vec![run1, run2]);
        assert_eq!(avg[0].elapsed_ms, 150);
        assert_eq!(avg[1].elapsed_ms, 300);
    }
}
