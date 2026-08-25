# Plan: v81.5.0 — 来歴付き品質レポート（Provenance + Quality 統合）

## Step 1: 前提確認

- `cargo test` を実行し、3851 tests, 0 failures を確認する
- `fav/src/test_framework.rs` に v81.4.0 の `quality_grade` が定義済みであることを確認する

## Step 2: `fav/src/test_framework.rs` に追記

`quality_grade` の定義の直後に以下を追加する。

```rust
// ── v81.5.0: ProvenanceQualityReport ──────────────────────────────────────────

/// ソース単位の来歴付き品質エントリ。
///
/// `provenance_hash` は本バージョンでは `String` スタブ（Favnir 3.0 連携は将来対応）。
/// `quality_score` は 0.0〜1.0 の範囲。
#[derive(Debug, Clone)]
pub struct ProvenanceQualityEntry {
    pub source_name: String,
    pub provenance_hash: String,
    pub quality_score: f64,
}

/// パイプライン単位の来歴付き品質レポート。
#[derive(Debug)]
pub struct ProvenanceQualityReport {
    pub entries: Vec<ProvenanceQualityEntry>,
    pub pipeline_name: String,
}

/// `entries` と `pipeline` からレポートを構築する。
pub fn build_provenance_quality_report(
    entries: Vec<ProvenanceQualityEntry>,
    pipeline: &str,
) -> ProvenanceQualityReport {
    ProvenanceQualityReport { entries, pipeline_name: pipeline.to_string() }
}

/// レポートを人間向けの文字列に変換する。
///
/// 出力形式:
/// `"pipeline={name} sources={count}\n- {source}: score={:.3} hash={hash}\n..."`
pub fn format_provenance_quality_report(report: &ProvenanceQualityReport) -> String {
    let mut out = format!(
        "pipeline={} sources={}",
        report.pipeline_name,
        report.entries.len(),
    );
    for e in &report.entries {
        out.push_str(&format!(
            "\n- {}: score={:.3} hash={}",
            e.source_name, e.quality_score, e.provenance_hash,
        ));
    }
    out
}

/// `quality_score` が最も低いエントリを返す。空のとき `None`。
///
/// スコアが同値の場合は先に現れたエントリを返す。
pub fn worst_quality_source(report: &ProvenanceQualityReport) -> Option<&ProvenanceQualityEntry> {
    report.entries.iter().reduce(|worst, e| {
        if e.quality_score < worst.quality_score { e } else { worst }
    })
}
```

## Step 3: `fav/src/driver.rs` に `mod v81500_tests` を追加

`mod v81400_tests { ... }` の直後に追加する。

```rust
#[cfg(test)]
mod v81500_tests {
    use fav_core::test_framework::*;

    #[test]
    fn provenance_quality_report_built() {
        let entries = vec![
            ProvenanceQualityEntry {
                source_name: "db_A".to_string(),
                provenance_hash: "abc123".to_string(),
                quality_score: 0.95,
            },
            ProvenanceQualityEntry {
                source_name: "api_B".to_string(),
                provenance_hash: "def456".to_string(),
                quality_score: 0.72,
            },
        ];
        let report = build_provenance_quality_report(entries, "my_pipeline");
        assert_eq!(report.pipeline_name, "my_pipeline");
        assert_eq!(report.entries.len(), 2);
        let formatted = format_provenance_quality_report(&report);
        assert!(formatted.contains("pipeline=my_pipeline"), "should contain pipeline name: {formatted}");
        assert!(formatted.contains("sources=2"),            "should contain sources=2: {formatted}");
        assert!(formatted.contains("db_A"),                 "should contain db_A: {formatted}");
        assert!(formatted.contains("api_B"),                "should contain api_B: {formatted}");
    }

    #[test]
    fn worst_source_identified() {
        let entries = vec![
            ProvenanceQualityEntry {
                source_name: "high".to_string(),
                provenance_hash: "h1".to_string(),
                quality_score: 0.95,
            },
            ProvenanceQualityEntry {
                source_name: "low".to_string(),
                provenance_hash: "l1".to_string(),
                quality_score: 0.40,
            },
            ProvenanceQualityEntry {
                source_name: "mid".to_string(),
                provenance_hash: "m1".to_string(),
                quality_score: 0.75,
            },
        ];
        let report = build_provenance_quality_report(entries, "pipe");
        let worst = worst_quality_source(&report);
        assert!(worst.is_some(), "should find worst source");
        assert_eq!(worst.unwrap().source_name, "low", "worst should be 'low': {:?}", worst);

        // 空 entries → None
        let empty = build_provenance_quality_report(vec![], "empty_pipe");
        assert!(worst_quality_source(&empty).is_none(), "empty report should return None");
    }
}
```

## Step 4: `cargo test` で全 pass 確認

```
cargo test 2>&1 | grep "test result"
# 期待: 3853 tests, 0 failures
```

## Step 5: CHANGELOG 更新

`CHANGELOG.md` の先頭に v81.5.0 エントリを追加する。

## Step 6: CI 事前確認

以下はすべて `fav/` ディレクトリで実行する。

```
cargo clippy --locked -- -D warnings
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```
